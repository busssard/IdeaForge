use leptos::prelude::*;

#[component]
pub fn Loading() -> impl IntoView {
    view! {
        <div class="loading">
            <div class="spinner"></div>
        </div>
    }
}
