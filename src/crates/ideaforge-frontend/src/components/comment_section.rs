use std::collections::HashMap;

use leptos::prelude::*;
use leptos_router::components::A;
use wasm_bindgen::JsCast;

use crate::api;
use crate::api::types::{ContributionResponse, CreateContributionRequest};
use crate::state::auth::AuthState;

#[component]
pub fn CommentSection(idea_id: String) -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let comments = RwSignal::new(Vec::<ContributionResponse>::new());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(String::new());
    let submit_loading = RwSignal::new(false);
    let user_names: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());

    let idea_id_stored = StoredValue::new(idea_id.clone());
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();

    // Load comments on mount, then fetch display names
    {
        let idea_id = idea_id.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api::contributions::list_contributions(&idea_id, Some("comment"), 1, 50).await {
                Ok(resp) => {
                    // Collect unique user IDs
                    let unique_ids: Vec<String> = {
                        let mut ids = Vec::new();
                        for c in &resp.data {
                            if !ids.contains(&c.user_id) {
                                ids.push(c.user_id.clone());
                            }
                        }
                        ids
                    };

                    comments.set(resp.data);

                    // Fetch display names for each unique user
                    for uid in unique_ids {
                        let uid_clone = uid.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            if let Ok(profile) = api::users::get_user(&uid_clone).await {
                                user_names.update(|map| {
                                    map.insert(uid_clone, profile.display_name);
                                });
                            }
                        });
                    }
                }
                Err(e) => error.set(e.message),
            }
            loading.set(false);
        });
    }

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if submit_loading.get_untracked() {
            return;
        }

        let body = textarea_ref
            .get()
            .map(|el| {
                let el: &web_sys::HtmlTextAreaElement = el.unchecked_ref();
                el.value()
            })
            .unwrap_or_default();

        if body.trim().is_empty() {
            return;
        }

        submit_loading.set(true);
        let idea_id = idea_id_stored.get_value();

        wasm_bindgen_futures::spawn_local(async move {
            let req = CreateContributionRequest {
                contribution_type: "comment".to_string(),
                title: None,
                body,
            };
            match api::contributions::create_contribution(&idea_id, req).await {
                Ok(comment) => {
                    // Fetch the display name for the new comment's author
                    let new_uid = comment.user_id.clone();
                    let names = user_names.get_untracked();
                    if !names.contains_key(&new_uid) {
                        let uid_clone = new_uid.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            if let Ok(profile) = api::users::get_user(&uid_clone).await {
                                user_names.update(|map| {
                                    map.insert(uid_clone, profile.display_name);
                                });
                            }
                        });
                    }

                    comments.update(|c| c.push(comment));
                    // Clear textarea
                    if let Some(el) = textarea_ref.get() {
                        let el: &web_sys::HtmlTextAreaElement = el.unchecked_ref();
                        el.set_value("");
                    }
                }
                Err(e) => error.set(e.message),
            }
            submit_loading.set(false);
        });
    };

    view! {
        <div class="comment-section">
            <h3>"Comments"</h3>

            // Error display
            {move || {
                let err = error.get();
                if err.is_empty() {
                    view! { <div></div> }.into_any()
                } else {
                    view! { <div class="form-error">{err}</div> }.into_any()
                }
            }}

            // Comment list
            {move || {
                if loading.get() {
                    view! { <p class="text-muted">"Loading comments..."</p> }.into_any()
                } else {
                    let items = comments.get();
                    let names = user_names.get();
                    if items.is_empty() {
                        view! { <p class="text-muted">"No comments yet. Be the first!"</p> }.into_any()
                    } else {
                        view! {
                            <div class="comment-list">
                                {items
                                    .into_iter()
                                    .map(|c| {
                                        let date = c
                                            .created_at
                                            .split('T')
                                            .next()
                                            .unwrap_or("")
                                            .to_string();
                                        let display_name = names
                                            .get(&c.user_id)
                                            .cloned()
                                            .unwrap_or_else(|| "Anonymous".to_string());
                                        let profile_url = format!("/profile/{}", c.user_id);
                                        view! {
                                            <div class="comment-item card">
                                                <div class="comment-meta">
                                                    <A href=profile_url attr:class="comment-author">
                                                        {display_name}
                                                    </A>
                                                    <span class="comment-date">{date}</span>
                                                </div>
                                                <p class="comment-body">{c.body.clone()}</p>
                                            </div>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </div>
                        }
                            .into_any()
                    }
                }
            }}

            // Comment form (only if authenticated)
            {move || {
                if auth.is_authenticated() {
                    view! {
                        <form class="comment-form mt-md" on:submit=on_submit>
                            <textarea
                                node_ref=textarea_ref
                                class="form-input"
                                placeholder="Add a comment..."
                                rows="3"
                                required
                            ></textarea>
                            <button
                                class="btn btn-primary btn-sm mt-sm"
                                type="submit"
                                disabled=move || submit_loading.get()
                            >
                                {move || {
                                    if submit_loading.get() { "Posting..." } else { "Post Comment" }
                                }}
                            </button>
                        </form>
                    }
                        .into_any()
                } else {
                    view! { <p class="text-muted">"Sign in to comment"</p> }.into_any()
                }
            }}
        </div>
    }
}
