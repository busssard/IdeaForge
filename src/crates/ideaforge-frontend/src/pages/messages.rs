//! `/messages` — E2E-encrypted messaging page. All crypto happens in the
//! browser; the server only sees ciphertext (see
//! `docs/architecture/simplex_messaging_spike.md` for the full threat
//! model).
//!
//! Phase-1d scope:
//!   - list the groups the user belongs to
//!   - auto-accept pending Welcomes so newly-invited conversations appear
//!   - show a basic chat view for the selected group with polling-based
//!     receive and a plain text input
//!   - graceful empty states when no groups / no MLS client yet

use std::collections::HashMap;

use leptos::prelude::*;

use crate::api::client::ApiError;

macro_rules! warn {
    ($($t:tt)*) => {
        web_sys::console::warn_1(&format!($($t)*).into())
    };
}
use crate::components::keystore_gate::KeystoreGate;
use crate::components::loading::Loading;
use crate::components::markdown::Markdown;
use crate::components::markdown_editor::MarkdownEditor;
use crate::components::messaging_welcome::MessagingWelcome;
use crate::components::protected::Protected;
use crate::mls::api;
use crate::state::auth::AuthState;
use crate::state::mls_state::{MlsClientRef, MlsState};

/// How often to poll for new welcomes + messages when a chat is open.
const POLL_INTERVAL_MS: u32 = 2_500;

/// Local decrypted-message cache, keyed by the server-side group id.
#[derive(Clone, Debug)]
struct LocalMessage {
    #[allow(dead_code)]
    server_id: i64,
    sender_user_id: String,
    plaintext: String,
    created_at: String,
}

/// A horizontal drag handle pinned above the composer. Clicking and
/// dragging up grows the textarea; dragging down shrinks it. Replaces the
/// native bottom-right resize handle (which CSS can't reposition). Relies
/// on the textarea having the `chat-composer-input` class.
#[component]
fn ComposerResizeHandle() -> impl IntoView {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    let on_pointer_down = move |ev: web_sys::PointerEvent| {
        ev.prevent_default();
        let start_y = ev.client_y();

        let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
            return;
        };
        let Ok(Some(ta_el)) = doc.query_selector(".chat-composer-input") else {
            return;
        };
        let Ok(textarea) = ta_el.clone().dyn_into::<web_sys::HtmlElement>() else {
            return;
        };
        let start_h = textarea.offset_height();

        // Use a Rc<Cell<Option<Closure>>> trick: install pointermove +
        // pointerup closures on document; pointerup tears both down. The
        // `leaked: Rc<...>` keeps them alive for the duration of the drag.
        let move_closure_slot: std::rc::Rc<
            std::cell::RefCell<Option<Closure<dyn FnMut(web_sys::PointerEvent)>>>,
        > = std::rc::Rc::new(std::cell::RefCell::new(None));
        let up_closure_slot: std::rc::Rc<
            std::cell::RefCell<Option<Closure<dyn FnMut(web_sys::PointerEvent)>>>,
        > = std::rc::Rc::new(std::cell::RefCell::new(None));

        // --- pointermove: update textarea height ---
        let textarea_for_move = textarea.clone();
        let move_closure =
            Closure::<dyn FnMut(web_sys::PointerEvent)>::new(move |ev: web_sys::PointerEvent| {
                let delta = start_y - ev.client_y(); // up = positive
                let new_h = (start_h + delta).max(70).min(800);
                let _ = textarea_for_move
                    .style()
                    .set_property("height", &format!("{new_h}px"));
            });
        let _ = doc
            .add_event_listener_with_callback("pointermove", move_closure.as_ref().unchecked_ref());
        *move_closure_slot.borrow_mut() = Some(move_closure);

        // --- pointerup: detach both listeners ---
        let doc_for_up = doc.clone();
        let move_slot_for_up = move_closure_slot.clone();
        let up_slot_for_up = up_closure_slot.clone();
        let up_closure =
            Closure::<dyn FnMut(web_sys::PointerEvent)>::new(move |_ev: web_sys::PointerEvent| {
                if let Some(mc) = move_slot_for_up.borrow_mut().take() {
                    let _ = doc_for_up.remove_event_listener_with_callback(
                        "pointermove",
                        mc.as_ref().unchecked_ref(),
                    );
                }
                if let Some(uc) = up_slot_for_up.borrow_mut().take() {
                    let _ = doc_for_up.remove_event_listener_with_callback(
                        "pointerup",
                        uc.as_ref().unchecked_ref(),
                    );
                }
            });
        let _ =
            doc.add_event_listener_with_callback("pointerup", up_closure.as_ref().unchecked_ref());
        *up_closure_slot.borrow_mut() = Some(up_closure);
    };

    view! {
        <div
            class="chat-composer-resize"
            role="separator"
            aria-orientation="horizontal"
            aria-label="Resize message composer"
            on:pointerdown=on_pointer_down
        >
            <span class="chat-composer-resize-grip" aria-hidden="true"></span>
        </div>
    }
}

