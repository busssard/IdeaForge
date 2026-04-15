//! First-visit explainer banner for `/messages`. Shown once per browser
//! (tracked in localStorage); explicitly dismissible.

use leptos::prelude::*;

const DISMISSED_KEY: &str = "ideaforge_messages_welcome_dismissed";

fn is_dismissed() -> bool {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(DISMISSED_KEY).ok().flatten())
        .map_or(false, |v| v == "1")
}

fn set_dismissed() {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(DISMISSED_KEY, "1");
    }
}

#[component]
pub fn MessagingWelcome() -> impl IntoView {
    let visible = RwSignal::new(!is_dismissed());
    let dismiss = move |_: web_sys::MouseEvent| {
        set_dismissed();
        visible.set(false);
    };

    view! {
        {move || {
            if !visible.get() {
                return view! { <span></span> }.into_any();
            }
            view! {
                <div class="messaging-welcome">
                    <div class="messaging-welcome-header">
                        <span class="messaging-welcome-icon">"\u{1F510}"</span>
                        <h3>"Private messaging, quickly explained"</h3>
                        <button
                            class="messaging-welcome-close"
                            on:click=dismiss.clone()
                            aria-label="Dismiss"
                            title="Dismiss"
                        >"\u{2715}"</button>
                    </div>
                    <ul class="messaging-welcome-list">
                        <li>
                            <strong>"End-to-end encrypted."</strong>
                            " Messages are sealed in your browser before they leave. The server only sees opaque ciphertext."
                        </li>
                        <li>
                            <strong>"6-digit PIN unlocks your history."</strong>
                            " You pick it once; it never leaves your device. We wrap your encryption keys under it."
                        </li>
                        <li>
                            <strong>"Forget the PIN → history gone."</strong>
                            " There is no reset and no recovery. We don't store anything we could read."
                        </li>
                        <li>
                            <strong>"Rate-limited."</strong>
                            " 3 wrong PIN attempts in an hour locks you out for an hour."
                        </li>
                        <li>
                            <strong>"Markdown works."</strong>
                            " Use the toolbar above the text field to format, link, or paste an image."
                        </li>
                    </ul>
                    <div class="messaging-welcome-footer">
                        <a href="/how-it-works" class="btn btn-secondary btn-sm">"How it works"</a>
                        <button class="btn btn-primary btn-sm" on:click=dismiss>
                            "Got it"
                        </button>
                    </div>
                </div>
            }.into_any()
        }}
    }
}
