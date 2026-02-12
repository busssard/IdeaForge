use leptos::prelude::*;

use crate::api;
use crate::api::types::UpdateMeRequest;
use crate::components::loading::Loading;
use crate::components::protected::Protected;
use crate::state::auth::AuthState;

#[component]
pub fn SettingsPage() -> impl IntoView {
    view! {
        <Protected>
            <SettingsContent />
        </Protected>
    }
}

#[component]
fn SettingsContent() -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let error = RwSignal::new(String::new());
    let success = RwSignal::new(String::new());
    let loading = RwSignal::new(false);

    // Load current user data
    let user_data = LocalResource::new(move || async move {
        api::users::get_me().await
    });

    // NodeRefs for form inputs
    let name_ref = NodeRef::<leptos::html::Input>::new();
    let bio_ref = NodeRef::<leptos::html::Textarea>::new();

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        error.set(String::new());
        success.set(String::new());

        let display_name = name_ref.get().map(|el| el.value()).unwrap_or_default();
        let bio = bio_ref.get().map(|el| el.value()).unwrap_or_default();

        if display_name.trim().is_empty() {
            error.set("Display name cannot be empty.".to_string());
            return;
        }

        if bio.len() > 2000 {
            error.set("Bio must be 2000 characters or less.".to_string());
            return;
        }

        loading.set(true);

        wasm_bindgen_futures::spawn_local(async move {
            let req = UpdateMeRequest {
                display_name: Some(display_name),
                bio: Some(bio),
            };

            match api::users::update_me(req).await {
                Ok(user) => {
                    auth.set_authenticated(&user);
                    success.set("Profile updated successfully.".to_string());
                }
                Err(e) => {
                    error.set(e.message);
                }
            }

            loading.set(false);
        });
    };

    view! {
        <div class="page settings">
            <div class="settings-container">
                <h1 class="page-title mb-lg">"Account Settings"</h1>

                <Suspense fallback=move || view! { <Loading /> }>
                    {move || {
                        user_data.get().map(|result| {
                            match &*result {
                                Ok(user) => {
                                    let display_name = user.display_name.clone();
                                    let bio = user.bio.clone();
                                    let email = user.email.clone();
                                    let role = user.role.clone();

                                    view! {
                                        <div class="card">
                                            <form on:submit=on_submit>
                                                // Error display
                                                {move || {
                                                    let err = error.get();
                                                    if err.is_empty() {
                                                        view! { <div></div> }.into_any()
                                                    } else {
                                                        view! {
                                                            <div class="alert alert-error mb-md">
                                                                <p>{err}</p>
                                                            </div>
                                                        }.into_any()
                                                    }
                                                }}

                                                // Success display
                                                {move || {
                                                    let msg = success.get();
                                                    if msg.is_empty() {
                                                        view! { <div></div> }.into_any()
                                                    } else {
                                                        view! {
                                                            <div class="alert alert-success mb-md">
                                                                <p>{msg}</p>
                                                            </div>
                                                        }.into_any()
                                                    }
                                                }}

                                                <div class="form-group">
                                                    <label class="form-label" for="email">"Email"</label>
                                                    <input
                                                        type="email"
                                                        id="email"
                                                        class="form-input"
                                                        value=email
                                                        disabled=true
                                                    />
                                                    <span class="form-hint">"Email cannot be changed."</span>
                                                </div>

                                                <div class="form-group">
                                                    <label class="form-label" for="role">"Role"</label>
                                                    <input
                                                        type="text"
                                                        id="role"
                                                        class="form-input"
                                                        value=role
                                                        disabled=true
                                                    />
                                                </div>

                                                <div class="form-group">
                                                    <label class="form-label" for="display_name">"Display Name"</label>
                                                    <input
                                                        type="text"
                                                        id="display_name"
                                                        class="form-input"
                                                        node_ref=name_ref
                                                        value=display_name
                                                        required=true
                                                        maxlength="100"
                                                    />
                                                </div>

                                                <div class="form-group">
                                                    <label class="form-label" for="bio">"Bio"</label>
                                                    <textarea
                                                        id="bio"
                                                        class="form-input"
                                                        node_ref=bio_ref
                                                        rows="5"
                                                        maxlength="2000"
                                                        placeholder="Tell others about yourself..."
                                                    >{bio}</textarea>
                                                    <span class="form-hint">"Maximum 2000 characters."</span>
                                                </div>

                                                <button
                                                    type="submit"
                                                    class="btn btn-primary"
                                                    disabled=move || loading.get()
                                                >
                                                    {move || if loading.get() { "Saving..." } else { "Save Changes" }}
                                                </button>
                                            </form>
                                        </div>
                                    }.into_any()
                                }
                                Err(e) => {
                                    let msg = e.message.clone();
                                    view! {
                                        <div class="card">
                                            <div class="error-display">
                                                <p>{msg}</p>
                                            </div>
                                        </div>
                                    }.into_any()
                                }
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