/// Render an RFC 3339 timestamp as a compact user-visible string.
/// Same day → "HH:MM"; otherwise → "Mon DD, HH:MM".
fn format_timestamp(ts: &str) -> String {
    let Ok(when) = chrono::DateTime::parse_from_rfc3339(ts) else {
        return String::new();
    };
    let when_utc = when.with_timezone(&chrono::Utc);
    let now = chrono::Utc::now();
    let same_day = when_utc.date_naive() == now.date_naive();
    if same_day {
        when_utc.format("%H:%M").to_string()
    } else {
        when_utc.format("%b %-d, %H:%M").to_string()
    }
}

#[derive(Copy, Clone)]
struct ChatState {
    /// `mls_group_id` bytes (b64) → list of decrypted messages.
    messages: RwSignal<HashMap<String, Vec<LocalMessage>>>,
    /// `server_group_id` → `mls_group_id` bytes (b64). Populated as the user
    /// lists their groups.
    group_map: RwSignal<HashMap<String, String>>,
    /// Currently-open conversation. Server group id.
    selected: RwSignal<Option<String>>,
    /// Transient send/receive error for the active chat.
    error: RwSignal<String>,
    /// Composer draft text. Lives at the page level (not inside `ChatView`)
    /// so it survives the re-renders triggered by the 2.5s poll — otherwise
    /// each refetch would re-instantiate `ChatView` with a fresh empty
    /// signal and the user's keystrokes would disappear mid-typing.
    draft: RwSignal<String>,
}

impl ChatState {
    fn new() -> Self {
        Self {
            messages: RwSignal::new(HashMap::new()),
            group_map: RwSignal::new(HashMap::new()),
            selected: RwSignal::new(None),
            error: RwSignal::new(String::new()),
            draft: RwSignal::new(String::new()),
        }
    }
}

#[component]
pub fn MessagesPage() -> impl IntoView {
    view! {
        <Protected>
            <KeystoreGate>
                <MessagesInner />
            </KeystoreGate>
        </Protected>
    }
}

