use leptos::prelude::*;

use crate::api;
use crate::api::types::UpdateMeRequest;
use crate::components::avatar_uploader::AvatarUploader;
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
    let user_data = LocalResource::new(move || async move { api::users::get_me().await });

    // NodeRefs for form inputs
    let name_ref = NodeRef::<leptos::html::Input>::new();
    let bio_ref = NodeRef::<leptos::html::Textarea>::new();
    let skills_ref = NodeRef::<leptos::html::Input>::new();
    let looking_for_ref = NodeRef::<leptos::html::Input>::new();
    let availability_ref = NodeRef::<leptos::html::Select>::new();
    let role_ref = NodeRef::<leptos::html::Select>::new();
    let loc1_ref = NodeRef::<leptos::html::Input>::new();
    let loc2_ref = NodeRef::<leptos::html::Input>::new();
    let loc3_ref = NodeRef::<leptos::html::Input>::new();
    let education_ref = NodeRef::<leptos::html::Input>::new();

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        error.set(String::new());
        success.set(String::new());

        let display_name = name_ref.get().map(|el| el.value()).unwrap_or_default();
        let bio = bio_ref.get().map(|el| el.value()).unwrap_or_default();
        let skills_str = skills_ref.get().map(|el| el.value()).unwrap_or_default();
        let looking_for = looking_for_ref
            .get()
            .map(|el| el.value())
            .unwrap_or_default();
        let availability = availability_ref
            .get()
            .map(|el| {
                let el: &web_sys::HtmlSelectElement = &el;
                el.value()
            })
            .unwrap_or_default();
        let role_val = role_ref
            .get()
            .map(|el| {
                let el: &web_sys::HtmlSelectElement = &el;
                el.value()
            })
            .unwrap_or_default();
        let loc1 = loc1_ref.get().map(|el| el.value()).unwrap_or_default();
        let loc2 = loc2_ref.get().map(|el| el.value()).unwrap_or_default();
        let loc3 = loc3_ref.get().map(|el| el.value()).unwrap_or_default();
        let locations_input: Vec<String> = [loc1, loc2, loc3]
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let education = education_ref.get().map(|el| el.value()).unwrap_or_default();

        if display_name.trim().is_empty() {
            error.set("Display name cannot be empty.".to_string());
            return;
        }

        if bio.len() > 2000 {
            error.set("Bio must be 2000 characters or less.".to_string());
            return;
        }

        // Parse skills from comma-separated string
        let skills: Vec<String> = skills_str
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        if skills.len() > 10 {
            error.set("Maximum 10 skills allowed.".to_string());
            return;
        }

        if locations_input.iter().any(|l| l.len() > 100) {
            error.set("Each location must be 100 characters or fewer.".to_string());
            return;
        }
        if education.len() > 100 {
            error.set("Education level must be 100 characters or fewer.".to_string());
            return;
        }

        loading.set(true);

        wasm_bindgen_futures::spawn_local(async move {
            let req = UpdateMeRequest {
                display_name: Some(display_name),
                bio: Some(bio),
                skills: Some(skills),
                looking_for: if looking_for.is_empty() {
                    Some(None)
                } else {
                    Some(Some(looking_for))
                },
                availability: if availability.is_empty() {
                    None
                } else {
                    Some(availability)
                },
                role: if role_val.is_empty() { None } else { Some(role_val) },
                locations: Some(locations_input),
                education_level: Some(if education.trim().is_empty() {
                    None
                } else {
                    Some(education.trim().to_string())
                }),
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
                                    let avatar_url = user.avatar_url.clone();
                                    let initial_letter = display_name
                                        .chars()
                                        .next()
                                        .unwrap_or('?')
                                        .to_uppercase()
                                        .to_string();
                                    let skills_str = user.skills.as_array()
                                        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
                                        .unwrap_or_default();
                                    let looking_for = user.looking_for.clone().unwrap_or_default();
                                    let availability = user.availability.clone().unwrap_or_default();
                                    let locations_list: Vec<String> = user.locations.as_array()
                                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                        .unwrap_or_default();
                                    let loc1_val = locations_list.first().cloned().unwrap_or_default();
                                    let loc2_val = locations_list.get(1).cloned().unwrap_or_default();
                                    let loc3_val = locations_list.get(2).cloned().unwrap_or_default();
                                    let education_val = user.education_level.clone().unwrap_or_default();

                                    view! {
                                        <div class="card mb-lg">
                                            <h3>"Profile photo"</h3>
                                            <AvatarUploader
                                                initial_url=avatar_url
                                                initial_letter=initial_letter
                                                on_uploaded=Callback::new(move |_new_url: String| {
                                                    wasm_bindgen_futures::spawn_local(async move {
                                                        auth.load_user().await;
                                                    });
                                                })
                                            />
                                        </div>
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
                                                    <select
                                                        id="role"
                                                        class="form-select"
                                                        node_ref=role_ref
                                                    >
                                                        <option value="entrepreneur" selected={role == "entrepreneur"}>"Entrepreneur"</option>
                                                        <option value="maker" selected={role == "maker"}>"Maker"</option>
                                                        <option value="curious" selected={role == "curious"}>"Curious"</option>
                                                    </select>
                                                    <span class="form-hint">"What best describes why you're on IdeaForge? You can change this any time."</span>
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

                                                <div class="form-group">
                                                    <label class="form-label" for="skills">"Skills"</label>
                                                    <input
                                                        type="text"
                                                        id="skills"
                                                        class="form-input"
                                                        node_ref=skills_ref
                                                        value=skills_str
                                                        placeholder="rust, frontend, marketing, 3d-printing..."
                                                    />
                                                    <span class="form-hint">"Comma-separated, max 10 skills. Helps others discover you."</span>
                                                </div>

                                                <div class="form-group">
                                                    <label class="form-label" for="looking_for">"Looking for"</label>
                                                    <input
                                                        type="text"
                                                        id="looking_for"
                                                        class="form-input"
                                                        node_ref=looking_for_ref
                                                        value=looking_for
                                                        placeholder="Co-founder, designer, marketing help..."
                                                        maxlength="500"
                                                    />
                                                    <span class="form-hint">"What kind of collaboration are you seeking?"</span>
                                                </div>

                                                <div class="form-group">
                                                    <label class="form-label" for="availability">"Availability"</label>
                                                    <select
                                                        id="availability"
                                                        class="form-select"
                                                        node_ref=availability_ref
                                                    >
                                                        <option value="" selected=availability.is_empty()>"Not specified"</option>
                                                        <option value="full-time" selected={availability == "full-time"}>"Full-time"</option>
                                                        <option value="part-time" selected={availability == "part-time"}>"Part-time"</option>
                                                        <option value="weekends" selected={availability == "weekends"}>"Weekends only"</option>
                                                        <option value="few-hours" selected={availability == "few-hours"}>"A few hours/week"</option>
                                                        <option value="open-to-chat" selected={availability == "open-to-chat"}>"Open to chat"</option>
                                                    </select>
                                                </div>

                                                <div class="form-group">
                                                    <label class="form-label">"Based in"</label>
                                                    <input
                                                        type="text"
                                                        class="form-input"
                                                        node_ref=loc1_ref
                                                        value=loc1_val
                                                        placeholder="Berlin, Germany"
                                                        maxlength="100"
                                                    />
                                                    <input
                                                        type="text"
                                                        class="form-input mt-sm"
                                                        node_ref=loc2_ref
                                                        value=loc2_val
                                                        placeholder="Remote • EU timezone"
                                                        maxlength="100"
                                                    />
                                                    <input
                                                        type="text"
                                                        class="form-input mt-sm"
                                                        node_ref=loc3_ref
                                                        value=loc3_val
                                                        placeholder="A third place, if you'd like"
                                                        maxlength="100"
                                                    />
                                                    <span class="form-hint">"Up to 3 locations — city, country, timezone, or anything that describes where you are. Empty fields are ignored."</span>
                                                </div>

                                                <div class="form-group">
                                                    <label class="form-label" for="education">"Education level"</label>
                                                    <input
                                                        type="text"
                                                        id="education"
                                                        class="form-input"
                                                        node_ref=education_ref
                                                        value=education_val
                                                        placeholder="e.g. BSc Computer Science, Self-taught, PhD Biology"
                                                        maxlength="100"
                                                    />
                                                    <span class="form-hint">"Free-text — whatever's meaningful to you."</span>
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
