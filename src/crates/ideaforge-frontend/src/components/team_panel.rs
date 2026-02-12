use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::api;
use crate::api::types::TeamMemberResponse;
use crate::state::auth::AuthState;

#[component]
pub fn TeamPanel(idea_id: String, author_id: String) -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let team_members = RwSignal::new(Vec::<TeamMemberResponse>::new());
    let loading = RwSignal::new(true);
    let apply_loading = RwSignal::new(false);
    let apply_success = RwSignal::new(false);
    let apply_error = RwSignal::new(String::new());

    let idea_id_stored = StoredValue::new(idea_id.clone());
    let message_ref = NodeRef::<leptos::html::Textarea>::new();

    // Check if current user is the author
    let is_author = {
        let author_id = author_id.clone();
        Memo::new(move |_| {
            auth.user
                .get()
                .map(|u| u.id == author_id)
                .unwrap_or(false)
        })
    };

    // Load team members on mount
    {
        let idea_id = idea_id.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api::team::list_team_members(&idea_id).await {
                Ok(members) => team_members.set(members),
                Err(_) => {} // silently fail for now
            }
            loading.set(false);
        });
    }

    let on_apply = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if apply_loading.get_untracked() {
            return;
        }

        let message = message_ref
            .get()
            .map(|el| {
                let el: &web_sys::HtmlTextAreaElement = el.unchecked_ref();
                el.value()
            })
            .unwrap_or_default();

        if message.trim().is_empty() {
            return;
        }

        apply_loading.set(true);
        let idea_id = idea_id_stored.get_value();

        wasm_bindgen_futures::spawn_local(async move {
            match api::team::apply_to_team(&idea_id, message).await {
                Ok(_) => {
                    apply_success.set(true);
                }
                Err(e) => apply_error.set(e.message),
            }
            apply_loading.set(false);
        });
    };

    view! {
        <div class="team-panel">
            <h3>"Team"</h3>

            // Team members list
            {move || {
                if loading.get() {
                    view! { <p class="text-muted">"Loading team..."</p> }.into_any()
                } else {
                    let members = team_members.get();
                    if members.is_empty() {
                        view! { <p class="text-muted">"No team members yet."</p> }.into_any()
                    } else {
                        view! {
                            <div class="team-members-list">
                                {members
                                    .into_iter()
                                    .map(|m| {
                                        view! {
                                            <div class="team-member-item">
                                                <span class="team-member-name">
                                                    {m.display_name.clone()}
                                                </span>
                                                <span class="badge">{m.role.clone()}</span>
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

            // Apply form (only if authenticated and not the author)
            {move || {
                if !auth.is_authenticated() || is_author.get() {
                    view! { <div></div> }.into_any()
                } else if apply_success.get() {
                    view! {
                        <p class="text-success">
                            "Application submitted! The author will review it."
                        </p>
                    }
                        .into_any()
                } else {
                    view! {
                        <div class="team-apply mt-md">
                            <h4>"Join This Idea"</h4>
                            {move || {
                                let err = apply_error.get();
                                if err.is_empty() {
                                    view! { <div></div> }.into_any()
                                } else {
                                    view! { <div class="form-error">{err}</div> }.into_any()
                                }
                            }}
                            <form on:submit=on_apply>
                                <textarea
                                    node_ref=message_ref
                                    class="form-input"
                                    placeholder="Why do you want to join? What skills can you bring?"
                                    rows="3"
                                    required
                                ></textarea>
                                <button
                                    class="btn btn-primary btn-sm mt-sm"
                                    type="submit"
                                    disabled=move || apply_loading.get()
                                >
                                    {move || {
                                        if apply_loading.get() {
                                            "Applying..."
                                        } else {
                                            "Apply to Join"
                                        }
                                    }}
                                </button>
                            </form>
                        </div>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}
