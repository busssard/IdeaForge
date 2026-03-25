use leptos::prelude::*;

use crate::api;
use crate::api::types::BotProfileResponse;
use crate::components::loading::Loading;

/// Section showing bot endorsements and AI insights on an idea page.
/// Clearly separated from human interaction.
#[component]
pub fn BotSection(idea_id: String) -> impl IntoView {
    let collapsed = RwSignal::new(true);

    let bots = LocalResource::new(move || async move {
        api::bots::list_bots().await
    });

    view! {
        <div class="bot-section">
            <button
                class="bot-section-toggle"
                on:click=move |_| collapsed.set(!collapsed.get_untracked())
            >
                <span class="bot-section-icon">"\u{1F916}"</span>
                <span class="bot-section-title">"AI & Bot Activity"</span>
                <span class="bot-section-arrow">
                    {move || if collapsed.get() { "\u{25B6}" } else { "\u{25BC}" }}
                </span>
            </button>

            {move || {
                if collapsed.get() {
                    return view! { <div></div> }.into_any();
                }

                view! {
                    <div class="bot-section-content">
                        <p class="bot-section-info">
                            "Bot interactions are clearly separated from human activity. "
                            "Bot endorsements do not count toward stoke totals."
                        </p>

                        <Suspense fallback=move || view! { <Loading /> }>
                            {move || {
                                bots.get().map(|result| {
                                    match &*result {
                                        Ok(resp) => {
                                            if resp.data.is_empty() {
                                                view! {
                                                    <p class="text-muted">"No bots have analyzed this idea yet."</p>
                                                }.into_any()
                                            } else {
                                                let items: Vec<BotProfileResponse> = resp.data.clone();
                                                view! {
                                                    <div class="bot-list">
                                                        {items.into_iter().map(|bot| {
                                                            view! { <BotCard bot=bot /> }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                }.into_any()
                                            }
                                        }
                                        Err(_) => {
                                            view! {
                                                <p class="text-muted">"Could not load bot activity."</p>
                                            }.into_any()
                                        }
                                    }
                                })
                            }}
                        </Suspense>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn BotCard(bot: BotProfileResponse) -> impl IntoView {
    let name = bot.username.clone();
    let operator = bot.operator.unwrap_or_default();
    let description = bot.description.unwrap_or_default();

    view! {
        <div class="bot-card">
            <div class="bot-card-header">
                <span class="bot-badge">"\u{1F916}"</span>
                <div>
                    <span class="bot-card-name">{name}</span>
                    {(!operator.is_empty()).then(|| view! {
                        <span class="bot-card-operator">{format!("by {}", operator)}</span>
                    })}
                </div>
            </div>
            {(!description.is_empty()).then(|| view! {
                <p class="bot-card-desc">{description}</p>
            })}
        </div>
    }
}
