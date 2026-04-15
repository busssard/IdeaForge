use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlSelectElement;

use crate::api;
use crate::api::types::IdeaResponse;
use crate::components::idea_card::IdeaCard;
use crate::components::loading::Loading;
use crate::components::pagination::Pagination;

#[component]
pub fn BrowsePage() -> impl IntoView {
    let page = RwSignal::new(1u64);
    let category_filter = RwSignal::new(String::new());
    let maturity_filter = RwSignal::new(String::new());
    let openness_filter = RwSignal::new(String::new());
    let lifecycle_filter = RwSignal::new(String::new());
    // sort_field: created | sparks | title.   sort_dir: desc | asc.
    let sort_field = RwSignal::new("created".to_string());
    let sort_dir = RwSignal::new("desc".to_string());
    let per_page = 12u64;

    // Build the API sort token from field + direction.
    let sort_token = Memo::new(move |_| {
        match (sort_field.get().as_str(), sort_dir.get().as_str()) {
            ("created", "asc") => "oldest",
            ("created", _) => "recent",
            ("sparks", "asc") => "sparks_asc",
            ("sparks", _) => "sparks_desc",
            ("title", "asc") => "title_asc",
            ("title", _) => "title_desc",
            ("members", "asc") => "members_asc",
            ("members", _) => "members_desc",
            ("comments", "asc") => "comments_asc",
            ("comments", _) => "comments_desc",
            _ => "recent",
        }.to_string()
    });

    // Load categories for filter dropdown
    let categories = LocalResource::new(move || async move {
        api::categories::list_categories().await.unwrap_or_default()
    });

    // Load ideas reactively based on filters
    let ideas = LocalResource::new(move || {
        let p = page.get();
        let cat = category_filter.get();
        let mat = maturity_filter.get();
        let opn = openness_filter.get();
        let lc = lifecycle_filter.get();
        let srt = sort_token.get();
        async move {
            api::ideas::list_ideas_v2(
                p,
                per_page,
                if cat.is_empty() { None } else { Some(cat.as_str()) },
                if mat.is_empty() { None } else { Some(mat.as_str()) },
                if opn.is_empty() { None } else { Some(opn.as_str()) },
                if lc.is_empty() { None } else { Some(lc.as_str()) },
                Some(srt.as_str()),
                None,
            )
            .await
        }
    });

    let on_page_change = Callback::new(move |p: u64| {
        page.set(p);
    });

    let on_category_change = move |ev: web_sys::Event| {
        let target: HtmlSelectElement = event_target(&ev);
        category_filter.set(target.value());
        page.set(1);
    };

    let on_sort_change = move |ev: web_sys::Event| {
        let target: HtmlSelectElement = event_target(&ev);
        sort_field.set(target.value());
        page.set(1);
    };

    let toggle_sort_dir = move |_: web_sys::MouseEvent| {
        sort_dir.update(|d| *d = if d == "desc" { "asc".to_string() } else { "desc".to_string() });
        page.set(1);
    };

    let clear_filters = move |_: web_sys::MouseEvent| {
        category_filter.set(String::new());
        maturity_filter.set(String::new());
        openness_filter.set(String::new());
        lifecycle_filter.set(String::new());
        sort_field.set("created".to_string());
        sort_dir.set("desc".to_string());
        page.set(1);
    };

    let any_filter_active = Memo::new(move |_| {
        !category_filter.get().is_empty()
            || !maturity_filter.get().is_empty()
            || !openness_filter.get().is_empty()
            || !lifecycle_filter.get().is_empty()
            || sort_field.get() != "created"
            || sort_dir.get() != "desc"
    });

    view! {
        <div class="page-header">
            <h1 class="page-title">"The Forge Floor"</h1>
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
                        <select class="filter-select" on:change=on_sort_change>
                            <option value="created" selected=move || sort_field.get() == "created">
                                "Newest"
                            </option>
                            <option value="sparks" selected=move || sort_field.get() == "sparks">
                                "Sparks"
                            </option>
                            <option value="title" selected=move || sort_field.get() == "title">
                                "Title"
                            </option>
                            <option value="members" selected=move || sort_field.get() == "members">
                                "Team size"
                            </option>
                            <option value="comments" selected=move || sort_field.get() == "comments">
                                "Comments"
                            </option>
                        </select>
                        <button
                            type="button"
                            class="sort-direction-btn"
                            on:click=toggle_sort_dir
                            title=move || if sort_dir.get() == "desc" { "Descending — click for ascending" } else { "Ascending — click for descending" }
                            aria-label="Toggle sort direction"
                        >
                            {move || if sort_dir.get() == "desc" { "\u{2193}" } else { "\u{2191}" }}
                        </button>
                    </div>
                </div>

                <div class="filter-section">
                    <h4 class="filter-section-title">"Category"</h4>
                    <select class="filter-select filter-select-full" on:change=on_category_change>
                        <option value="">"All categories"</option>
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

                <div class="filter-section">
                    <h4 class="filter-section-title">"Maturity"</h4>
                    <RadioGroup
                        name="maturity".to_string()
                        selected=maturity_filter
                        on_change=Callback::new(move |v: String| {
                            maturity_filter.set(v);
                            page.set(1);
                        })
                        options=vec![
                            ("".into(), "Any".into()),
                            ("spark".into(), "Spark".into()),
                            ("half_baked".into(), "Half-Baked".into()),
                            ("thought_through".into(), "Thought Through".into()),
                            ("serious".into(), "Serious Proposal".into()),
                            ("in_work".into(), "In Work".into()),
                        ]
                    />
                </div>

                <div class="filter-section">
                    <h4 class="filter-section-title">"Openness"</h4>
                    <RadioGroup
                        name="openness".to_string()
                        selected=openness_filter
                        on_change=Callback::new(move |v: String| {
                            openness_filter.set(v);
                            page.set(1);
                        })
                        options=vec![
                            ("".into(), "Any".into()),
                            ("open".into(), "Open".into()),
                            ("collaborative".into(), "Collaborative".into()),
                            ("commercial".into(), "Commercial".into()),
                            ("nda_protected".into(), "NDA-protected".into()),
                        ]
                    />
                </div>

                <div class="filter-section">
                    <h4 class="filter-section-title">"Status"</h4>
                    <RadioGroup
                        name="lifecycle".to_string()
                        selected=lifecycle_filter
                        on_change=Callback::new(move |v: String| {
                            lifecycle_filter.set(v);
                            page.set(1);
                        })
                        options=vec![
                            ("".into(), "Any".into()),
                            ("not_started".into(), "Not started".into()),
                            ("ongoing".into(), "Ongoing".into()),
                            ("finished".into(), "Finished".into()),
                        ]
                    />
                </div>

            </aside>

            <section class="listing-content">
                <Suspense fallback=move || view! { <Loading /> }>
                    {move || {
                        ideas.get().map(|result| {
                            match &*result {
                                Ok(resp) => {
                                    let current_page = resp.meta.page;
                                    let total_pages = resp.meta.total_pages;
                                    let total = resp.meta.total;
                                    let items: Vec<IdeaResponse> = resp.data.clone();

                                    if items.is_empty() {
                                        view! {
                                            <div class="empty-state">
                                                <h3>"No ideas found"</h3>
                                                <p>"Try adjusting your filters or bring a new idea to the forge."</p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <p class="text-muted mb-md">
                                                {format!("{total} idea{}", if total == 1 { "" } else { "s" })}
                                            </p>
                                            <div class="ideas-grid">
                                                {items.into_iter().map(|idea| {
                                                    view! { <IdeaCard idea=idea /> }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                            <Pagination
                                                current_page=current_page
                                                total_pages=total_pages
                                                on_page_change=on_page_change
                                            />
                                        }.into_any()
                                    }
                                }
                                Err(e) => {
                                    view! {
                                        <div class="error-display">
                                            <h3>"Failed to load ideas"</h3>
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
    }
}

#[component]
fn RadioGroup(
    name: String,
    selected: RwSignal<String>,
    #[prop(into)] on_change: Callback<String>,
    options: Vec<(String, String)>,
) -> impl IntoView {
    let name = StoredValue::new(name);
    view! {
        <div class="filter-radio-group">
            {options.into_iter().map(|(value, label)| {
                let group_name = name.get_value();
                let val = value.clone();
                let val_for_check = value.clone();
                view! {
                    <label class="filter-radio">
                        <input
                            type="radio"
                            name=group_name
                            value=value.clone()
                            prop:checked=move || selected.get() == val_for_check
                            on:change=move |_| on_change.run(val.clone())
                        />
                        <span>{label}</span>
                    </label>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

fn event_target<T: wasm_bindgen::JsCast>(event: &web_sys::Event) -> T {
    event
        .target()
        .expect("event target")
        .dyn_into::<T>()
        .expect("event target cast")
}
