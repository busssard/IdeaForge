use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use leptos_router::components::A;

use crate::api;
use crate::components::bot_section::BotSection;
use crate::components::comment_section::CommentSection;
use crate::components::loading::Loading;
use crate::components::maturity_badge::MaturityBadge;
use crate::components::nda_wall::NdaWall;
use crate::components::share_buttons::ShareButtons;
use crate::components::stoke_button::StokeButton;
use crate::components::subscribe_button::SubscribeButton;
use crate::components::suggestion_section::SuggestionSection;
use crate::components::task_board::TaskBoard;
use crate::components::team_panel::TeamPanel;
use crate::components::visibility_badge::VisibilityBadge;

#[component]
pub fn IdeaDetailPage() -> impl IntoView {
    let params = use_params_map();
    let refetch_trigger = RwSignal::new(0u32);

    let idea = LocalResource::new(move || {
        let _ = refetch_trigger.get(); // subscribe to trigger for refetch
        let id = params.get().get("id").unwrap_or_default();
        async move { api::ideas::get_idea(&id).await }
    });

    view! {
        <Suspense fallback=move || view! { <Loading /> }>
            {move || {
                idea.get().map(|result| {
                    match &*result {
                        Ok(idea) => {
                            let openness_for_badge = idea.openness.clone();
                            let created_date = idea
                                .created_at
                                .split('T')
                                .next()
                                .unwrap_or("")
                                .to_string();
                            let updated_date = idea
                                .updated_at
                                .split('T')
                                .next()
                                .unwrap_or("")
                                .to_string();
                            let author_id = idea.author_id.clone();
                            let idea_id = idea.id.clone();
                            let idea_id_nda = idea.id.clone();
                            let idea_id_comments = idea.id.clone();
                            let idea_id_suggestions = idea.id.clone();
                            let idea_id_team = idea.id.clone();
                            let idea_id_subscribe = idea.id.clone();
                            let idea_id_bots = idea.id.clone();
                            let idea_id_board = idea.id.clone();
                            let author_id_team = idea.author_id.clone();
                            let author_id_board = idea.author_id.clone();
                            let has_stoked = idea.has_stoked.unwrap_or(false);
                            let share_url = format!("{}/ideas/{}", get_base_url(), idea.id);
                            let share_title = idea.title.clone();
                            let share_summary = idea.summary.clone();

                            // Determine if this idea is NDA-protected and not yet signed
                            let is_nda_protected = idea.openness == "nda_protected"
                                || idea.nda_required.unwrap_or(false);
                            let nda_signed = idea.nda_signed.unwrap_or(false);
                            let show_nda_wall = is_nda_protected
                                && !nda_signed
                                && idea.description.contains("[NDA Required]");

                            view! {
                                <div class="idea-detail">
                                    <div class="idea-detail-header">
                                        <h1 class="idea-detail-title">{idea.title.clone()}</h1>
                                        <div class="idea-detail-badges">
                                            <MaturityBadge maturity=idea.maturity.clone() />
                                            <VisibilityBadge openness=openness_for_badge />
                                        </div>
                                        <div class="idea-detail-meta">
                                            <span>
                                                "by "
                                                <A href=format!(
                                                    "/profile/{author_id}",
                                                )>"view author"</A>
                                            </span>
                                            <span>"Created " {created_date}</span>
                                            <span>"Updated " {updated_date}</span>
                                        </div>
                                    </div>

                                    <div class="card mb-lg">
                                        <h4>"Summary"</h4>
                                        <p class="mt-sm" style="color: var(--text-secondary)">
                                            {idea.summary.clone()}
                                        </p>
                                    </div>

                                    {if show_nda_wall {
                                        view! {
                                            <NdaWall
                                                idea_id=idea_id_nda
                                                on_signed=Callback::new(move |_| {
                                                    refetch_trigger
                                                        .set(refetch_trigger.get_untracked() + 1);
                                                })
                                            />
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <div class="idea-detail-body">
                                                {idea.description.clone()}
                                            </div>

                                            <div class="idea-detail-actions">
                                                <StokeButton
                                                    idea_id=idea_id.clone()
                                                    initial_count=idea.stoke_count
                                                    initial_stoked=has_stoked
                                                />
                                                <SubscribeButton idea_id=idea_id_subscribe />
                                                <ShareButtons
                                                    url=share_url
                                                    title=share_title
                                                    summary=share_summary
                                                />
                                                <A href="/browse" attr:class="btn btn-ghost">
                                                    "Back to Forge Floor"
                                                </A>
                                            </div>

                                            // Team section
                                            <div class="card mb-lg">
                                                <TeamPanel
                                                    idea_id=idea_id_team
                                                    author_id=author_id_team
                                                />
                                            </div>

                                            // Task board section
                                            <div class="card mb-lg">
                                                <TaskBoard
                                                    idea_id=idea_id_board
                                                    author_id=author_id_board
                                                />
                                            </div>

                                            // Comments section
                                            <div class="card mb-lg">
                                                <CommentSection idea_id=idea_id_comments />
                                            </div>

                                            // Suggestions section
                                            <div class="card mb-lg">
                                                <SuggestionSection idea_id=idea_id_suggestions />
                                            </div>

                                            // Bot / AI activity section (parallel track)
                                            <BotSection idea_id=idea_id_bots />
                                        }
                                            .into_any()
                                    }}
                                </div>
                            }
                                .into_any()
                        }
                        Err(e) => {
                            if e.status == 404 {
                                view! {
                                    <div class="empty-state">
                                        <h3>"Idea not found"</h3>
                                        <p>"This idea may have been archived or doesn't exist."</p>
                                        <A href="/browse" attr:class="btn btn-secondary">
                                            "Back to Forge Floor"
                                        </A>
                                    </div>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <div class="error-display">
                                        <h3>"Failed to load idea"</h3>
                                        <p>{e.message.clone()}</p>
                                    </div>
                                }
                                    .into_any()
                            }
                        }
                    }
                })
            }}
        </Suspense>
    }
}

fn get_base_url() -> String {
    web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_else(|| "https://ideaforge.io".to_string())
}
