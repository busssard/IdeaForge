use leptos::prelude::*;

use crate::state::auth::AuthState;

/// A wrapper component that redirects unauthenticated users to /login.
///
/// Children are rendered eagerly and placed inside a conditionally-visible
/// container. An `Effect` handles the redirect when the user is not
/// authenticated.
#[component]
pub fn Protected(children: Children) -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let is_authenticated = Memo::new(move |_| auth.is_authenticated());

    Effect::new(move || {
        if !is_authenticated.get() {
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/login");
            }
        }
    });

    // Eagerly evaluate children outside of any reactive closure.
    // Children is FnOnce so it cannot be called inside a move || closure.
    // We render both branches and toggle visibility via CSS display.
    let child_view = children();

    let authenticated_style = move || {
        if is_authenticated.get() { "" } else { "display:none" }
    };
    let loading_style = move || {
        if is_authenticated.get() { "display:none" } else { "" }
    };

    view! {
        <div style=authenticated_style>
            {child_view}
        </div>
        <div style=loading_style>
            <div class="loading"><div class="spinner"></div></div>
        </div>
    }
}
