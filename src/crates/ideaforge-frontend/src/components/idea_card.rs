use leptos::prelude::*;
use leptos_router::components::A;

use crate::api::types::IdeaResponse;
use crate::components::maturity_badge::MaturityBadge;
use crate::components::stoke_button::StokeButton;

#[component]
pub fn IdeaCard(idea: IdeaResponse) -> impl IntoView {
    let id = idea.id.clone();
    let title = idea.title.clone();
    let summary = idea.summary.clone();
    let maturity = idea.maturity.clone();
    let stoke_count = idea.stoke_count;
    let idea_id_for_stoke = idea.id.clone();
    let created_date = idea.created_at.split('T').next().unwrap_or("").to_string();
    let has_stoked = idea.has_stoked.unwrap_or(false);

    view! {
        <A href=format!("/ideas/{id}") attr:class="card card-clickable idea-card fade-in" attr:style="text-decoration: none; color: inherit; display: block;">
            <div class="idea-card-header">
                <h3 class="idea-card-title">{title}</h3>
                <MaturityBadge maturity=maturity />
            </div>
            <p class="idea-card-summary">{summary}</p>
            <div class="idea-card-footer" on:click=|ev: web_sys::MouseEvent| { ev.prevent_default(); ev.stop_propagation(); }>
                <StokeButton
                    idea_id=idea_id_for_stoke
                    initial_count=stoke_count
                    initial_stoked=has_stoked
                />
                <span class="idea-card-meta">{created_date}</span>
            </div>
        </A>
    }
}
