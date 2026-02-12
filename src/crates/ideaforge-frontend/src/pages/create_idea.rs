use leptos::prelude::*;
use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};

use crate::api;
use crate::api::types::CreateIdeaRequest;
use crate::components::protected::Protected;

#[component]
pub fn CreateIdeaPage() -> impl IntoView {
    view! {
        <Protected>
            <CreateIdeaForm />
        </Protected>
    }
}

#[component]
fn CreateIdeaForm() -> impl IntoView {
    let error = RwSignal::new(String::new());
    let loading = RwSignal::new(false);

    let title_ref = NodeRef::<leptos::html::Input>::new();
    let summary_ref = NodeRef::<leptos::html::Textarea>::new();
    let description_ref = NodeRef::<leptos::html::Textarea>::new();
    let openness_ref = NodeRef::<leptos::html::Select>::new();
    let category_ref = NodeRef::<leptos::html::Select>::new();

    let categories = LocalResource::new(move || async move {
        api::categories::list_categories().await.unwrap_or_default()
    });

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if loading.get_untracked() {
            return;
        }

        let title = title_ref.get().map(|el| {
            let el: &HtmlInputElement = &el;
            el.value()
        }).unwrap_or_default();
        let summary = summary_ref.get().map(|el| {
            let el: &HtmlTextAreaElement = &el;
            el.value()
        }).unwrap_or_default();
        let description = description_ref.get().map(|el| {
            let el: &HtmlTextAreaElement = &el;
            el.value()
        }).unwrap_or_default();
        let openness = openness_ref.get().map(|el| {
            let el: &HtmlSelectElement = &el;
            el.value()
        }).unwrap_or_default();
        let category_id = category_ref.get().map(|el| {
            let el: &HtmlSelectElement = &el;
            el.value()
        }).unwrap_or_default();

        if title.is_empty() || summary.is_empty() || description.is_empty() {
            error.set("Title, summary, and description are required".into());
            return;
        }

        loading.set(true);
        error.set(String::new());

        let req = CreateIdeaRequest {
            title,
            summary,
            description,
            openness: if openness.is_empty() { None } else { Some(openness) },
            category_id: if category_id.is_empty() { None } else { Some(category_id) },
        };

        wasm_bindgen_futures::spawn_local(async move {
            match api::ideas::create_idea(req).await {
                Ok(idea) => {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href(&format!("/ideas/{}", idea.id));
                    }
                }
                Err(e) => {
                    error.set(e.message);
                    loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="settings-container">
            <h1 class="page-title mb-lg">"Bring Your Idea to the Forge"</h1>

            {move || {
                let err = error.get();
                if err.is_empty() {
                    view! { <div></div> }.into_any()
                } else {
                    view! { <div class="form-error mb-md">{err}</div> }.into_any()
                }
            }}

            <form on:submit=on_submit>
                <div class="form-group">
                    <label class="form-label" for="title">"Title"</label>
                    <input
                        node_ref=title_ref
                        class="form-input"
                        type="text"
                        id="title"
                        placeholder="What's your idea called?"
                        maxlength="200"
                        required
                    />
                    <span class="form-help">"Max 200 characters"</span>
                </div>

                <div class="form-group">
                    <label class="form-label" for="summary">"Summary"</label>
                    <textarea
                        node_ref=summary_ref
                        class="form-textarea"
                        id="summary"
                        placeholder="A brief pitch — what problem does it solve?"
                        maxlength="500"
                        rows="3"
                        required
                    ></textarea>
                    <span class="form-help">"Max 500 characters"</span>
                </div>

                <div class="form-group">
                    <label class="form-label" for="description">"Full Description"</label>
                    <textarea
                        node_ref=description_ref
                        class="form-textarea"
                        id="description"
                        placeholder="Describe your idea in detail..."
                        rows="8"
                        required
                    ></textarea>
                </div>

                <div class="form-group">
                    <label class="form-label" for="openness">"Openness"</label>
                    <select node_ref=openness_ref class="form-select" id="openness">
                        <option value="open">"Open — anyone can contribute"</option>
                        <option value="collaborative">"Collaborative — team-based"</option>
                        <option value="commercial">"Commercial — IP protected"</option>
                    </select>
                </div>

                <div class="form-group">
                    <label class="form-label" for="category">"Category"</label>
                    <select node_ref=category_ref class="form-select" id="category">
                        <option value="">"No category"</option>
                        <Suspense fallback=|| ()>
                            {move || {
                                categories.get().map(|cats| {
                                    cats.iter().map(|c| {
                                        view! {
                                            <option value={c.id.clone()}>{c.name.clone()}</option>
                                        }
                                    }).collect::<Vec<_>>()
                                })
                            }}
                        </Suspense>
                    </select>
                </div>

                <button
                    class="btn btn-primary btn-lg"
                    style="width: 100%"
                    type="submit"
                    disabled=move || loading.get()
                >
                    {move || if loading.get() { "Forging..." } else { "Bring to the Forge" }}
                </button>
            </form>
        </div>
    }
}
