use leptos::prelude::*;
use leptos_router::components::A;

use crate::api::types::IdeaResponse;
use crate::components::maturity_badge::MaturityBadge;
use crate::components::visibility_badge::VisibilityBadge;
use crate::state::auth::AuthState;

#[component]
pub fn IdeaCard(idea: IdeaResponse) -> impl IntoView {
    let id = idea.id.clone();
    let title = idea.title.clone();
    let summary = idea.summary.clone();
    let maturity = idea.maturity.clone();
    let openness = idea.openness.clone();
    let stoke_count = idea.stoke_count;
    let created_date = idea.created_at.split('T').next().unwrap_or("").to_string();
    let has_stoked = idea.has_stoked.unwrap_or(false);
    let show_visibility = openness != "open"; // only show badge for non-default visibility
    let author_name = idea.author_name.clone().unwrap_or_else(|| "Unknown".to_string());

    // Check if the current user is the author of this idea
    let auth = expect_context::<AuthState>();
    let is_own_idea = auth
        .user
        .get_untracked()
        .map_or(false, |u| u.id == idea.author_id);
    let card_class = if is_own_idea {
        "card card-clickable idea-card idea-card-own fade-in"
    } else {
        "card card-clickable idea-card fade-in"
    };

    view! {
        <A href=format!("/ideas/{id}") attr:class=card_class attr:style="text-decoration: none; color: inherit; display: block;">
            <div class="idea-card-header">
                <h3 class="idea-card-title">{title}</h3>
                <div class="idea-card-badges">
                    <MaturityBadge maturity=maturity />
                    {show_visibility.then(|| view! { <VisibilityBadge openness=openness /> })}
                </div>
            </div>
            <span class="idea-card-author">"by " {author_name}</span>
            <p class="idea-card-summary">{summary}</p>
            <div class="idea-card-footer">
                <span
                    class=if has_stoked { "idea-card-sparks stoked" } else { "idea-card-sparks" }
                    title="Open the idea to spark it"
                >
                    <span class="flame">{if has_stoked { "\u{1F525}" } else { "\u{1FAB5}" }}</span>
                    <span>{stoke_count.to_string()}</span>
                </span>
                <span class="idea-card-meta">{created_date}</span>
            </div>
        </A>
    }
}
