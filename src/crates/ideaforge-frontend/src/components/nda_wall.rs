use leptos::prelude::*;

use crate::api;

#[component]
pub fn NdaWall(
    idea_id: String,
    #[prop(into)] on_signed: Callback<()>,
) -> impl IntoView {
    let idea_id = StoredValue::new(idea_id);

    let signer_name = RwSignal::new(String::new());
    let agreed = RwSignal::new(false);
    let signing = RwSignal::new(false);
    let error = RwSignal::new(String::new());

    let template = LocalResource::new(move || {
        let id = idea_id.get_value();
        async move { api::nda::get_nda_template(&id).await }
    });

    let can_sign = move || {
        !signer_name.get().trim().is_empty() && agreed.get() && !signing.get()
    };

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if !can_sign() {
            return;
        }

        signing.set(true);
        error.set(String::new());

        let id = idea_id.get_value();
        let name = signer_name.get_untracked();

        wasm_bindgen_futures::spawn_local(async move {
            match api::nda::sign_nda(&id, &name).await {
                Ok(_) => {
                    signing.set(false);
                    on_signed.call(());
                }
                Err(e) => {
                    error.set(e.message);
                    signing.set(false);
                }
            }
        });
    };

    view! {
        <Suspense fallback=move || {
            view! {
                <div class="nda-wall">
                    <p>"Loading NDA..."</p>
                </div>
            }
        }>
            {move || {
                template.get().map(|result| {
                    match &*result {
                        Ok(tmpl) => {
                            let title = tmpl.title.clone();
                            let body = tmpl.body.clone();
                            let days = tmpl.confidentiality_period_days;
                            let jurisdiction = tmpl
                                .jurisdiction
                                .clone()
                                .unwrap_or_else(|| "Not specified".to_string());

                            view! {
                                <div class="nda-wall">
                                    <div class="nda-wall-header">
                                        <h3>"\u{1F512} NDA Required"</h3>
                                        <p>
                                            "This idea's full pitch is protected by a Non-Disclosure Agreement. "
                                            "You must review and sign the NDA below to view the details."
                                        </p>
                                    </div>

                                    <div class="nda-document">
                                        <h4>{title}</h4>
                                        <div class="nda-body">{body}</div>
                                    </div>

                                    <p class="nda-meta">
                                        "Confidentiality period: " {days.to_string()} " days"
                                        " \u{2022} Jurisdiction: " {jurisdiction}
                                    </p>

                                    <form class="nda-sign-form" on:submit=on_submit>
                                        {move || {
                                            let err = error.get();
                                            if err.is_empty() {
                                                view! { <span></span> }.into_any()
                                            } else {
                                                view! {
                                                    <div class="form-error">{err}</div>
                                                }
                                                    .into_any()
                                            }
                                        }}

                                        <input
                                            type="text"
                                            placeholder="Your legal name"
                                            prop:value=move || signer_name.get()
                                            on:input=move |ev| {
                                                signer_name
                                                    .set(event_target_value(&ev));
                                            }
                                        />

                                        <label>
                                            <input
                                                type="checkbox"
                                                prop:checked=move || agreed.get()
                                                on:change=move |ev| {
                                                    agreed
                                                        .set(checkbox_checked(&ev));
                                                }
                                            />
                                            "I have read and agree to the terms of this NDA"
                                        </label>

                                        <button
                                            type="submit"
                                            class="btn btn-primary"
                                            disabled=move || !can_sign()
                                        >
                                            {move || {
                                                if signing.get() { "Signing..." } else { "Sign NDA" }
                                            }}
                                        </button>
                                    </form>
                                </div>
                            }
                                .into_any()
                        }
                        Err(e) => {
                            view! {
                                <div class="nda-wall">
                                    <div class="error-display">
                                        <h3>"Failed to load NDA"</h3>
                                        <p>{e.message.clone()}</p>
                                    </div>
                                </div>
                            }
                                .into_any()
                        }
                    }
                })
            }}
        </Suspense>
    }
}

fn checkbox_checked(ev: &web_sys::Event) -> bool {
    use wasm_bindgen::JsCast;
    ev.target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|el| el.checked())
        .unwrap_or(false)
}
