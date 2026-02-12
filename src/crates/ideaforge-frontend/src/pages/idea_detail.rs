use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use leptos_router::components::A;

use crate::api;
use crate::components::comment_section::CommentSection;
use crate::components::loading::Loading;
use crate::components::maturity_badge::MaturityBadge;
use crate::components::stoke_button::StokeButton;
use crate::components::subscribe_button::SubscribeButton;
use crate::components::suggestion_section::SuggestionSection;
use crate::components::team_panel::TeamPanel;

#[component]
pub fn IdeaDetailPage() -> impl IntoView {
    let params = use_params_map();

    let idea = LocalResource::new(move || {
        let id = params.get().get("id").unwrap_or_default();
        async move { api::ideas::get_idea(&id).await }
    });

    view! {
        <Suspense fallback=move || view! { <Loading /> }>
            {move || {
                idea.get().map(|result| {
                    match &*result {
                        Ok(idea) => {
                            let openness_class = match idea.openness.as_str() {
                                "open" => "badge badge-open",
                                "collaborative" => "badge badge-collaborative",
                                "commercial" => "badge badge-commercial",
                                _ => "badge",
                            };
                            let openness_label = idea.openness.clone();
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
                            let idea_id_comments = idea.id.clone();
                            let idea_id_suggestions = idea.id.clone();
                            let idea_id_team = idea.id.clone();
                            let idea_id_subscribe = idea.id.clone();
                            let author_id_team = idea.author_id.clone();
                            let has_stoked = idea.has_stoked.unwrap_or(false);

                            view! {
                                <div class="idea-detail">
                                    <div class="idea-detail-header">
                                        <h1 class="idea-detail-title">{idea.title.clone()}</h1>
                                        <div class="idea-detail-badges">
                                            <MaturityBadge maturity=idea.maturity.clone() />
                                            <span class=openness_class>{openness_label}</span>
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

                                    <div class="idea-detail-body">{idea.description.clone()}</div>

                                    <div class="idea-detail-actions">
                                        <StokeButton
                                            idea_id=idea_id.clone()
                                            initial_count=idea.stoke_count
                                            initial_stoked=has_stoked
                                        />
                                        <SubscribeButton idea_id=idea_id_subscribe />
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

                                    // Comments section
                                    <div class="card mb-lg">
                                        <CommentSection idea_id=idea_id_comments />
                                    </div>

                                    // Suggestions section
                                    <div class="card mb-lg">
                                        <SuggestionSection idea_id=idea_id_suggestions />
                                    </div>
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