#[component]
fn MessagesInner() -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let mls = expect_context::<MlsState>();
    let chat = ChatState::new();

    // Reactive: list of groups the user currently belongs to.
    let groups = LocalResource::new(move || {
        // Re-run when the MLS client applies new state (joined a group, etc).
        let _ = mls.revision.get();
        async move { api::list_my_groups().await }
    });

    // Accept welcomes as soon as they show up. Runs on a timer so a fresh
    // invitation from another tab / device appears without a manual refresh.
    spawn_welcome_accept_loop(mls, chat, groups);

    // Poll messages for the currently-selected group.
    spawn_message_poll_loop(mls, chat);

    view! {
        <MessagingWelcome />
        <div class="messages-page">
            <aside class="messages-sidebar">
                <div class="messages-sidebar-header">
                    <h2>"Messages"</h2>
                    {move || {
                        let err = mls.init_error.get();
                        (!err.is_empty()).then(|| view! {
                            <p class="form-error">{err}</p>
                        })
                    }}
                    {move || (!mls.ready.get()).then(|| view! {
                        <p class="text-muted">"Setting up encryption…"</p>
                    })}
                </div>

                <Transition fallback=move || view! { <Loading /> }>
                    {move || {
                        let my_id = auth.user.get().map(|u| u.id).unwrap_or_default();
                        groups.get().map(|res| match &*res {
                            Ok(list) => {
                                // Stash mls_group_id lookups for the chat code.
                                let mut map = HashMap::new();
                                for g in &list.data {
                                    map.insert(g.id.clone(), g.mls_group_id_b64.clone());
                                }
                                chat.group_map.set(map);

                                // Hydrate chat.messages from the persisted
                                // client state so BOTH sent and received
                                // history survive refreshes.
                                if let Some(client) = mls.client_ref() {
                                    let c = client.borrow();
                                    chat.messages.update(|bucket_map| {
                                        for g in &list.data {
                                            if bucket_map.get(&g.mls_group_id_b64).map(|v| !v.is_empty()).unwrap_or(false) {
                                                continue;
                                            }
                                            let mls_id = match crate::mls::api::decode(&g.mls_group_id_b64) {
                                                Ok(v) => v,
                                                Err(_) => continue,
                                            };
                                            let history: Vec<LocalMessage> = c
                                                .messages_for(&mls_id)
                                                .map(|s| LocalMessage {
                                                    server_id: s.server_id.unwrap_or(0),
                                                    sender_user_id: s.sender_user_id.clone(),
                                                    plaintext: s.plaintext.clone(),
                                                    created_at: s.created_at.clone(),
                                                })
                                                .collect();
                                            if !history.is_empty() {
                                                bucket_map.insert(g.mls_group_id_b64.clone(), history);
                                            }
                                        }
                                    });
                                }

                                // If the profile page parked a conversation to
                                // open, select it now that the list is loaded.
                                if let Some(target) = mls.pending_selection.get_untracked()
                                    && list.data.iter().any(|g| g.id == target) {
                                        chat.selected.set(Some(target));
                                        mls.pending_selection.set(None);
                                    }

                                if list.data.is_empty() {
                                    view! {
                                        <div class="messages-empty">
                                            <p class="text-muted">
                                                "No private conversations yet. Open someone's profile and click "
                                                <em>"Message privately"</em>"."
                                            </p>
                                        </div>
                                    }.into_any()
                                } else {
                                    let items = list.data.clone();
                                    let my_id_outer = my_id.clone();
                                    view! {
                                        <ul class="conversation-list">
                                            {items.into_iter().map(|g| {
                                                let id = g.id.clone();
                                                let id_click = id.clone();
                                                let id_select = id.clone();
                                                let id_delete = id.clone();
                                                let title = conversation_title(&g, &my_id_outer);
                                                let title_confirm = title.clone();
                                                let is_selected = move || {
                                                    chat.selected.get().as_deref() == Some(&id_select)
                                                };
                                                let on_delete = move |ev: web_sys::MouseEvent| {
                                                    ev.stop_propagation();
                                                    // `window.confirm` is synchronous in the browser —
                                                    // fine for a destructive action the user explicitly invoked.
                                                    let prompt = format!(
                                                        "Delete conversation \"{}\"? This removes it from your \
                                                         account; if no one else is in it, the server-side \
                                                         history is wiped.",
                                                        title_confirm
                                                    );
                                                    let confirmed = web_sys::window()
                                                        .and_then(|w| w.confirm_with_message(&prompt).ok())
                                                        .unwrap_or(false);
                                                    if !confirmed {
                                                        return;
                                                    }
                                                    let id = id_delete.clone();
                                                    wasm_bindgen_futures::spawn_local(async move {
                                                        match api::leave_group(&id).await {
                                                            Ok(()) => {
                                                                // If we were looking at this chat, close the pane.
                                                                if chat.selected.get_untracked().as_deref() == Some(&id) {
                                                                    chat.selected.set(None);
                                                                }
                                                                chat.messages.update(|m| {
                                                                    // Forget any decrypted history we held.
                                                                    let stale: Vec<String> = m.keys().cloned().collect();
                                                                    for k in stale {
                                                                        m.remove(&k);
                                                                    }
                                                                });
                                                                mls.bump_revision();
                                                            }
                                                            Err(e) => {
                                                                chat.error.set(format!(
                                                                    "Couldn't delete: {}", e.message
                                                                ));
                                                            }
                                                        }
                                                    });
                                                };
                                                view! {
                                                    <li class="conversation-row">
                                                        <button
                                                            class=move || if is_selected() {
                                                                "conversation-item conversation-item-active"
                                                            } else {
                                                                "conversation-item"
                                                            }
                                                            on:click=move |_| {
                                                                chat.selected.set(Some(id_click.clone()));
                                                                chat.error.set(String::new());
                                                                chat.draft.set(String::new());
                                                            }
                                                        >
                                                            {title}
                                                        </button>
                                                        <button
                                                            class="conversation-delete"
                                                            title="Delete this conversation"
                                                            aria-label="Delete conversation"
                                                            on:click=on_delete
                                                        >
                                                            "\u{1F5D1}"
                                                        </button>
                                                    </li>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </ul>
                                    }.into_any()
                                }
                            }
                            Err(e) => view! {
                                <p class="form-error">{e.message.clone()}</p>
                            }.into_any(),
                        })
                    }}
                </Transition>
            </aside>

            <section class="messages-thread">
                {move || {
                    match chat.selected.get() {
                        None => view! {
                            <div class="messages-empty">
                                <p class="text-muted">
                                    "Pick a conversation on the left, or start one from a user's profile."
                                </p>
                            </div>
                        }.into_any(),
                        Some(group_id) => {
                            // Pull the summary out of the cached groups list
                            // so the chat header has a real title.
                            let title = groups
                                .get()
                                .and_then(|res| {
                                    res.as_ref().ok().and_then(|list| {
                                        let my_id = auth.user.get().map(|u| u.id).unwrap_or_default();
                                        list.data
                                            .iter()
                                            .find(|g| g.id == group_id)
                                            .map(|g| conversation_title(g, &my_id))
                                    })
                                })
                                .unwrap_or_else(|| short_id(&group_id));
                            view! {
                                <ChatView
                                    group_id=group_id
                                    title=title
                                    mls=mls
                                    chat=chat
                                    auth=auth
                                />
                            }.into_any()
                        }
                    }
                }}
            </section>
        </div>
    }
}

#[component]
fn ChatView(
    group_id: String,
    title: String,
    mls: MlsState,
    chat: ChatState,
    auth: AuthState,
) -> impl IntoView {
    let group_id = StoredValue::new(group_id);
    let title = StoredValue::new(title);
    let current_user_id = auth.user.get_untracked().map(|u| u.id).unwrap_or_default();
    let current_user_id = StoredValue::new(current_user_id);
    let draft = chat.draft;
    let sending = RwSignal::new(false);

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if sending.get_untracked() {
            return;
        }
        let text = draft.get_untracked().trim().to_string();
        if text.is_empty() {
            return;
        }
        draft.set(String::new());

        let server_id = group_id.get_value();
        let mls_group_id_b64 = chat.group_map.get_untracked().get(&server_id).cloned();
        let Some(mls_group_id_b64) = mls_group_id_b64 else {
            chat.error.set("Unknown group.".to_string());
            return;
        };
        let Some(client) = mls.client_ref() else {
            chat.error.set("MLS client not ready yet.".to_string());
            return;
        };

        sending.set(true);
        chat.error.set(String::new());

        wasm_bindgen_futures::spawn_local(async move {
            let mls_group_id = match api::decode(&mls_group_id_b64) {
                Ok(v) => v,
                Err(e) => {
                    chat.error.set(format!("Bad group id: {}", e.message));
                    sending.set(false);
                    return;
                }
            };
            let me_for_send = current_user_id.get_value();
            let ciphertext = {
                let mut c = client.borrow_mut();
                let ct = match c.encrypt(&mls_group_id, text.as_bytes()) {
                    Ok(ct) => ct,
                    Err(e) => {
                        chat.error.set(format!("Encrypt failed: {e}"));
                        sending.set(false);
                        return;
                    }
                };
                // Remember the plaintext locally — MLS refuses to decrypt
                // our own sends, so the blob is the only record that
                // survives a refresh.
                c.remember_sent(&mls_group_id, me_for_send, text.clone());
                ct
            };
            match api::post_message(&server_id, &ciphertext).await {
                Ok(()) => {
                    // Optimistically append our own message locally — we
                    // can't `decrypt` our own sends on the same client.
                    let msg = LocalMessage {
                        server_id: 0, // unknown until poll catches up
                        sender_user_id: current_user_id.get_value(),
                        plaintext: text,
                        created_at: chrono::Utc::now().to_rfc3339(),
                    };
                    chat.messages.update(|map| {
                        map.entry(mls_group_id_b64.clone()).or_default().push(msg);
                    });
                    // Encrypting advanced the ratchet → save the new state.
                    persist_if_possible(mls).await;
                }
                Err(e) => {
                    chat.error.set(format!("Send failed: {}", e.message));
                }
            }
            sending.set(false);
        });
    };

    view! {
        <div class="chat-view">
            <header class="chat-header">
                {move || title.get_value()}
            </header>

            <div class="chat-messages">
                {move || {
                    let server_id = group_id.get_value();
                    let mls_b64 = chat.group_map.with(|m| m.get(&server_id).cloned());
                    let Some(mls_b64) = mls_b64 else {
                        return view! {
                            <p class="text-muted">"Preparing conversation…"</p>
                        }.into_any();
                    };
                    let msgs = chat.messages.with(|m| m.get(&mls_b64).cloned().unwrap_or_default());
                    if msgs.is_empty() {
                        view! {
                            <p class="text-muted chat-empty">
                                "No messages yet. Say hello."
                            </p>
                        }.into_any()
                    } else {
                        let me = current_user_id.get_value();
                        view! {
                            <ul class="chat-message-list">
                                {msgs.into_iter().map(|m| {
                                    let mine = m.sender_user_id == me;
                                    let class = if mine { "chat-msg chat-msg-mine" } else { "chat-msg" };
                                    let ts = format_timestamp(&m.created_at);
                                    view! {
                                        <li class=class>
                                            <Markdown content=m.plaintext class="chat-msg-text".to_string() />
                                            <span class="chat-msg-time">{ts}</span>
                                        </li>
                                    }
                                }).collect::<Vec<_>>()}
                            </ul>
                        }.into_any()
                    }
                }}
            </div>

            {move || {
                let err = chat.error.get();
                (!err.is_empty()).then(|| view! {
                    <p class="form-error chat-error">{err}</p>
                })
            }}

            <ComposerResizeHandle />
            <form class="chat-composer" on:submit=on_submit>
                <div
                    class="chat-composer-editor"
                    // Enter to send, Shift+Enter for newline (idiomatic).
                    on:keydown=move |ev: web_sys::KeyboardEvent| {
                        if ev.key() == "Enter" && !ev.shift_key() {
                            ev.prevent_default();
                            // Manually trigger the form submit path.
                            if !sending.get_untracked() && !draft.get_untracked().trim().is_empty() {
                                let text = draft.get_untracked().trim().to_string();
                                draft.set(String::new());
                                let server_id = group_id.get_value();
                                let mls_group_id_b64 = chat
                                    .group_map
                                    .get_untracked()
                                    .get(&server_id)
                                    .cloned();
                                let Some(mls_group_id_b64) = mls_group_id_b64 else {
                                    chat.error.set("Unknown group.".to_string());
                                    return;
                                };
                                let Some(client) = mls.client_ref() else {
                                    chat.error.set("MLS client not ready yet.".to_string());
                                    return;
                                };
                                sending.set(true);
                                chat.error.set(String::new());
                                wasm_bindgen_futures::spawn_local(async move {
                                    let mls_group_id = match api::decode(&mls_group_id_b64) {
                                        Ok(v) => v,
                                        Err(e) => {
                                            chat.error.set(format!("Bad group id: {}", e.message));
                                            sending.set(false);
                                            return;
                                        }
                                    };
                                    let me_for_send = current_user_id.get_value();
                                    let ciphertext = {
                                        let mut c = client.borrow_mut();
                                        let ct = match c.encrypt(&mls_group_id, text.as_bytes()) {
                                            Ok(ct) => ct,
                                            Err(e) => {
                                                chat.error.set(format!("Encrypt failed: {e}"));
                                                sending.set(false);
                                                return;
                                            }
                                        };
                                        c.remember_sent(&mls_group_id, me_for_send.clone(), text.clone());
                                        ct
                                    };
                                    match api::post_message(&server_id, &ciphertext).await {
                                        Ok(()) => {
                                            let msg = LocalMessage {
                                                server_id: 0,
                                                sender_user_id: me_for_send,
                                                plaintext: text,
                                                created_at: chrono::Utc::now().to_rfc3339(),
                                            };
                                            chat.messages.update(|map| {
                                                map.entry(mls_group_id_b64.clone()).or_default().push(msg);
                                            });
                                            persist_if_possible(mls).await;
                                        }
                                        Err(e) => {
                                            chat.error.set(format!("Send failed: {}", e.message));
                                        }
                                    }
                                    sending.set(false);
                                });
                            }
                        }
                    }
                >
                    <MarkdownEditor
                        value=draft
                        placeholder="Type an encrypted message… (Enter to send, Shift+Enter for newline)".to_string()
                        rows=3
                        input_class="chat-composer-input".to_string()
                    />
                </div>
                <button
                    class="btn btn-primary btn-sm"
                    type="submit"
                    disabled=move || sending.get()
                >
                    {move || if sending.get() { "Sending…" } else { "Send" }}
                </button>
            </form>
        </div>
    }
}

fn short_id(id: &str) -> String {
    if id.len() > 8 {
        format!("Conversation {}…", &id[..8])
    } else {
        format!("Conversation {id}")
    }
}

/// A display label for a conversation. Prefers:
///   1. Explicit group name
///   2. For 1:1: the other member's display name
///   3. For multi-member: comma-joined peer names (truncated)
///   4. Fallback: short server id
fn conversation_title(g: &api::GroupSummary, my_id: &str) -> String {
    if let Some(name) = &g.name
        && !name.is_empty()
        && name != "smoketest"
    {
        return name.clone();
    }
    let peers: Vec<&str> = g
        .members
        .iter()
        .filter(|m| m.user_id != my_id)
        .map(|m| m.display_name.as_str())
        .collect();
    match peers.len() {
        0 => short_id(&g.id),
        1 => peers[0].to_string(),
        n if n <= 3 => peers.join(", "),
        _ => format!("{} + {} more", peers[..2].join(", "), peers.len() - 2),
    }
}

fn spawn_welcome_accept_loop(
    mls: MlsState,
    chat: ChatState,
    groups: LocalResource<Result<api::GroupList, ApiError>>,
) {
    Effect::new(move |_| {
        // Bind reactively to mls.ready — once it's true we start polling.
        if !mls.ready.get() {
            return;
        }
        wasm_bindgen_futures::spawn_local(async move {
            // One-shot on page mount; then the poll loop below keeps going.
            if let Err(e) = accept_pending_welcomes(mls, chat).await {
                warn!("welcome accept pass: {}", e);
            }
            groups.refetch();
        });
    });

    // Background polling loop for both welcomes and messages. Runs forever
    // while the page is mounted; Leptos tears it down on unmount via the
    // Effect scope.
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(POLL_INTERVAL_MS).await;
                if !mls.ready.get_untracked() {
                    continue;
                }
                let before = mls.client_ref().map(|c| c.borrow().has_group(&[]));
                let _ = before; // silence unused
                if let Err(e) = accept_pending_welcomes(mls, chat).await {
                    warn!("welcome accept pass: {}", e);
                }
                groups.refetch();
            }
        });
    });
}

