use leptos::prelude::*;
use leptos_router::components::A;

use crate::api;
use crate::api::types::IdeaResponse;
use crate::components::idea_card::IdeaCard;
use crate::components::loading::Loading;

#[component]
pub fn HomePage() -> impl IntoView {
    let ideas = LocalResource::new(move || async move {
        api::ideas::list_ideas(1, 6, None, None, None).await
    });

    view! {
        <section class="hero">
            <h1 class="hero-title">"Where Ideas Take Shape"</h1>
            <p class="hero-subtitle">
                "IdeaForge connects entrepreneurs, makers, investors, and creatives "
                "to transform raw ideas into reality. Bring your spark — we'll help you forge it."
            </p>
            <div class="hero-actions">
                <A href="/browse" attr:class="btn btn-primary btn-lg">"Explore the Forge Floor"</A>
                <A href="/register" attr:class="btn btn-secondary btn-lg">"Join the Forge"</A>
            </div>
        </section>

        <section class="dashboard-section">
            <h2>"Fresh from the Forge"</h2>
            <Suspense fallback=move || view! { <Loading /> }>
                {move || {
                    ideas.get().map(|result| {
                        match &*result {
                            Ok(resp) => {
                                if resp.data.is_empty() {
                                    view! {
                                        <div class="empty-state">
                                            <h3>"The forge is quiet"</h3>
                                            <p>"Be the first to bring an idea."</p>
                                            <A href="/ideas/new" attr:class="btn btn-primary">"Bring to the Forge"</A>
                                        </div>
                                    }.into_any()
                                } else {
                                    let items: Vec<IdeaResponse> = resp.data.clone();
                                    view! {
                                        <div class="ideas-grid">
                                            {items.into_iter().map(|idea| {
                                                view! { <IdeaCard idea=idea /> }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                        <div class="text-center mt-lg">
                                            <A href="/browse" attr:class="btn btn-secondary">"View all ideas"</A>
                                        </div>
                                    }.into_any()
                                }
                            }
                            Err(e) => {
                                view! {
                                    <div class="error-display">
                                        <p>{e.message.clone()}</p>
                                    </div>
                                }.into_any()
                            }
                        }
                    })
                }}
            </Suspense>
        </section>
    }
}
