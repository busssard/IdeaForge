use leptos::prelude::*;
use leptos_router::components::A;

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
                // Simple checklist - these are tracked client-side
                let steps_done = if profile_done { 1 } else { 0 };
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
                            <li>
                                <span class="step-check">{"\u{2B1C}"}</span>
                                <A href="/ideas/new" attr:class="step-link">"Create your first idea"</A>
                            </li>
                            <li>
                                <span class="step-check">{"\u{2B1C}"}</span>
                                <A href="/browse" attr:class="step-link">"Stoke an idea you like"</A>
                            </li>
                            <li>
                                <span class="step-check">{"\u{2B1C}"}</span>
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
