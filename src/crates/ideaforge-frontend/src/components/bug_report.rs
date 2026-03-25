use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::api::client;

#[component]
pub fn BugReportButton() -> impl IntoView {
    let show_form = RwSignal::new(false);
    let submitting = RwSignal::new(false);
    let submitted = RwSignal::new(false);
    let error_msg = RwSignal::new(String::new());

    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();
    let severity_ref = NodeRef::<leptos::html::Select>::new();

    let toggle = move |_: web_sys::MouseEvent| {
        show_form.set(!show_form.get_untracked());
        submitted.set(false);
        error_msg.set(String::new());
    };

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if submitting.get_untracked() {
            return;
        }

        let description = textarea_ref
            .get()
            .map(|el| {
                let el: &web_sys::HtmlTextAreaElement = el.unchecked_ref();
                el.value()
            })
            .unwrap_or_default();

        if description.trim().is_empty() {
            return;
        }

        let severity = severity_ref
            .get()
            .map(|el| {
                let el: &web_sys::HtmlSelectElement = el.unchecked_ref();
                el.value()
            })
            .unwrap_or_else(|| "normal".to_string());

        let page_url = web_sys::window()
            .and_then(|w| w.location().href().ok())
            .unwrap_or_default();

        submitting.set(true);
        error_msg.set(String::new());

        wasm_bindgen_futures::spawn_local(async move {
            let body = serde_json::json!({
                "description": description.trim(),
                "page_url": page_url,
                "severity": severity,
            });

            match client::post::<serde_json::Value, serde_json::Value>(
                "/api/v1/bugs",
                &body,
            )
            .await
            {
                Ok(_) => {
                    submitted.set(true);
                    // Clear textarea
                    if let Some(el) = textarea_ref.get() {
                        let el: &web_sys::HtmlTextAreaElement = el.unchecked_ref();
                        el.set_value("");
                    }
                }
                Err(e) => {
                    error_msg.set(e.message);
                }
            }
            submitting.set(false);
        });
    };

    view! {
        // Floating bug report button
        <button
            class="bug-report-fab"
            on:click=toggle
            title="Report a bug"
        >
            {move || if show_form.get() { "\u{2715}" } else { "\u{1F41B}" }}
        </button>

        // Bug report form overlay
        <div
            class="bug-report-panel"
            style:display=move || if show_form.get() { "block" } else { "none" }
        >
            {move || {
                if submitted.get() {
                    view! {
                        <div class="bug-report-success">
                            <p>"Bug report saved. Thanks!"</p>
                            <button class="btn btn-sm btn-ghost" on:click=move |_| {
                                submitted.set(false);
                                show_form.set(false);
                            }>"Close"</button>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <form class="bug-report-form" on:submit=on_submit>
                            <h4>"Report a Bug"</h4>

                            {move || {
                                let err = error_msg.get();
                                if err.is_empty() {
                                    view! { <span></span> }.into_any()
                                } else {
                                    view! { <div class="form-error">{err}</div> }.into_any()
                                }
                            }}

                            <textarea
                                node_ref=textarea_ref
                                placeholder="What went wrong? What did you expect?"
                                rows="4"
                                required
                            ></textarea>

                            <select node_ref=severity_ref>
                                <option value="low">"Minor / cosmetic"</option>
                                <option value="normal" selected>"Normal"</option>
                                <option value="high">"Major / broken feature"</option>
                                <option value="critical">"Critical / can't use app"</option>
                            </select>

                            <button
                                class="btn btn-primary btn-sm"
                                type="submit"
                                disabled=move || submitting.get()
                            >
                                {move || if submitting.get() { "Sending..." } else { "Submit" }}
                            </button>
                        </form>
                    }.into_any()
                }
            }}
        </div>
    }
}
