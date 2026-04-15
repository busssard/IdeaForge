//! "Message privately" button on a user's profile. Clicks:
//!   1. Consume one of the target user's published KeyPackages (atomic).
//!   2. Create a new MLS group locally with that KeyPackage as the peer.
//!   3. POST the Welcome + group metadata to the delivery service.
//!   4. Navigate to `/messages` (the new group will surface on next poll).
//!
//! No plaintext ever leaves the browser — the server receives only Welcome
//! ciphertext and the opaque MLS group ID.

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::mls::api;
use crate::state::auth::AuthState;
use crate::state::mls_state::MlsState;

#[component]
pub fn MessagePrivatelyButton(target_user_id: String) -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let mls = expect_context::<MlsState>();
    let target = StoredValue::new(target_user_id);
    let error = RwSignal::new(String::new());
    let starting = RwSignal::new(false);
    // use_navigate() must be called during render, not inside an event
    // handler. Stash the resulting NavigateFn in a local-storage StoredValue
    // (the function isn't Send) so every click can grab a fresh clone.
    let navigate = StoredValue::new_local(use_navigate());

    let is_self = {
        let target_inner = target.get_value();
        move || auth.user.get().is_some_and(|u| u.id == target_inner)
    };

    let on_click = move |_: web_sys::MouseEvent| {
        if starting.get_untracked() {
            return;
        }
        let client = match mls.client_ref() {
            Some(c) => c,
            None => {
                // Keystore not unlocked yet — bounce to /messages so the
                // user can set up or enter their PIN, then come back.
                let navigate_fn = navigate.get_value();
                navigate_fn("/messages", Default::default());
                return;
            }
        };
        if !mls.ready.get_untracked() {
            error.set("Encryption still setting up — try again in a moment.".into());
            return;
        }

        starting.set(true);
        error.set(String::new());

        let target_user = target.get_value();
        let me = auth.user.get_untracked().map(|u| u.id).unwrap_or_default();
        let navigate_fn = navigate.get_value();

        wasm_bindgen_futures::spawn_local(async move {
            // Before creating a new group, look for an existing 1:1 with this
            // peer. Users expect "Message privately" to reopen their previous
            // thread, not silently pile up new groups each click.
            if let Ok(groups) = api::list_my_groups().await {
                let existing = groups.data.iter().find(|g| {
                    let ids: std::collections::HashSet<&str> =
                        g.members.iter().map(|m| m.user_id.as_str()).collect();
                    ids.len() == 2
                        && ids.contains(me.as_str())
                        && ids.contains(target_user.as_str())
                });
                if let Some(g) = existing {
                    mls.pending_selection.set(Some(g.id.clone()));
                    navigate_fn("/messages", Default::default());
                    starting.set(false);
                    return;
                }
            }

            // No existing conversation — do the full handshake.
            let kp_bytes = match api::consume_key_package(&target_user).await {
                Ok(bytes) => bytes,
                Err(e) => {
                    if e.status == 404 {
                        error.set("That user hasn't set up private messaging yet.".into());
                    } else {
                        error.set(format!("Couldn't start chat: {}", e.message));
                    }
                    starting.set(false);
                    return;
                }
            };

            let payload = {
                let mut c = client.borrow_mut();
                let peer_kp = match c.import_peer_key_package(&kp_bytes) {
                    Ok(kp) => kp,
                    Err(e) => {
                        error.set(format!("Peer KeyPackage invalid: {e}"));
                        starting.set(false);
                        return;
                    }
                };
                match c.create_group_with(vec![peer_kp]) {
                    Ok(p) => p,
                    Err(e) => {
                        error.set(format!("Couldn't build group: {e}"));
                        starting.set(false);
                        return;
                    }
                }
            };

            let req = api::CreateGroupRequest {
                mls_group_id_b64: api::encode(&payload.mls_group_id),
                name: None,
                initial_members: vec![target_user.clone()],
                welcomes_b64: vec![api::encode(&payload.welcome)],
            };
            match api::create_group(&req).await {
                Ok(resp) => {
                    mls.pending_selection.set(Some(resp.id));
                    mls.bump_revision();
                    // Creating a group mutated the OpenMLS storage — save it.
                    if let Some(keys) = mls.keys_ref() {
                        let snapshot = client.borrow().to_serialized();
                        if let Ok(snap) = snapshot
                            && let Ok(pt) = serde_json::to_vec(&snap)
                            && let Ok(wrapped) = crate::mls::crypto::seal(&keys.wrap_key, &pt)
                        {
                            use base64::{Engine, engine::general_purpose::STANDARD};
                            let body = serde_json::json!({
                                "verifier_b64": STANDARD.encode(keys.verifier),
                                "wrapped_blob_b64": STANDARD.encode(wrapped),
                            });
                            let _ =
                                crate::api::client::put::<serde_json::Value, serde_json::Value>(
                                    "/api/v1/mls/keystore",
                                    &body,
                                )
                                .await;
                        }
                    }
                    navigate_fn("/messages", Default::default());
                }
                Err(e) => {
                    error.set(format!("Server rejected group: {}", e.message));
                }
            }
            starting.set(false);
        });
    };

    view! {
        <div class="message-privately">
            {move || {
                if is_self() {
                    view! { <span></span> }.into_any()
                } else if !auth.is_authenticated() {
                    view! {
                        <p class="text-muted">
                            "Log in to message this user privately."
                        </p>
                    }
                    .into_any()
                } else {
                    view! {
                        <button
                            class="btn btn-secondary"
                            on:click=on_click
                            disabled=move || starting.get()
                            title="Open an end-to-end encrypted private message"
                        >
                            {move || if starting.get() { "Starting…" } else { "\u{1F510} Message privately" }}
                        </button>
                    }
                    .into_any()
                }
            }}
            {move || {
                let e = error.get();
                (!e.is_empty()).then(|| view! {
                    <p class="form-error">{e}</p>
                })
            }}
        </div>
    }
}
