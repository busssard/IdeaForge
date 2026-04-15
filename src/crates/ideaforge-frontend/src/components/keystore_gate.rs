//! Gates access to any MLS feature behind a PIN. First visit → set-a-PIN
//! form; subsequent visits → unlock-with-PIN form. Once the client is
//! installed in `MlsState`, the wrapped child renders.

use leptos::prelude::*;

use crate::components::loading::Loading;
use crate::mls::{api, client::MlsClient, keystore};
use crate::state::auth::AuthState;
use crate::state::mls_state::MlsState;

#[derive(Clone, Debug)]
enum Phase {
    Loading,
    NoKeystore,
    Locked(i64), // timestamp ms
    Unlock,
    Unlocked,
    Error(String),
}

/// Wraps a child view. The child only renders after the user has unlocked
/// their keystore this session.
#[component]
pub fn KeystoreGate(children: ChildrenFn) -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let mls = expect_context::<MlsState>();
    let phase = RwSignal::new(Phase::Loading);

    // Kick off a status fetch on mount. If the client is already installed,
    // we short-circuit to Unlocked. On transient failures (network blip,
    // token expiry etc.) we retry every 10 seconds until success — so
    // `lost-token` errors resolve themselves without the user needing to
    // hammer reload.
    Effect::new(move |_| {
        if mls.client_ref().is_some() {
            phase.set(Phase::Unlocked);
            return;
        }
        wasm_bindgen_futures::spawn_local(async move {
            loop {
                match keystore::status().await {
                    Ok(s) => {
                        if s.exists {
                            if let Some(locked_ms) = s.locked_until_ms {
                                let now_ms = chrono::Utc::now().timestamp_millis();
                                if locked_ms > now_ms {
                                    phase.set(Phase::Locked(locked_ms));
                                    return;
                                }
                            }
                            phase.set(Phase::Unlock);
                        } else {
                            phase.set(Phase::NoKeystore);
                        }
                        return;
                    }
                    Err(e) => {
                        phase.set(Phase::Error(format!(
                            "Couldn't reach the keystore — retrying… ({})",
                            e.message
                        )));
                        gloo_timers::future::TimeoutFuture::new(10_000).await;
                    }
                }
            }
        });
    });

    view! {
        {move || match phase.get() {
            Phase::Loading => view! { <Loading /> }.into_any(),
            Phase::Unlocked => children().into_any(),
            Phase::NoKeystore => view! {
                <SetupForm phase=phase auth=auth mls=mls />
            }.into_any(),
            Phase::Unlock => view! {
                <UnlockForm phase=phase mls=mls />
            }.into_any(),
            Phase::Locked(ms) => view! {
                <LockedNotice locked_until_ms=ms />
            }.into_any(),
            Phase::Error(msg) => view! {
                <div class="keystore-error card">
                    <h3>"Couldn't reach the keystore"</h3>
                    <p>{msg}</p>
                </div>
            }.into_any(),
        }}
    }
}

#[component]
fn SetupForm(phase: RwSignal<Phase>, auth: AuthState, mls: MlsState) -> impl IntoView {
    let pin1 = RwSignal::new(String::new());
    let pin2 = RwSignal::new(String::new());
    let working = RwSignal::new(false);
    let err = RwSignal::new(String::new());

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if working.get_untracked() {
            return;
        }
        err.set(String::new());

        let a = pin1.get_untracked();
        let b = pin2.get_untracked();
        if !is_six_digit(&a) {
            err.set("PIN must be exactly 6 digits.".into());
            return;
        }
        if a != b {
            err.set("The two PINs don't match.".into());
            return;
        }
        let user_id = match auth.user.get_untracked() {
            Some(u) => u.id,
            None => {
                err.set("You aren't logged in.".into());
                return;
            }
        };

        working.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            // Yield so the browser can paint "Setting up…" before we block
            // the main thread for ~1s on Argon2id key derivation.
            gloo_timers::future::TimeoutFuture::new(16).await;
            let client = match MlsClient::new(user_id.into_bytes()) {
                Ok(c) => c,
                Err(e) => {
                    err.set(format!("MLS client init failed: {e}"));
                    working.set(false);
                    return;
                }
            };
            let keys = match keystore::setup(&a, &client).await {
                Ok(k) => k,
                Err(e) => {
                    err.set(format!("Setup failed: {}", e.message));
                    working.set(false);
                    return;
                }
            };

            // Publish a starter batch of KeyPackages so other users can invite us.
            let mut kps = Vec::with_capacity(4);
            for _ in 0..4 {
                let bundle = match client.generate_key_package() {
                    Ok(b) => b,
                    Err(e) => {
                        err.set(format!("KeyPackage gen failed: {e}"));
                        working.set(false);
                        return;
                    }
                };
                match client.serialize_key_package(&bundle) {
                    Ok(bytes) => kps.push(bytes),
                    Err(e) => {
                        err.set(format!("KeyPackage serialize failed: {e}"));
                        working.set(false);
                        return;
                    }
                }
            }
            if let Err(e) = api::publish_key_packages(&kps, 90).await {
                err.set(format!("Publish KeyPackages failed: {}", e.message));
                working.set(false);
                return;
            }

            // Re-persist so the blob captures the fresh KeyPackage entries
            // OpenMLS wrote into storage.
            if let Err(e) = keystore::persist(&client, &keys).await {
                err.set(format!("Persist after KP gen failed: {}", e.message));
                working.set(false);
                return;
            }

            mls.set_client(client, keys);
            mls.ready.set(true);
            phase.set(Phase::Unlocked);
        });
    };

    view! {
        <div class="keystore-panel card">
            <h2>"Set a 6-digit PIN"</h2>
            <p class="text-muted">
                "Your messages are end-to-end encrypted. Pick a 6-digit PIN; you'll enter it each
                time you open your inbox. If you forget it, your message history is unrecoverable —
                we don't store it anywhere we could read."
            </p>
            <p class="text-muted keystore-perf-hint">
                "Setting up the encryption takes up to ~60 seconds on slower machines — your browser
                may feel unresponsive for a moment."
            </p>
            <form on:submit=on_submit class="keystore-form">
                <label class="form-group">
                    <span class="form-label">"Pick a PIN"</span>
                    <input
                        class="form-input pin-input"
                        type="password"
                        inputmode="numeric"
                        autocomplete="new-password"
                        maxlength="6"
                        required
                        prop:value=move || pin1.get()
                        on:input=move |ev| pin1.set(event_target_value(&ev))
                    />
                </label>
                <label class="form-group">
                    <span class="form-label">"Confirm PIN"</span>
                    <input
                        class="form-input pin-input"
                        type="password"
                        inputmode="numeric"
                        autocomplete="new-password"
                        maxlength="6"
                        required
                        prop:value=move || pin2.get()
                        on:input=move |ev| pin2.set(event_target_value(&ev))
                    />
                </label>
                {move || {
                    let e = err.get();
                    (!e.is_empty()).then(|| view! { <p class="form-error">{e}</p> })
                }}
                <button
                    type="submit"
                    class="btn btn-primary"
                    disabled=move || working.get()
                >
                    {move || if working.get() { "Setting up…" } else { "Set PIN and enter" }}
                </button>
            </form>
            <p class="keystore-help">
                <a href="/how-it-works" target="_blank" rel="noopener">
                    "How private messaging works \u{2192}"
                </a>
            </p>
        </div>
    }
}

