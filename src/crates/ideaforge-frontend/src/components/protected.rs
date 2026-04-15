use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::state::auth::AuthState;

/// A wrapper component that redirects unauthenticated users to /login.
///
/// Waits for auth loading to complete before checking authentication.
/// Children are rendered eagerly and shown/hidden via CSS.
#[component]
pub fn Protected(children: Children) -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let navigate = use_navigate();

    // Only redirect when loading is done AND user is not authenticated
    Effect::new(move || {
        let loading = auth.loading.get();
        let authenticated = auth.is_authenticated();
        if !loading && !authenticated {
            navigate("/login", Default::default());
        }
    });

    let child_view = children();

    // `display: contents` removes the wrapper from layout entirely so the
    // real child participates directly in whatever layout its parent
    // established (flex, grid, etc.). Switch to `display: none` to hide the
    // pre-auth content without disturbing neighbours.
    let show_content = move || {
        if auth.is_authenticated() { "display: contents" } else { "display:none" }
    };
    let show_loading = move || {
        if auth.loading.get() && !auth.is_authenticated() { "" } else { "display:none" }
    };

    view! {
        <div style=show_content>
            {child_view}
        </div>
        <div style=show_loading>
            <div class="loading"><div class="spinner"></div></div>
        </div>
    }
}
