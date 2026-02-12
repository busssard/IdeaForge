use leptos::prelude::*;

use crate::api::client;
use crate::api::types::UserResponse;

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub role: String,
}

#[derive(Debug, Clone, Copy)]
pub struct AuthState {
    pub user: RwSignal<Option<CurrentUser>>,
    /// True while we're checking a stored token on startup.
    pub loading: RwSignal<bool>,
}

impl AuthState {
    pub fn new() -> Self {
        let user = RwSignal::new(None);
        let has_token = client::get_token().is_some();
        let loading = RwSignal::new(has_token);

        let state = Self { user, loading };

        // If we have a token, try to load the user
        if has_token {
            wasm_bindgen_futures::spawn_local(async move {
                state.load_user().await;
                state.loading.set(false);
            });
        }

        state
    }

    pub fn is_authenticated(&self) -> bool {
        self.user.get().is_some()
    }

    pub fn set_authenticated(&self, user_resp: &UserResponse) {
        self.user.set(Some(CurrentUser {
            id: user_resp.id.clone(),
            email: user_resp.email.clone(),
            display_name: user_resp.display_name.clone(),
            role: user_resp.role.clone(),
        }));
    }

    pub fn set_from_token_response(&self, user_id: &str) {
        // Minimal auth state from login/register — we'll load full user on next navigation
        self.user.set(Some(CurrentUser {
            id: user_id.to_string(),
            email: String::new(),
            display_name: String::new(),
            role: String::new(),
        }));
    }

    pub fn logout(&self) {
        client::clear_tokens();
        self.user.set(None);
    }

    pub async fn load_user(&self) {
        match crate::api::users::get_me().await {
            Ok(user) => {
                self.set_authenticated(&user);
            }
            Err(e) => {
                if e.status == 401 {
                    // Try refresh
                    match crate::api::auth::refresh().await {
                        Ok(_) => {
                            // Retry get_me with new token
                            if let Ok(user) = crate::api::users::get_me().await {
                                self.set_authenticated(&user);
                            } else {
                                client::clear_tokens();
                                self.user.set(None);
                            }
                        }
                        Err(_) => {
                            client::clear_tokens();
                            self.user.set(None);
                        }
                    }
                } else {
                    client::clear_tokens();
                    self.user.set(None);
                }
            }
        }
    }
}
