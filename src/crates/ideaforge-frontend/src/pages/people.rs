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
    let skills_filter = RwSignal::new(String::new());
    let sort_field = RwSignal::new("joined".to_string()); // joined | name
    let sort_dir = RwSignal::new("desc".to_string());
    let page = RwSignal::new(1u64);

    let sort_token = Memo::new(move |_| {
        match sort_field.get().as_str() {
            "joined" => if sort_dir.get() == "asc" { "oldest" } else { "recently_joined" },
            "name" => if sort_dir.get() == "asc" { "name_asc" } else { "name_desc" },
            // Aggregate sorts only make sense desc ("most X") — the UI hides the
            // direction toggle for these via `aggregate_active`.
            "ideas" => "most_ideas",
            "stokes" => "most_stokes",
            "active" => "most_active",
            _ => "recently_joined",
        }.to_string()
    });

    let aggregate_active = Memo::new(move |_| {
        matches!(sort_field.get().as_str(), "ideas" | "stokes" | "active")
    });

    let users = LocalResource::new(move || {
        let role = role_filter.get();
        let skills = skills_filter.get();
        let sort = sort_token.get();
        let p = page.get();
        async move {
            let role_opt = if role.is_empty() { None } else { Some(role.as_str()) };
            let skills_opt = if skills.is_empty() { None } else { Some(skills.as_str()) };
            api::users::list_users(p, 20, role_opt, skills_opt, Some(sort.as_str())).await
        }
    });

    let toggle_sort_dir = move |_: web_sys::MouseEvent| {
        sort_dir.update(|d| *d = if d == "desc" { "asc".to_string() } else { "desc".to_string() });
        page.set(1);
    };

    let any_filter_active = Memo::new(move |_| {
        !role_filter.get().is_empty()
            || !skills_filter.get().is_empty()
            || sort_field.get() != "joined"
            || sort_dir.get() != "desc"
    });

    let clear_filters = move |_: web_sys::MouseEvent| {
        role_filter.set(String::new());
        skills_filter.set(String::new());
        sort_field.set("joined".to_string());
        sort_dir.set("desc".to_string());
        page.set(1);
    };

    view! {
        <div class="page">
            <div class="page-header">
                <h1 class="page-title">"Discover People"</h1>
                <p class="page-subtitle">"Find collaborators, makers, and entrepreneurs to build with"</p>
            </div>

            <div class="listing-layout">
                <aside class="filter-sidebar">
                    <div class="filter-sidebar-header">
                        <h3 class="filter-sidebar-title">"Filters"</h3>
                        {move || any_filter_active.get().then(|| view! {
                            <button class="filter-clear-btn" on:click=clear_filters>
                                "Reset"
                            </button>
                        })}
                    </div>

                    <div class="filter-section">
                        <h4 class="filter-section-title">"Sort by"</h4>
                        <div class="sort-control">
                            <select
                                class="filter-select"
                                on:change=move |ev| {
                                    sort_field.set(event_target_value(&ev));
                                    page.set(1);
                                }
                            >
                                <option value="joined" selected=move || sort_field.get() == "joined">
                                    "Joined date"
                                </option>
                                <option value="name" selected=move || sort_field.get() == "name">
                                    "Name"
                                </option>
                                <option value="ideas" selected=move || sort_field.get() == "ideas">
                                    "Most ideas"
                                </option>
                                <option value="stokes" selected=move || sort_field.get() == "stokes">
                                    "Most sparks"
                                </option>
                                <option value="active" selected=move || sort_field.get() == "active">
                                    "Most active"
                                </option>
                            </select>
                            {move || (!aggregate_active.get()).then(|| view! {
                                <button
                                    type="button"
                                    class="sort-direction-btn"
                                    on:click=toggle_sort_dir
                                    title=move || if sort_dir.get() == "desc" { "Descending — click for ascending" } else { "Ascending — click for descending" }
                                    aria-label="Toggle sort direction"
                                >
                                    {move || if sort_dir.get() == "desc" { "\u{2193}" } else { "\u{2191}" }}
                                </button>
                            })}
                        </div>
                    </div>

                    <div class="filter-section">
                        <h4 class="filter-section-title">"Role"</h4>
                        <div class="filter-radio-group">
                            {[
                                ("", "Any"),
                                ("entrepreneur", "Entrepreneur"),
                                ("maker", "Maker"),
                                ("curious", "Curious"),
                            ].into_iter().map(|(value, label)| {
                                let value_s = value.to_string();
                                let value_check = value_s.clone();
                                view! {
                                    <label class="filter-radio">
                                        <input
                                            type="radio"
                                            name="role"
                                            value=value_s.clone()
                                            prop:checked=move || role_filter.get() == value_check
                                            on:change=move |_| {
                                                role_filter.set(value_s.clone());
                                                page.set(1);
                                            }
                                        />
                                        <span>{label}</span>
                                    </label>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>

                    <div class="filter-section">
                        <h4 class="filter-section-title">"Skills"</h4>
                        <input
                            type="text"
                            class="filter-select filter-select-full"
                            placeholder="rust, design, ..."
                            prop:value=move || skills_filter.get()
                            on:input=move |ev| {
                                skills_filter.set(event_target_value(&ev));
                                page.set(1);
                            }
                        />
                    </div>
                </aside>

                <section class="listing-content">
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
                </section>
            </div>
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
    let avatar_url = user.avatar_url.clone();

    let role_class = format!("user-card-role role-{}", role);
    let fallback_letter = display_name.chars().next().unwrap_or('?').to_uppercase().to_string();

    view! {
        <A href=format!("/profile/{id}") attr:class="user-card card card-clickable fade-in" attr:style="text-decoration: none; color: inherit; display: block;">
            <div class="user-card-header">
                <div class="user-card-avatar">
                    {match avatar_url {
                        Some(url) => view! {
                            <img src=url alt="" class="user-card-avatar-img" />
                        }.into_any(),
                        None => view! { <span>{fallback_letter}</span> }.into_any(),
                    }}
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
