use leptos::prelude::*;

use crate::api;
use crate::state::auth::AuthState;

#[component]
pub fn SubscribeButton(idea_id: String) -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let subscribed = RwSignal::new(false);
    let loading = RwSignal::new(false);
    let error = RwSignal::new(String::new());
    let idea_id_stored = StoredValue::new(idea_id);

    // Fetch current subscription state on mount so the button doesn't start
    // with a stale `false` and then silently no-op on click for already-subscribed users.
    if auth.is_authenticated() {
        let id = idea_id_stored.get_value();
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(status) = api::subscriptions::get_subscription_status(&id).await {
                subscribed.set(status.subscribed);
            }
        });
    }

    let toggle = move |_| {
        if loading.get_untracked() {
            return;
        }
        if !auth.is_authenticated() {
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/login");
            }
            return;
        }

        loading.set(true);
        error.set(String::new());
        let was_subscribed = subscribed.get_untracked();
        let id = idea_id_stored.get_value();

        // Optimistic update
        subscribed.set(!was_subscribed);

        wasm_bindgen_futures::spawn_local(async move {
            let result = if was_subscribed {
                api::subscriptions::unsubscribe(&id).await.map(|_| ())
            } else {
                api::subscriptions::subscribe(&id).await.map(|_| ())
            };
            if let Err(e) = result {
                subscribed.set(was_subscribed); // revert
                if e.status == 401 {
                    // The api client already tried a refresh; if we still get
                    // 401 the session is truly dead. Clear it and send them
                    // to /login instead of surfacing "invalid or expired token".
                    auth.logout();
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/login");
                    }
                } else {
                    error.set(e.message);
                }
            }
            loading.set(false);
        });
    };

    view! {
        <div class="subscribe-wrapper">
            <button
                class=move || {
                    if subscribed.get() {
                        "btn btn-ghost btn-sm subscribed"
                    } else {
                        "btn btn-ghost btn-sm"
                    }
                }
                on:click=toggle
                disabled=move || loading.get()
                title=move || {
                    if !auth.is_authenticated() { "Log in to subscribe".to_string() }
                    else if subscribed.get() { "You'll be notified of updates — click to unsubscribe".to_string() }
                    else { "Get notified when this idea is updated".to_string() }
                }
            >
                <span class="subscribe-icon">
                    {move || if subscribed.get() { "\u{1F514}" } else { "\u{1F515}" }}
                </span>
                {move || {
                    if loading.get() { "..." }
                    else if subscribed.get() { "Subscribed" }
                    else { "Subscribe" }
                }}
            </button>
            {move || {
                let msg = error.get();
                (!msg.is_empty()).then(|| view! {
                    <span class="subscribe-error">{msg}</span>
                })
            }}
        </div>
    }
}
