use leptos::prelude::*;

use crate::state::auth::AuthState;

/// A wrapper component that redirects unauthenticated users to /login.
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

    view! {
        {move || {
            if is_authenticated.get() {
                children().into_any()
            } else {
                view! {
                    <div class="loading"><div class="spinner"></div></div>
                }.into_any()
            }
        }}
    }
}
