use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::api;
use crate::api::types::{ContributionResponse, CreateContributionRequest};
use crate::state::auth::AuthState;

#[component]
pub fn SuggestionSection(idea_id: String) -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let suggestions = RwSignal::new(Vec::<ContributionResponse>::new());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(String::new());
    let submit_loading = RwSignal::new(false);

    let idea_id_stored = StoredValue::new(idea_id.clone());
    let title_ref = NodeRef::<leptos::html::Input>::new();
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();

    // Load suggestions on mount
    {
        let idea_id = idea_id.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api::contributions::list_contributions(&idea_id, Some("suggestion"), 1, 50).await
            {
                Ok(resp) => suggestions.set(resp.data),
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

        let title = title_ref
            .get()
            .map(|el| {
                let el: &web_sys::HtmlInputElement = el.unchecked_ref();
                el.value()
            })
            .unwrap_or_default();

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

        let title_opt = if title.trim().is_empty() {
            None
        } else {
            Some(title)
        };

        wasm_bindgen_futures::spawn_local(async move {
            let req = CreateContributionRequest {
                contribution_type: "suggestion".to_string(),
                title: title_opt,
                body,
            };
            match api::contributions::create_contribution(&idea_id, req).await {
                Ok(suggestion) => {
                    suggestions.update(|s| s.push(suggestion));
                    // Clear form fields
                    if let Some(el) = title_ref.get() {
                        let el: &web_sys::HtmlInputElement = el.unchecked_ref();
                        el.set_value("");
                    }
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
        <div class="suggestion-section">
            <h3>"Suggestions"</h3>

            // Error display
            {move || {
                let err = error.get();
                if err.is_empty() {
                    view! { <div></div> }.into_any()
                } else {
                    view! { <div class="form-error">{err}</div> }.into_any()
                }
            }}

            // Suggestion list
            {move || {
                if loading.get() {
                    view! { <p class="text-muted">"Loading suggestions..."</p> }.into_any()
                } else {
                    let items = suggestions.get();
                    if items.is_empty() {
                        view! {
                            <p class="text-muted">"No suggestions yet. Share your ideas!"</p>
                        }
                            .into_any()
                    } else {
                        view! {
                            <div class="suggestion-list">
                                {items
                                    .into_iter()
                                    .map(|s| {
                                        let date = s
                                            .created_at
                                            .split('T')
                                            .next()
                                            .unwrap_or("")
                                            .to_string();
                                        let title_display = s.title.clone().unwrap_or_default();
                                        let has_title = !title_display.is_empty();
                                        view! {
                                            <div class="suggestion-item card">
                                                <div class="suggestion-meta">
                                                    <span class="suggestion-date">{date}</span>
                                                </div>
                                                {if has_title {
                                                    view! {
                                                        <h4 class="suggestion-title">{title_display}</h4>
                                                    }
                                                        .into_any()
                                                } else {
                                                    view! { <div></div> }.into_any()
                                                }}
                                                <p class="suggestion-body">{s.body.clone()}</p>
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

            // Suggestion form (only if authenticated)
            {move || {
                if auth.is_authenticated() {
                    view! {
                        <form class="suggestion-form mt-md" on:submit=on_submit>
                            <input
                                node_ref=title_ref
                                type="text"
                                class="form-input mb-sm"
                                placeholder="Suggestion title (optional)"
                            />
                            <textarea
                                node_ref=textarea_ref
                                class="form-input"
                                placeholder="Describe your suggestion..."
                                rows="4"
                                required
                            ></textarea>
                            <button
                                class="btn btn-primary btn-sm mt-sm"
                                type="submit"
                                disabled=move || submit_loading.get()
                            >
                                {move || {
                                    if submit_loading.get() {
                                        "Submitting..."
                                    } else {
                                        "Submit Suggestion"
                                    }
                                }}
                            </button>
                        </form>
                    }
                        .into_any()
                } else {
                    view! { <p class="text-muted">"Sign in to suggest"</p> }.into_any()
                }
            }}
        </div>
    }
}
