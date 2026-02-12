use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="page not-found">
            <div class="empty-state">
                <h1>"404"</h1>
                <h3>"Page Not Found"</h3>
                <p>"This page doesn't exist in the forge."</p>
                <A href="/" attr:class="btn btn-primary">"Back to Home"</A>
            </div>
        </div>
    }
}
