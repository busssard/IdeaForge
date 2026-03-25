use leptos::prelude::*;

use crate::api::stokes;
use crate::state::auth::AuthState;

#[component]
pub fn StokeButton(
    idea_id: String,
    initial_count: i32,
    #[prop(default = false)] initial_stoked: bool,
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

            if result.is_err() {
                // Revert on error
                if was_stoked {
                    stoked.set(true);
                    count.update(|c| *c += 1);
                } else {
                    stoked.set(false);
                    count.update(|c| *c -= 1);
                }
            }

            loading.set(false);
        });
    };

    view! {
        <button
            class=move || if stoked.get() { "stoke-btn stoked" } else { "stoke-btn" }
            on:click=toggle
            disabled=move || loading.get()
        >
            <span class="flame">{move || if stoked.get() { "\u{1F525}" } else { "\u{1FAB5}" }}</span>
            <span>{move || count.get().to_string()}</span>
        </button>
    }
}
