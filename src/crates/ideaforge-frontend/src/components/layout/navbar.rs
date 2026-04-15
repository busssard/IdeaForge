use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use crate::components::forge_logo::ForgeLogo;
use crate::components::notification_bell::NotificationBell;
use crate::state::auth::AuthState;

#[component]
pub fn Navbar() -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let is_logged_in = Memo::new(move |_| auth.is_authenticated());
    let logout = move |_| {
        auth.logout();
        let navigate = use_navigate();
        navigate("/", Default::default());
    };

    view! {
        <nav class="navbar">
            <div class="navbar-inner">
                <A href="/" attr:class="navbar-brand">
                    <ForgeLogo class="navbar-logo".to_string() />
                    <span class="navbar-brand-text">"IdeaForge"</span>
                </A>

                <div class="navbar-links">
                    <A href="/browse">"Forge Floor"</A>
                    <A href="/people">"People"</A>
                    <A href="/dashboard">"Dashboard"</A>
                </div>

                <div class="navbar-actions">
                    {move || {
                        if is_logged_in.get() {
                            view! {
                                <A href="/ideas/new" attr:class="btn btn-primary btn-sm">
                                    "Bring to the Forge"
                                </A>
                                <A href="/messages" attr:class="btn btn-ghost btn-sm">
                                    "\u{1F510} Messages"
                                </A>
                                <NotificationBell />
                                <A href="/settings" attr:class="btn btn-ghost btn-sm">
                                    "Settings"
                                </A>
                                <button class="btn btn-ghost btn-sm" on:click=logout>
                                    "Logout"
                                </button>
                            }.into_any()
                        } else {
                            view! {
                                <A href="/login" attr:class="btn btn-secondary btn-sm">
                                    "Login"
                                </A>
                                <A href="/register" attr:class="btn btn-primary btn-sm">
                                    "Join the Forge"
                                </A>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </nav>
    }
}