async fn accept_pending_welcomes(mls: MlsState, chat: ChatState) -> Result<(), ApiError> {
    let welcomes = api::list_welcomes().await?;
    if welcomes.data.is_empty() {
        return Ok(());
    }
    let Some(client) = mls.client_ref() else {
        return Ok(());
    };
    for envelope in welcomes.data {
        let ciphertext = match api::decode(&envelope.ciphertext_b64) {
            Ok(v) => v,
            Err(e) => {
                warn!("bad welcome b64: {}", e.message);
                continue;
            }
        };
        let joined_mls_id = {
            let mut c = client.borrow_mut();
            match c.accept_welcome(&ciphertext) {
                Ok(id) => id,
                Err(e) => {
                    warn!("accept_welcome failed: {e}");
                    chat.error.set(format!(
                        "Couldn't accept a conversation invite — {e}. Usually means the \
                         inviter used a stale KeyPackage; ask them to try again."
                    ));
                    // Ack the bad welcome so we don't spin on it forever.
                    let _ = api::ack_welcome(&envelope.id).await;
                    continue;
                }
            }
        };
        let _ = joined_mls_id;
        api::ack_welcome(&envelope.id).await?;
        mls.bump_revision();
        persist_if_possible(mls).await;
    }
    Ok(())
}

