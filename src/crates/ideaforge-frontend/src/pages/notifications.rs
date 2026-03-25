use leptos::prelude::*;
use leptos_router::components::A;

use crate::api;
use crate::api::types::NotificationResponse;
use crate::components::loading::Loading;
use crate::components::protected::Protected;

#[component]
pub fn NotificationsPage() -> impl IntoView {
    view! {
        <Protected>
            <NotificationsContent />
        </Protected>
    }
}

#[component]
fn NotificationsContent() -> impl IntoView {
    let refresh_trigger = RwSignal::new(0u32);

    let notifications = LocalResource::new(move || {
        let _ = refresh_trigger.get(); // track changes
        async move {
            api::notifications::list_notifications(1, 50, false).await
        }
    });

    let mark_all = move |_: web_sys::MouseEvent| {
        wasm_bindgen_futures::spawn_local(async move {
            if api::notifications::mark_all_read().await.is_ok() {
                refresh_trigger.set(refresh_trigger.get_untracked() + 1);
            }
        });
    };

    view! {
        <div class="page">
            <div class="page-header" style="display: flex; justify-content: space-between; align-items: center;">
                <h1 class="page-title">"Notifications"</h1>
                <button class="btn btn-secondary btn-sm" on:click=mark_all>
                    "Mark all read"
                </button>
            </div>

            <Suspense fallback=move || view! { <Loading /> }>
                {move || {
                    notifications.get().map(|result| {
                        match &*result {
                            Ok(resp) => {
                                if resp.data.is_empty() {
                                    view! {
                                        <div class="empty-state">
                                            <h3>"All caught up!"</h3>
                                            <p>"No notifications yet. Go explore the forge!"</p>
                                            <A href="/browse" attr:class="btn btn-primary">"Browse Ideas"</A>
                                        </div>
                                    }.into_any()
                                } else {
                                    let items: Vec<NotificationResponse> = resp.data.clone();
                                    let refresh = refresh_trigger;
                                    view! {
                                        <div class="notification-list">
                                            {items.into_iter().map(|notif| {
                                                view! { <NotificationItem notif=notif refresh=refresh /> }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }
                            }
                            Err(e) => {
                                view! {
                                    <div class="error-display">
                                        <p>{e.message.clone()}</p>
                                    </div>
                                }.into_any()
                            }
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn NotificationItem(notif: NotificationResponse, refresh: RwSignal<u32>) -> impl IntoView {
    let is_unread = notif.read_at.is_none();
    let id = notif.id.clone();
    let kind = notif.kind.clone();
    let title = notif.title.clone();
    let message = notif.message.clone();
    let link_url = notif.link_url.clone();
    let created = notif.created_at.split('T').next().unwrap_or("").to_string();

    let kind_icon = match kind.as_str() {
        "stoke" => "\u{1F525}",
        "comment" => "\u{1F4AC}",
        "suggestion" => "\u{1F4A1}",
        "team_application" => "\u{1F91D}",
        "team_accepted" => "\u{2705}",
        "team_rejected" => "\u{274C}",
        "milestone" => "\u{1F3C6}",
        "bot_analysis" => "\u{1F916}",
        "mention" => "\u{1F4E2}",
        _ => "\u{1F514}",
    };

    let card_class = if is_unread {
        "notification-item notification-unread"
    } else {
        "notification-item"
    };

    let mark_read = move |_: web_sys::MouseEvent| {
        let id = id.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if api::notifications::mark_read(&id).await.is_ok() {
                refresh.set(refresh.get_untracked() + 1);
            }
        });
    };

    view! {
        <div class=card_class>
            <span class="notification-icon">{kind_icon}</span>
            <div class="notification-content">
                <p class="notification-title">{title}</p>
                <p class="notification-message">{message}</p>
                <div class="notification-meta">
                    <span class="notification-time">{created}</span>
                    {link_url.map(|url| view! {
                        <A href=url attr:class="notification-link">"View"</A>
                    })}
                    {is_unread.then(|| view! {
                        <button class="btn-text" on:click=mark_read>"Mark read"</button>
                    })}
                </div>
            </div>
        </div>
    }
}
