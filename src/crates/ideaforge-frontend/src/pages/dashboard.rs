use leptos::prelude::*;
use leptos_router::components::A;

use crate::api;
use crate::components::idea_card::IdeaCard;
use crate::components::loading::Loading;
use crate::components::protected::Protected;
use crate::state::auth::AuthState;

#[component]
pub fn DashboardPage() -> impl IntoView {
    view! {
        <Protected>
            <DashboardContent />
        </Protected>
    }
}

#[component]
fn DashboardContent() -> impl IntoView {
    let _auth = expect_context::<AuthState>();

    let my_ideas = LocalResource::new(move || async move {
        api::ideas::list_ideas(1, 20, None, None, None).await
    });

    view! {
        <div class="page dashboard">
            <div class="page-header">
                <h1 class="page-title">"My Dashboard"</h1>
                <A href="/ideas/new" attr:class="btn btn-primary">"New Idea"</A>
            </div>

            <section class="dashboard-section">
                <h2>"My Ideas"</h2>
                <Suspense fallback=move || view! { <Loading /> }>
                    {move || {
                        my_ideas.get().map(|result| {
                            match &*result {
                                Ok(resp) => {
                                    if resp.data.is_empty() {
                                        view! {
                                            <div class="empty-state">
                                                <h3>"No ideas yet"</h3>
                                                <p>"Your forge is empty. Bring your first idea!"</p>
                                                <A href="/ideas/new" attr:class="btn btn-primary">"Bring to the Forge"</A>
                                            </div>
                                        }.into_any()
                                    } else {
                                        let items = resp.data.clone();
                                        view! {
                                            <div class="ideas-grid">
                                                {items.into_iter().map(|idea| {
                                                    view! { <IdeaCard idea=idea /> }
                                                }).collect::<Vec<_>>()}
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
        </div>
    }
}