/// Best-effort auto-save of MLS state after a mutation. Silently logs failures
/// rather than surfacing them — persistence is a background concern and the
/// user just succeeded at the actual operation.
async fn persist_if_possible(mls: MlsState) {
    let (Some(client), Some(keys)) = (mls.client_ref(), mls.keys_ref()) else {
        return;
    };
    let snapshot = {
        let c = client.borrow();
        match c.to_serialized() {
            Ok(s) => s,
            Err(e) => {
                warn!("persist: serialize failed: {e}");
                return;
            }
        }
    };
    let plaintext = match serde_json::to_vec(&snapshot) {
        Ok(p) => p,
        Err(e) => {
            warn!("persist: json failed: {e}");
            return;
        }
    };
    let wrapped = match crate::mls::crypto::seal(&keys.wrap_key, &plaintext) {
        Ok(w) => w,
        Err(e) => {
            warn!("persist: seal failed: {e}");
            return;
        }
    };
    use base64::{Engine, engine::general_purpose::STANDARD};
    let body = serde_json::json!({
        "verifier_b64": STANDARD.encode(keys.verifier),
        "wrapped_blob_b64": STANDARD.encode(wrapped),
    });
    if let Err(e) = crate::api::client::put::<serde_json::Value, serde_json::Value>(
        "/api/v1/mls/keystore",
        &body,
    )
    .await
    {
        warn!("persist: PUT failed ({}): {}", e.status, e.message);
    }
}

