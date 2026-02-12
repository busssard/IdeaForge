use leptos::prelude::*;

use crate::api;
use crate::state::auth::AuthState;

#[component]
pub fn SubscribeButton(idea_id: String) -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let subscribed = RwSignal::new(false);
    let loading = RwSignal::new(false);
    let idea_id_stored = StoredValue::new(idea_id);

    let toggle = move |_| {
        if loading.get_untracked() {
            return;
        }
        if !auth.is_authenticated() {
            return;
        }

        loading.set(true);
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
            if result.is_err() {
                subscribed.set(was_subscribed); // revert
            }
            loading.set(false);
        });
    };

    view! {
        <button
            class=move || {
                if subscribed.get() {
                    "btn btn-ghost btn-sm subscribed"
                } else {
                    "btn btn-ghost btn-sm"
                }
            }
            on:click=toggle
            disabled=move || loading.get() || !auth.is_authenticated()
        >
            {move || if subscribed.get() { "Subscribed" } else { "Subscribe" }}
        </button>
    }
}
