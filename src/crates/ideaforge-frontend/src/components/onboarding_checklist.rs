use leptos::prelude::*;
use leptos_router::components::A;

use crate::api;
use crate::state::auth::AuthState;

/// Onboarding checklist banner for new users.
/// Shows progress through first-time actions. Dismissible via localStorage.
#[component]
pub fn OnboardingChecklist() -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let dismissed = RwSignal::new(is_dismissed());

    // Track completion of onboarding steps
    let has_profile = Signal::derive(move || {
        auth.user.get().map_or(false, |u| !u.display_name.is_empty())
    });

    // Track whether user has created at least one idea
    let has_idea = RwSignal::new(false);
    // Track whether user has stoked at least one idea
    let has_stoke = RwSignal::new(false);
    // Track whether user has visited the people page (via localStorage)
    let has_visited_people = RwSignal::new(check_visited_people());

    // Fetch idea count and stoke count on mount
    {
        let user_signal = auth.user;
        wasm_bindgen_futures::spawn_local(async move {
            let user_id = user_signal.get_untracked().map(|u| u.id).unwrap_or_default();
            if user_id.is_empty() {
                return;
            }

            // Check if user has created any ideas
            if let Ok(resp) = api::ideas::list_ideas(1, 1, None, None, None, Some(&user_id)).await {
                has_idea.set(resp.meta.total > 0);
            }

            // Check if user has stoked any ideas
            if let Ok(resp) = api::ideas::list_my_stoked_ideas(1, 1).await {
                has_stoke.set(resp.meta.total > 0);
            }
        });
    }

    let dismiss = move |_: web_sys::MouseEvent| {
        set_dismissed();
        dismissed.set(true);
    };

    view! {
        {move || {
            let is_authenticated = auth.user.get().is_some();
            let is_dismissed = dismissed.get();

            if !is_authenticated || is_dismissed {
                view! { <div></div> }.into_any()
            } else {
                let profile_done = has_profile.get();
                let idea_done = has_idea.get();
                let stoke_done = has_stoke.get();
                let people_done = has_visited_people.get();

                let steps_done = [profile_done, idea_done, stoke_done, people_done]
                    .iter()
                    .filter(|&&v| v)
                    .count();
                let total_steps = 4;
                let progress_pct = (steps_done as f32 / total_steps as f32 * 100.0) as u32;

                view! {
                    <div class="onboarding-banner">
                        <div class="onboarding-header">
                            <h3 class="onboarding-title">"Welcome to the Forge!"</h3>
                            <button class="onboarding-dismiss" on:click=dismiss title="Dismiss">
                                <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                                    <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
                                </svg>
                            </button>
                        </div>

                        <div class="onboarding-progress">
                            <div class="onboarding-progress-bar" style=format!("width: {}%", progress_pct)></div>
                        </div>
                        <span class="onboarding-progress-text">{format!("{} of {} complete", steps_done, total_steps)}</span>

                        <ul class="onboarding-steps">
                            <li class=if profile_done { "step-done" } else { "" }>
                                <span class="step-check">{if profile_done { "\u{2705}" } else { "\u{2B1C}" }}</span>
                                <A href="/settings" attr:class="step-link">"Complete your profile"</A>
                            </li>
                            <li class=if idea_done { "step-done" } else { "" }>
                                <span class="step-check">{if idea_done { "\u{2705}" } else { "\u{2B1C}" }}</span>
                                <A href="/ideas/new" attr:class="step-link">"Create your first idea"</A>
                            </li>
                            <li class=if stoke_done { "step-done" } else { "" }>
                                <span class="step-check">{if stoke_done { "\u{2705}" } else { "\u{2B1C}" }}</span>
                                <A href="/browse" attr:class="step-link">"Stoke an idea you like"</A>
                            </li>
                            <li class=if people_done { "step-done" } else { "" }>
                                <span class="step-check">{if people_done { "\u{2705}" } else { "\u{2B1C}" }}</span>
                                <A href="/people" attr:class="step-link">"Discover collaborators"</A>
                            </li>
                        </ul>
                    </div>
                }.into_any()
            }
        }}
    }
}

fn is_dismissed() -> bool {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("ideaforge_onboarding_dismissed").ok().flatten())
        .map_or(false, |v| v == "true")
}

fn set_dismissed() {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
    {
        let _ = storage.set_item("ideaforge_onboarding_dismissed", "true");
    }
}

fn check_visited_people() -> bool {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("ideaforge_visited_people").ok().flatten())
        .map_or(false, |v| v == "true")
}
