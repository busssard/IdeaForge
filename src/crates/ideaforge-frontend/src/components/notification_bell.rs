use leptos::prelude::*;
use leptos_router::components::A;

use crate::api;
use crate::state::auth::AuthState;

/// How often to re-poll the unread count while the nav is mounted. 30s is
/// a reasonable compromise between "marker shows up quickly after a new
/// message arrives" and "don't hammer the API".
const POLL_INTERVAL_MS: u32 = 30_000;

/// Notification bell icon in the navbar showing unread count.
#[component]
pub fn NotificationBell() -> impl IntoView {
    let auth = expect_context::<AuthState>();

    // Tick bumps every POLL_INTERVAL_MS so the resource re-runs and the badge
    // reflects newly-arrived notifications without a page refresh.
    let tick = RwSignal::new(0u32);
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(POLL_INTERVAL_MS).await;
                tick.update(|t| *t = t.wrapping_add(1));
            }
        });
    });

    let unread = LocalResource::new(move || {
        let _ = tick.get();
        async move {
            if auth.user.get_untracked().is_some() {
                api::notifications::unread_count()
                    .await
                    .ok()
                    .map(|r| r.unread_count)
            } else {
                None
            }
        }
    });

    view! {
        {move || {
            if auth.user.get().is_none() {
                return view! { <span></span> }.into_any();
            }

            let count = unread.get()
                .and_then(|r| (*r).clone())
                .unwrap_or(0);

            view! {
                <A href="/notifications" attr:class="notification-bell" attr:title="Notifications">
                    <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                        <path d="M12 22c1.1 0 2-.9 2-2h-4c0 1.1.89 2 2 2zm6-6v-5c0-3.07-1.64-5.64-4.5-6.32V4c0-.83-.67-1.5-1.5-1.5s-1.5.67-1.5 1.5v.68C7.63 5.36 6 7.92 6 11v5l-2 2v1h16v-1l-2-2z"/>
                    </svg>
                    {(count > 0).then(|| view! {
                        <span class="notification-badge">{count}</span>
                    })}
                </A>
            }.into_any()
        }}
    }
}
