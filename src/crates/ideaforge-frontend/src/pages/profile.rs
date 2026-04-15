use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api;
use crate::api::types::IdeaResponse;
use crate::components::idea_card::IdeaCard;
use crate::components::loading::Loading;
use crate::components::markdown::Markdown;
use crate::components::message_privately_button::MessagePrivatelyButton;

/// Which activity list is currently expanded under the stat row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActivityTab {
    None,
    Authored,
    Contributing,
    Stoked,
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let params = use_params_map();

    let user = LocalResource::new(move || {
        let id = params.get().get("id").unwrap_or_default();
        async move { api::users::get_user(&id).await }
    });

    // Lightbox visibility for the large avatar view.
    let lightbox_open = RwSignal::new(false);

    // Which activity list is expanded. Exactly one at a time; clicking the
    // same stat closes it. Each list lazy-loads on first open.
    let active_tab = RwSignal::new(ActivityTab::None);

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
                                let user_id = u.id.clone();
                                let display_name = u.display_name.clone();
                                let role = u.role.clone();
                                let bio = u.bio.clone();
                                let avatar_url = u.avatar_url.clone();
                                let member_since = u.created_at.split('T').next().unwrap_or("").to_string();
                                let skills: Vec<String> = u.skills.as_array()
                                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                    .unwrap_or_default();
                                let looking_for = u.looking_for.clone();
                                let availability = u.availability.clone();
                                let locations: Vec<String> = u.locations.as_array()
                                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                    .unwrap_or_default();
                                let education_level = u.education_level.clone();
                                let idea_count = u.idea_count;
                                let stoke_count = u.stoke_count;
                                let contribution_count = u.contribution_count;

                                let role_badge_class = format!("badge badge-{}", role.to_lowercase());
                                let lightbox_src = avatar_url.clone();

                                let toggle_tab = move |tab: ActivityTab| {
                                    if active_tab.get_untracked() == tab {
                                        active_tab.set(ActivityTab::None);
                                    } else {
                                        active_tab.set(tab);
                                    }
                                };

                                view! {
                                    <div class="card profile-card">
                                        <div class="profile-header">
                                            <button
                                                type="button"
                                                class="profile-avatar profile-avatar-clickable"
                                                title="Click to enlarge"
                                                on:click=move |_| {
                                                    if avatar_url.is_some() { lightbox_open.set(true); }
                                                }
                                            >
                                                {match avatar_url.clone() {
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
                                            </button>

                                            <div class="profile-info">
                                                <h2 class="profile-name">{display_name}</h2>
                                                <span class=role_badge_class>{role}</span>
                                                <p class="text-muted profile-meta">
                                                    "Member since " {member_since}
                                                </p>
                                            </div>
                                        </div>

                                        <div class="profile-stats">
                                            <button
                                                type="button"
                                                class=move || stat_btn_class(active_tab, ActivityTab::Authored)
                                                on:click=move |_| toggle_tab(ActivityTab::Authored)
                                            >
                                                <span class="profile-stat-value">{idea_count}</span>
                                                <span class="profile-stat-label">"Ideas"</span>
                                            </button>
                                            <button
                                                type="button"
                                                class=move || stat_btn_class(active_tab, ActivityTab::Contributing)
                                                on:click=move |_| toggle_tab(ActivityTab::Contributing)
                                            >
                                                <span class="profile-stat-value">{contribution_count}</span>
                                                <span class="profile-stat-label">"Contributing"</span>
                                            </button>
                                            <button
                                                type="button"
                                                class=move || stat_btn_class(active_tab, ActivityTab::Stoked)
                                                on:click=move |_| toggle_tab(ActivityTab::Stoked)
                                            >
                                                <span class="profile-stat-value">{stoke_count}</span>
                                                <span class="profile-stat-label">"Sparked"</span>
                                            </button>
                                        </div>

                                        {
                                            let uid = user_id.clone();
                                            move || match active_tab.get() {
                                                ActivityTab::None => view! { <span></span> }.into_any(),
                                                ActivityTab::Authored => view! {
                                                    <ActivityList
                                                        user_id=uid.clone()
                                                        tab=ActivityTab::Authored
                                                    />
                                                }.into_any(),
                                                ActivityTab::Contributing => view! {
                                                    <ActivityList
                                                        user_id=uid.clone()
                                                        tab=ActivityTab::Contributing
                                                    />
                                                }.into_any(),
                                                ActivityTab::Stoked => view! {
                                                    <ActivityList
                                                        user_id=uid.clone()
                                                        tab=ActivityTab::Stoked
                                                    />
                                                }.into_any(),
                                            }
                                        }

                                        {if !bio.is_empty() {
                                            view! {
                                                <div class="profile-bio">
                                                    <h3>"About"</h3>
                                                    <Markdown content=bio />
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="profile-bio">
                                                    <p class="text-muted">"This user hasn't added a bio yet."</p>
                                                </div>
                                            }.into_any()
                                        }}

                                        {(!skills.is_empty()).then(|| {
                                            let skill_tags = skills.clone();
                                            view! {
                                                <div class="profile-section">
                                                    <h3>"Skills"</h3>
                                                    <div class="user-card-skills">
                                                        {skill_tags.into_iter().map(|skill| {
                                                            view! { <span class="skill-tag">{skill}</span> }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </div>
                                            }
                                        })}

                                        {(!locations.is_empty()).then(|| {
                                            let pills = locations.clone();
                                            view! {
                                                <div class="profile-section">
                                                    <h3>"Based in"</h3>
                                                    <div class="profile-location-list">
                                                        {pills.into_iter().map(|loc| view! {
                                                            <span class="profile-location-pill">{loc}</span>
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </div>
                                            }
                                        })}

                                        {education_level.clone().map(|edu| view! {
                                            <div class="profile-section">
                                                <h3>"Education"</h3>
                                                <p>{edu}</p>
                                            </div>
                                        })}

                                        {looking_for.map(|lf| view! {
                                            <div class="profile-section">
                                                <h3>"Looking for"</h3>
                                                <p>{lf}</p>
                                            </div>
                                        })}

                                        {availability.map(|av| view! {
                                            <div class="profile-section">
                                                <h3>"Availability"</h3>
                                                <p>{av}</p>
                                            </div>
                                        })}

                                        <div class="profile-actions mt-md">
                                            <MessagePrivatelyButton target_user_id=u.id.clone() />
                                            <A href="/people" attr:class="btn btn-secondary">"Discover People"</A>
                                            <A href="/browse" attr:class="btn btn-secondary">"Forge Floor"</A>
                                        </div>
                                    </div>

                                    {
                                        let src = lightbox_src.clone();
                                        move || {
                                            if !lightbox_open.get() { return view! { <span></span> }.into_any(); }
                                            let url = match src.clone() {
                                                Some(u) if !u.is_empty() => u,
                                                _ => return view! { <span></span> }.into_any(),
                                            };
                                            view! {
                                                <div
                                                    class="avatar-lightbox"
                                                    role="dialog"
                                                    aria-modal="true"
                                                    on:click=move |_| lightbox_open.set(false)
                                                >
                                                    <img src=url class="avatar-lightbox-img" alt="Profile photo full size" />
                                                    <button
                                                        class="avatar-lightbox-close"
                                                        aria-label="Close"
                                                        on:click=move |ev: web_sys::MouseEvent| {
                                                            ev.stop_propagation();
                                                            lightbox_open.set(false);
                                                        }
                                                    >"\u{2715}"</button>
                                                </div>
                                            }.into_any()
                                        }
                                    }
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

fn stat_btn_class(active: RwSignal<ActivityTab>, tab: ActivityTab) -> String {
    if active.get() == tab {
        "profile-stat profile-stat-active".into()
    } else {
        "profile-stat".into()
    }
}

#[component]
fn ActivityList(user_id: String, tab: ActivityTab) -> impl IntoView {
    let uid = StoredValue::new(user_id);
    let list = LocalResource::new(move || {
        let id = uid.get_value();
        async move {
            match tab {
                ActivityTab::Authored => api::users::get_user_authored_ideas(&id, 1, 20).await,
                ActivityTab::Contributing => api::users::get_user_contributions(&id, 1, 20).await,
                ActivityTab::Stoked => api::users::get_user_stoked_ideas(&id, 1, 20).await,
                ActivityTab::None => unreachable!(),
            }
        }
    });

    let empty_label = match tab {
        ActivityTab::Authored => "Nothing authored yet.",
        ActivityTab::Contributing => "Not on any idea's team yet.",
        ActivityTab::Stoked => "No sparks given yet.",
        ActivityTab::None => "",
    };

    view! {
        <div class="profile-activity">
            <Suspense fallback=move || view! { <Loading /> }>
                {move || list.get().map(|r| match &*r {
                    Ok(resp) => {
                        if resp.data.is_empty() {
                            view! { <p class="text-muted">{empty_label}</p> }.into_any()
                        } else {
                            let items: Vec<IdeaResponse> = resp.data.clone();
                            view! {
                                <div class="profile-activity-grid">
                                    {items.into_iter().map(|i| view! {
                                        <IdeaCard idea=i />
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }
                    Err(e) => view! {
                        <p class="form-error">{e.message.clone()}</p>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