#[component]
fn UnlockForm(phase: RwSignal<Phase>, mls: MlsState) -> impl IntoView {
    let pin = RwSignal::new(String::new());
    let working = RwSignal::new(false);
    let err = RwSignal::new(String::new());

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if working.get_untracked() {
            return;
        }
        err.set(String::new());
        let p = pin.get_untracked();
        if !is_six_digit(&p) {
            err.set("PIN must be 6 digits.".into());
            return;
        }

        working.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            // Yield so the browser can paint "Unlocking…" before we block on
            // Argon2id derivation.
            gloo_timers::future::TimeoutFuture::new(16).await;
            match keystore::unlock(&p).await {
                Ok((client, keys)) => {
                    mls.set_client(client, keys);
                    mls.ready.set(true);
                    phase.set(Phase::Unlocked);
                }
                Err(e) => {
                    match e.code.as_str() {
                        "LOCKED" => {
                            // Server said try-later; flip to locked notice if
                            // we can parse the locked_until. Falls through to
                            // plain error otherwise.
                            if e.message.contains("later") {
                                err.set("Too many failed PIN attempts. Try again later.".into());
                                // Re-fetch status to update UI
                                if let Ok(s) = keystore::status().await
                                    && let Some(ms) = s.locked_until_ms
                                {
                                    phase.set(Phase::Locked(ms));
                                }
                            } else {
                                err.set(e.message);
                            }
                        }
                        _ => err.set(e.message),
                    }
                    working.set(false);
                }
            }
        });
    };

    view! {
        <div class="keystore-panel card">
            <h2>"Enter your PIN"</h2>
            <p class="text-muted">
                "Your encrypted messages are locked with a 6-digit PIN. 3 wrong attempts in an hour
                will lock you out for an hour."
            </p>
            <p class="text-muted keystore-perf-hint">
                "Unlocking takes up to ~60 seconds on slower machines — your browser may feel
                unresponsive for a moment."
            </p>
            <form on:submit=on_submit class="keystore-form">
                <label class="form-group">
                    <span class="form-label">"PIN"</span>
                    <input
                        class="form-input pin-input"
                        type="password"
                        inputmode="numeric"
                        autocomplete="current-password"
                        maxlength="6"
                        required
                        autofocus
                        prop:value=move || pin.get()
                        on:input=move |ev| pin.set(event_target_value(&ev))
                    />
                </label>
                {move || {
                    let e = err.get();
                    (!e.is_empty()).then(|| view! { <p class="form-error">{e}</p> })
                }}
                <button
                    type="submit"
                    class="btn btn-primary"
                    disabled=move || working.get()
                >
                    {move || if working.get() { "Unlocking…" } else { "Unlock" }}
                </button>
            </form>
            <p class="keystore-help">
                <a href="/how-it-works" target="_blank" rel="noopener">
                    "How private messaging works \u{2192}"
                </a>
            </p>
        </div>
    }
}

#[component]
fn LockedNotice(locked_until_ms: i64) -> impl IntoView {
    let until_str = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(locked_until_ms)
        .map(|t| t.format("%H:%M UTC").to_string())
        .unwrap_or_else(|| "later".into());
    view! {
        <div class="keystore-panel card">
            <h2>"Locked out"</h2>
            <p class="text-muted">
                "Too many failed PIN attempts. Your keystore is locked until "
                {until_str}
                ". This protects against someone trying to brute-force your PIN."
            </p>
            <p class="keystore-help">
                <a href="/how-it-works" target="_blank" rel="noopener">
                    "How private messaging works \u{2192}"
                </a>
            </p>
        </div>
    }
}

fn is_six_digit(s: &str) -> bool {
    s.len() == 6 && s.chars().all(|c| c.is_ascii_digit())
}