fn spawn_message_poll_loop(mls: MlsState, chat: ChatState) {
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(POLL_INTERVAL_MS).await;
                let Some(selected) = chat.selected.get_untracked() else {
                    continue;
                };
                let mls_b64 = chat.group_map.with_untracked(|m| m.get(&selected).cloned());
                let Some(mls_b64) = mls_b64 else {
                    continue;
                };
                let Some(client) = mls.client_ref() else {
                    continue;
                };

                let mls_group_id = match api::decode(&mls_b64) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                // Cursor lives on the MlsClient so it's persisted across
                // refreshes — that's what prevents SecretReuseError when we
                // would otherwise re-feed already-consumed ciphertexts.
                let cursor = client.borrow().cursor(&mls_group_id);
                let result = match api::list_messages(&selected, cursor).await {
                    Ok(r) => r,
                    Err(e) => {
                        chat.error.set(format!("Poll failed: {}", e.message));
                        continue;
                    }
                };
                if result.data.is_empty() {
                    continue;
                }

                let mut highest = cursor;
                let mut new_msgs: Vec<LocalMessage> = Vec::new();
                {
                    let mut c = client.borrow_mut();
                    for envelope in result.data {
                        highest = highest.max(envelope.id);
                        let ciphertext = match api::decode(&envelope.ciphertext_b64) {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        match c.decrypt(&mls_group_id, &ciphertext) {
                            Ok(Some(plaintext)) => {
                                let text = String::from_utf8(plaintext).unwrap_or_default();
                                c.remember_received(
                                    &mls_group_id,
                                    envelope.sender_user_id.clone(),
                                    text.clone(),
                                    envelope.id,
                                    envelope.created_at.clone(),
                                );
                                new_msgs.push(LocalMessage {
                                    server_id: envelope.id,
                                    sender_user_id: envelope.sender_user_id,
                                    plaintext: text,
                                    created_at: envelope.created_at,
                                });
                            }
                            Ok(None) => { /* protocol message, silently applied */ }
                            Err(
                                crate::mls::client::MlsClientError::OwnMessage
                                | crate::mls::client::MlsClientError::AlreadyProcessed,
                            ) => {
                                // Expected: own message, or we already decrypted
                                // this in a prior session. Plaintext is already
                                // in the persisted store.
                            }
                            Err(e) => {
                                warn!("decrypt failed (msg {}): {e}", envelope.id);
                                chat.error.set(format!(
                                    "Couldn't decrypt message #{} — {e}",
                                    envelope.id
                                ));
                            }
                        }
                    }
                    c.set_cursor(&mls_group_id, highest);
                }

                let received_any = !new_msgs.is_empty();
                if received_any {
                    let mls_b64_key = mls_b64.clone();
                    chat.messages.update(|map| {
                        let bucket = map.entry(mls_b64_key).or_default();
                        for m in new_msgs {
                            bucket.push(m);
                        }
                    });
                }
                // Persist every tick — cheap, and captures cursor advances
                // even when we didn't decrypt anything new this round.
                persist_if_possible(mls).await;
            }
        });
    });
}

// Force an import keep-alive — we rely on this type in a few places via the
// reactive signal path.
#[allow(dead_code)]
fn _keep_alive(_: MlsClientRef) {}
