//! Per-session MLS state. The `MlsClient` handle only becomes `Some` after
//! the user unlocks (or sets up) their keystore with their 6-digit PIN.
//!
//! `wrap_key` is cached here too — after a successful unlock we hold the
//! Argon2id-derived bytes in memory so state-changing operations can
//! re-wrap + persist to the server without a second PIN prompt.

use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use leptos::reactive::owner::LocalStorage;

use crate::mls::client::MlsClient;
use crate::mls::crypto::DerivedKeys;

pub type MlsClientRef = Rc<RefCell<MlsClient>>;

#[derive(Copy, Clone)]
pub struct MlsState {
    pub client: StoredValue<Option<MlsClientRef>, LocalStorage>,
    /// Argon2id-derived wrap_key + verifier. Kept in memory until logout so
    /// `persist()` can re-wrap MLS state after every send/receive/commit.
    pub keys: StoredValue<Option<DerivedKeys>, LocalStorage>,
    /// True once KeyPackages for the unlocked client have been published.
    pub ready: RwSignal<bool>,
    pub init_error: RwSignal<String>,
    pub revision: RwSignal<u64>,
    /// Set when "Message privately" wants the messages page to open a
    /// specific conversation after unlock.
    pub pending_selection: RwSignal<Option<String>>,
}

impl MlsState {
    pub fn new() -> Self {
        Self {
            client: StoredValue::new_local(None),
            keys: StoredValue::new_local(None),
            ready: RwSignal::new(false),
            init_error: RwSignal::new(String::new()),
            revision: RwSignal::new(0),
            pending_selection: RwSignal::new(None),
        }
    }

    pub fn client_ref(&self) -> Option<MlsClientRef> {
        self.client.get_value()
    }

    pub fn keys_ref(&self) -> Option<DerivedKeys> {
        self.keys.get_value()
    }

    /// Install a freshly unlocked / set-up client. The caller has already
    /// derived the keys (Argon2id); we cache them so subsequent `persist`
    /// calls don't re-prompt.
    pub fn set_client(&self, client: MlsClient, keys: DerivedKeys) {
        self.client.set_value(Some(Rc::new(RefCell::new(client))));
        self.keys.set_value(Some(keys));
        self.init_error.set(String::new());
    }

    /// Clear everything — used on logout or PIN-fail-reset.
    pub fn clear(&self) {
        self.client.set_value(None);
        self.keys.set_value(None);
        self.ready.set(false);
        self.init_error.set(String::new());
    }

    pub fn bump_revision(&self) {
        self.revision.update(|v| *v = v.wrapping_add(1));
    }
}

impl Default for MlsState {
    fn default() -> Self {
        Self::new()
    }
}
