use leptos::prelude::*;

use crate::components::protected::Protected;

#[component]
pub fn SettingsPage() -> impl IntoView {
    view! {
        <Protected>
            <SettingsContent />
        </Protected>
    }
}

#[component]
fn SettingsContent() -> impl IntoView {
    view! {
        <div class="page settings">
            <div class="settings-container">
                <h1 class="page-title mb-lg">"Account Settings"</h1>
                <div class="card">
                    <p class="text-muted">"Settings management coming soon..."</p>
                </div>
            </div>
        </div>
    }
}
