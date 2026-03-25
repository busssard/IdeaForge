use leptos::prelude::*;
use leptos_router::components::A;

use crate::api;
use crate::api::types::PublicUserResponse;
use crate::components::loading::Loading;

#[component]
pub fn PeoplePage() -> impl IntoView {
    // Mark that the user has visited the people page (for onboarding checklist)
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item("ideaforge_visited_people", "true");
    }

    let role_filter = RwSignal::new(String::new());
    let sort_filter = RwSignal::new("recently_joined".to_string());
    let page = RwSignal::new(1u64);

    let users = LocalResource::new(move || {
        let role = role_filter.get();
        let sort = sort_filter.get();
        let p = page.get();
        async move {
            let role_opt = if role.is_empty() { None } else { Some(role.as_str()) };
            let sort_opt = Some(sort.as_str());
            api::users::list_users(p, 20, role_opt, None, sort_opt).await
        }
    });

    view! {
        <div class="page">
            <div class="page-header">
                <h1 class="page-title">"Discover People"</h1>
                <p class="page-subtitle">"Find collaborators, makers, and entrepreneurs to build with"</p>
            </div>

            <div class="browse-filters">
                <div class="filter-group">
                    <label class="form-label">"Role"</label>
                    <select
                        class="form-select"
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            role_filter.set(val);
                            page.set(1);
                        }
                    >
                        <option value="">"All roles"</option>
                        <option value="entrepreneur">"Entrepreneurs"</option>
                        <option value="maker">"Makers"</option>
                        <option value="curious">"Curious Explorers"</option>
                    </select>
                </div>

                <div class="filter-group">
                    <label class="form-label">"Sort by"</label>
                    <select
                        class="form-select"
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            sort_filter.set(val);
                            page.set(1);
                        }
                    >
                        <option value="recently_joined">"Recently Joined"</option>
                        <option value="most_active">"Most Active"</option>
                    </select>
                </div>
            </div>

            <Suspense fallback=move || view! { <Loading /> }>
                {move || {
                    users.get().map(|result| {
                        match &*result {
                            Ok(resp) => {
                                if resp.data.is_empty() {
                                    view! {
                                        <div class="empty-state">
                                            <h3>"No people found"</h3>
                                            <p>"Try adjusting your filters."</p>
                                        </div>
                                    }.into_any()
                                } else {
                                    let items: Vec<PublicUserResponse> = resp.data.clone();
                                    let total_pages = resp.meta.total_pages;
                                    let current_page = resp.meta.page;
                                    view! {
                                        <div class="people-grid">
                                            {items.into_iter().map(|user| {
                                                view! { <UserCard user=user /> }
                                            }).collect::<Vec<_>>()}
                                        </div>

                                        {(total_pages > 1).then(|| {
                                            view! {
                                                <div class="pagination">
                                                    <button
                                                        class="btn btn-secondary"
                                                        disabled=move || current_page <= 1
                                                        on:click=move |_| page.set(page.get_untracked().saturating_sub(1).max(1))
                                                    >
                                                        "Previous"
                                                    </button>
                                                    <span class="pagination-info">
                                                        {format!("Page {} of {}", current_page, total_pages)}
                                                    </span>
                                                    <button
                                                        class="btn btn-secondary"
                                                        disabled=move || current_page >= total_pages
                                                        on:click=move |_| page.set(page.get_untracked() + 1)
                                                    >
                                                        "Next"
                                                    </button>
                                                </div>
                                            }
                                        })}
                                    }.into_any()
                                }
                            }
                            Err(e) => {
                                view! {
                                    <div class="error-display">
                                        <p>{e.message.clone()}</p>
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

#[component]
fn UserCard(user: PublicUserResponse) -> impl IntoView {
    let id = user.id.clone();
    let display_name = user.display_name.clone();
    let bio = if user.bio.len() > 150 {
        format!("{}...", &user.bio[..147])
    } else {
        user.bio.clone()
    };
    let role = user.role.clone();
    let role_label = match role.as_str() {
        "entrepreneur" => "Entrepreneur".to_string(),
        "maker" => "Maker".to_string(),
        "curious" => "Explorer".to_string(),
        "admin" => "Admin".to_string(),
        _ => role.clone(),
    };
    let skills: Vec<String> = user.skills.as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let looking_for = user.looking_for.clone();
    let idea_count = user.idea_count;
    let stoke_count = user.stoke_count;

    let role_class = format!("user-card-role role-{}", role);

    view! {
        <A href=format!("/profile/{id}") attr:class="user-card card card-clickable fade-in" attr:style="text-decoration: none; color: inherit; display: block;">
            <div class="user-card-header">
                <div class="user-card-avatar">
                    {display_name.chars().next().unwrap_or('?').to_uppercase().to_string()}
                </div>
                <div class="user-card-info">
                    <h3 class="user-card-name">{display_name}</h3>
                    <span class=role_class>{role_label}</span>
                </div>
            </div>

            {(!bio.is_empty()).then(|| view! {
                <p class="user-card-bio">{bio}</p>
            })}

            {(!skills.is_empty()).then(|| {
                let skill_tags: Vec<String> = skills.into_iter().take(5).collect();
                view! {
                    <div class="user-card-skills">
                        {skill_tags.into_iter().map(|skill| {
                            view! { <span class="skill-tag">{skill}</span> }
                        }).collect::<Vec<_>>()}
                    </div>
                }
            })}

            {looking_for.map(|lf| view! {
                <p class="user-card-looking-for">
                    <span class="looking-for-label">"Looking for: "</span>
                    {lf}
                </p>
            })}

            <div class="user-card-stats">
                <span class="user-stat" title="Ideas created">{format!("{} ideas", idea_count)}</span>
                <span class="user-stat" title="Stokes given">{format!("{} stokes", stoke_count)}</span>
            </div>
        </A>
    }
}
