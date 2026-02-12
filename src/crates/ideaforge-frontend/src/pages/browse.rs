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
    let per_page = 12u64;

    // Load categories for filter dropdown
    let categories = LocalResource::new(move || async move {
        api::categories::list_categories().await.unwrap_or_default()
    });

    // Load ideas reactively based on filters
    let ideas = LocalResource::new(move || {
        let p = page.get();
        let cat = category_filter.get();
        let mat = maturity_filter.get();
        async move {
            api::ideas::list_ideas(
                p,
                per_page,
                if cat.is_empty() { None } else { Some(cat.as_str()) },
                if mat.is_empty() { None } else { Some(mat.as_str()) },
                None,
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

    let on_maturity_change = move |ev: web_sys::Event| {
        let target: HtmlSelectElement = event_target(&ev);
        maturity_filter.set(target.value());
        page.set(1);
    };

    view! {
        <div class="page-header">
            <h1 class="page-title">"The Forge Floor"</h1>
        </div>

        <div class="filters-bar">
            <select class="filter-select" on:change=on_category_change>
                <option value="">"All Categories"</option>
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

            <select class="filter-select" on:change=on_maturity_change>
                <option value="">"All Maturity"</option>
                <option value="spark">"Spark"</option>
                <option value="half_baked">"Half-Baked"</option>
                <option value="thought_through">"Thought Through"</option>
                <option value="serious">"Serious Proposal"</option>
                <option value="in_work">"In Work"</option>
            </select>
        </div>

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
    }
}

fn event_target<T: wasm_bindgen::JsCast>(event: &web_sys::Event) -> T {
    event
        .target()
        .expect("event target")
        .dyn_into::<T>()
        .expect("event target cast")
}
