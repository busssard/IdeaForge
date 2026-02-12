use leptos::prelude::*;

#[component]
pub fn ErrorDisplay(#[prop(into)] message: String) -> impl IntoView {
    view! {
        <div class="error-display">
            <h3>"Something went wrong"</h3>
            <p>{message}</p>
        </div>
    }
}
