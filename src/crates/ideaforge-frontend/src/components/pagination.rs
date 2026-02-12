use leptos::prelude::*;

#[component]
pub fn Pagination(
    current_page: u64,
    total_pages: u64,
    on_page_change: Callback<u64>,
) -> impl IntoView {
    if total_pages <= 1 {
        return view! { <div></div> }.into_any();
    }

    let pages: Vec<u64> = {
        let mut v = Vec::new();
        let start = current_page.saturating_sub(2).max(1);
        let end = (start + 4).min(total_pages);
        let start = if end == total_pages {
            end.saturating_sub(4).max(1)
        } else {
            start
        };
        for p in start..=end {
            v.push(p);
        }
        v
    };

    view! {
        <div class="pagination">
            <button
                class="pagination-btn"
                disabled=move || current_page <= 1
                on:click=move |_| on_page_change.run(current_page - 1)
            >
                "Prev"
            </button>
            {pages.into_iter().map(|p| {
                let is_active = p == current_page;
                view! {
                    <button
                        class=if is_active { "pagination-btn active" } else { "pagination-btn" }
                        on:click=move |_| on_page_change.run(p)
                    >
                        {p.to_string()}
                    </button>
                }
            }).collect::<Vec<_>>()}
            <button
                class="pagination-btn"
                disabled=move || current_page >= total_pages
                on:click=move |_| on_page_change.run(current_page + 1)
            >
                "Next"
            </button>
        </div>
    }.into_any()
}
