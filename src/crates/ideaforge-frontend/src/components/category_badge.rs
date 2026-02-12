use leptos::prelude::*;

#[component]
pub fn CategoryBadge(#[prop(into)] name: String) -> impl IntoView {
    view! {
        <span class="badge badge-category">{name}</span>
    }
}
