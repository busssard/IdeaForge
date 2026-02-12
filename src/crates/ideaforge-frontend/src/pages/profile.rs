use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use leptos_router::components::A;

#[component]
pub fn ProfilePage() -> impl IntoView {
    let params = use_params_map();

    let user_id = move || {
        params.get().get("id").unwrap_or_default()
    };

    view! {
        <div class="page profile">
            <div class="page-header">
                <h1 class="page-title">"User Profile"</h1>
            </div>
            <div class="card">
                <p class="text-muted">"Profile for user: " {user_id}</p>
                <p>"Coming soon..."</p>
                <A href="/browse" attr:class="btn btn-secondary mt-md">"Back to Forge Floor"</A>
            </div>
        </div>
    }
}
