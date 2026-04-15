pub mod api;
pub mod components;
pub mod mls;
pub mod pages;
pub mod state;

use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::hooks::use_location;
use leptos_router::path;
use wasm_bindgen::prelude::wasm_bindgen;

use components::bug_report::BugReportButton;
use components::layout::footer::Footer;
use components::layout::navbar::Navbar;
use components::onboarding_checklist::OnboardingChecklist;
use pages::about::AboutPage;
use pages::auth::LoginPage;
use pages::auth::RegisterPage;
use pages::browse::BrowsePage;
use pages::create_idea::CreateIdeaPage;
use pages::dashboard::DashboardPage;
use pages::home::HomePage;
use pages::how_it_works::HowItWorksPage;
use pages::idea_detail::IdeaDetailPage;
use pages::messages::MessagesPage;
use pages::notifications::NotificationsPage;
use pages::people::PeoplePage;
use pages::profile::ProfilePage;
use pages::settings::SettingsPage;
use state::auth::AuthState;
use state::mls_state::MlsState;

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

    // Provide MLS state. The client is only installed once the user unlocks
    // (or sets up) their keystore with their PIN — that happens in the
    // messages page's keystore gate, not here. On logout we clear state so
    // the next account can't see stale keys.
    let mls = MlsState::new();
    provide_context(mls);
    Effect::new(move |prev: Option<Option<String>>| {
        let current = auth.user.get().map(|u| u.id);
        if prev.is_some() && current != prev.clone().unwrap_or(None) && current.is_none() {
            mls.clear();
        }
        current
    });

    view! {
        <Router>
            <Navbar />
            <ChromeAndRoutes />
            <BugReportButton />
        </Router>
    }
}

/// Inside-router wrapper so `use_location` can read the current path. We use
/// it to collapse the onboarding banner + footer on routes that need to fill
/// the whole viewport (e.g. `/messages`), eliminating page-level scroll.
#[component]
fn ChromeAndRoutes() -> impl IntoView {
    let location = use_location();
    let is_fullscreen_route = Memo::new(move |_| location.pathname.get().starts_with("/messages"));

    view! {
        {move || (!is_fullscreen_route.get()).then(|| view! { <OnboardingChecklist /> })}
        <main class=move || if is_fullscreen_route.get() {
            "main-content main-content-fullscreen"
        } else {
            "main-content"
        }>
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
                <Route path=path!("/messages") view=MessagesPage />
                <Route path=path!("/about") view=AboutPage />
                <Route path=path!("/how-it-works") view=HowItWorksPage />
            </Routes>
        </main>
        {move || (!is_fullscreen_route.get()).then(|| view! { <Footer /> })}
    }
}
