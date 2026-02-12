use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api;
use crate::components::loading::Loading;

#[component]
pub fn ProfilePage() -> impl IntoView {
    let params = use_params_map();

    let user = LocalResource::new(move || {
        let id = params.get().get("id").unwrap_or_default();
        async move { api::users::get_user(&id).await }
    });

    view! {
        <div class="page profile">
            <div class="page-header">
                <h1 class="page-title">"User Profile"</h1>
            </div>

            <Suspense fallback=move || view! { <Loading /> }>
                {move || {
                    user.get().map(|result| {
                        match &*result {
                            Ok(u) => {
                                let display_name = u.display_name.clone();
                                let role = u.role.clone();
                                let bio = u.bio.clone();
                                let avatar_url = u.avatar_url.clone();
                                let member_since = u.created_at.split('T').next().unwrap_or("").to_string();

                                let role_badge_class = format!("badge badge-{}", role.to_lowercase());

                                view! {
                                    <div class="card profile-card">
                                        <div class="profile-header">
                                            <div class="profile-avatar">
                                                {match avatar_url {
                                                    Some(ref url) if !url.is_empty() => {
                                                        let url = url.clone();
                                                        view! {
                                                            <img src=url alt="Avatar" class="avatar-img" />
                                                        }.into_any()
                                                    }
                                                    _ => {
                                                        let initials = display_name
                                                            .split_whitespace()
                                                            .filter_map(|w| w.chars().next())
                                                            .take(2)
                                                            .collect::<String>()
                                                            .to_uppercase();
                                                        view! {
                                                            <div class="avatar-placeholder">
                                                                {initials}
                                                            </div>
                                                        }.into_any()
                                                    }
                                                }}
                                            </div>

                                            <div class="profile-info">
                                                <h2 class="profile-name">{display_name}</h2>
                                                <span class=role_badge_class>{role}</span>
                                                <p class="text-muted profile-meta">
                                                    "Member since " {member_since}
                                                </p>
                                            </div>
                                        </div>

                                        {if !bio.is_empty() {
                                            view! {
                                                <div class="profile-bio">
                                                    <h3>"About"</h3>
                                                    <p>{bio}</p>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="profile-bio">
                                                    <p class="text-muted">"This user hasn't added a bio yet."</p>
                                                </div>
                                            }.into_any()
                                        }}

                                        <div class="profile-actions mt-md">
                                            <A href="/browse" attr:class="btn btn-secondary">"Back to Forge Floor"</A>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                            Err(e) => {
                                let msg = e.message.clone();
                                view! {
                                    <div class="card">
                                        <div class="error-display">
                                            <h3>"Could not load profile"</h3>
                                            <p>{msg}</p>
                                            <A href="/browse" attr:class="btn btn-secondary mt-md">"Back to Forge Floor"</A>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
