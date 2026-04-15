use leptos::prelude::*;

use crate::api::stokes;
use crate::state::auth::AuthState;

#[component]
pub fn StokeButton(
    idea_id: String,
    initial_count: i32,
    #[prop(default = false)] initial_stoked: bool,
    #[prop(default = false)] prominent: bool,
) -> impl IntoView {
    let count = RwSignal::new(initial_count);
    let stoked = RwSignal::new(initial_stoked);
    let loading = RwSignal::new(false);
    let auth = expect_context::<AuthState>();

    let idea_id = StoredValue::new(idea_id);

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
        let was_stoked = stoked.get_untracked();
        let id = idea_id.get_value();

        // Optimistic update
        if was_stoked {
            stoked.set(false);
            count.update(|c| *c -= 1);
        } else {
            stoked.set(true);
            count.update(|c| *c += 1);
        }

        wasm_bindgen_futures::spawn_local(async move {
            let result = if was_stoked {
                stokes::withdraw_stoke(&id).await.map(|_| ())
            } else {
                stokes::stoke_idea(&id).await.map(|_| ())
            };

            if let Err(e) = result {
                // Revert on error
                if was_stoked {
                    stoked.set(true);
                    count.update(|c| *c += 1);
                } else {
                    stoked.set(false);
                    count.update(|c| *c -= 1);
                }
                if e.status == 401 {
                    // Refresh failed too — drop the stale session and send
                    // them to /login rather than silently failing.
                    auth.logout();
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/login");
                    }
                }
            }

            loading.set(false);
        });
    };

    let base_class = if prominent { "stoke-btn stoke-btn-prominent" } else { "stoke-btn" };

    view! {
        <button
            class=move || if stoked.get() { format!("{base_class} stoked") } else { base_class.to_string() }
            on:click=toggle
            disabled=move || loading.get()
            title=move || {
                if !auth.is_authenticated() { "Log in to spark this idea".to_string() }
                else if stoked.get() { "You sparked this — click to withdraw".to_string() }
                else { "Spark this idea".to_string() }
            }
        >
            <span class="flame">{move || if stoked.get() { "\u{1F525}" } else { "\u{1FAB5}" }}</span>
            <span class="stoke-btn-count">{move || count.get().to_string()}</span>
            {prominent.then(|| view! {
                <span class="stoke-btn-label">
                    {move || if stoked.get() { "Sparked" } else { "Spark this idea" }}
                </span>
            })}
        </button>
    }
}
