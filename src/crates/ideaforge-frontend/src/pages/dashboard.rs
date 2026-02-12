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
    let active_tab = RwSignal::new("ideas".to_string());

    // Load my ideas
    let my_ideas = LocalResource::new(move || async move {
        api::ideas::list_ideas(1, 20, None, None, None).await
    });

    // Load my stoked ideas
    let my_stokes = LocalResource::new(move || async move {
        api::ideas::list_my_stoked_ideas(1, 20).await
    });

    view! {
        <div class="page dashboard">
            <div class="page-header">
                <h1 class="page-title">"My Dashboard"</h1>
                <A href="/ideas/new" attr:class="btn btn-primary">"New Idea"</A>
            </div>

            <div class="dashboard-tabs">
                <button
                    class=move || if active_tab.get() == "ideas" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("ideas".into())
                >"My Ideas"</button>
                <button
                    class=move || if active_tab.get() == "stokes" { "tab active" } else { "tab" }
                    on:click=move |_| active_tab.set("stokes".into())
                >"My Stokes"</button>
            </div>

            // My Ideas tab
            <section
                class="dashboard-section"
                style=move || if active_tab.get() == "ideas" { "" } else { "display:none" }
            >
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

            // My Stokes tab
            <section
                class="dashboard-section"
                style=move || if active_tab.get() == "stokes" { "" } else { "display:none" }
            >
                <Suspense fallback=move || view! { <Loading /> }>
                    {move || {
                        my_stokes.get().map(|result| {
                            match &*result {
                                Ok(resp) => {
                                    if resp.data.is_empty() {
                                        view! {
                                            <div class="empty-state">
                                                <h3>"No stoked ideas"</h3>
                                                <p>"You haven't stoked any ideas yet. Explore the Forge Floor to discover ideas worth stoking!"</p>
                                                <A href="/browse" attr:class="btn btn-primary">"Explore the Forge Floor"</A>
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
