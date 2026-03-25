pub mod api;
pub mod components;
pub mod pages;
pub mod state;

use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use wasm_bindgen::prelude::wasm_bindgen;

use components::layout::navbar::Navbar;
use components::layout::footer::Footer;
use components::onboarding_checklist::OnboardingChecklist;
use pages::auth::LoginPage;
use pages::auth::RegisterPage;
use pages::browse::BrowsePage;
use pages::create_idea::CreateIdeaPage;
use pages::dashboard::DashboardPage;
use pages::home::HomePage;
use pages::idea_detail::IdeaDetailPage;
use pages::notifications::NotificationsPage;
use pages::people::PeoplePage;
use pages::profile::ProfilePage;
use pages::settings::SettingsPage;
use state::auth::AuthState;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[component]
pub fn App() -> impl IntoView {
    // Provide global auth state
    let auth = AuthState::new();
    provide_context(auth);

    view! {
        <Router>
            <Navbar />
            <OnboardingChecklist />
            <main class="main-content">
                <Routes fallback=|| view! { <p>"404 — Page not found"</p> }.into_view()>
                    <Route path=path!("/") view=HomePage />
                    <Route path=path!("/browse") view=BrowsePage />
                    <Route path=path!("/people") view=PeoplePage />
                    <Route path=path!("/login") view=LoginPage />
                    <Route path=path!("/register") view=RegisterPage />
                    <Route path=path!("/ideas/new") view=CreateIdeaPage />
                    <Route path=path!("/ideas/:id") view=IdeaDetailPage />
                    <Route path=path!("/dashboard") view=DashboardPage />
                    <Route path=path!("/profile/:id") view=ProfilePage />
                    <Route path=path!("/settings") view=SettingsPage />
                    <Route path=path!("/notifications") view=NotificationsPage />
                </Routes>
            </main>
            <Footer />
        </Router>
    }
}
