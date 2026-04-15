use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use leptos_router::components::A;

use crate::api;
use crate::api::types::UpdateIdeaRequest;
use crate::components::bot_section::BotSection;
use crate::components::comment_section::CommentSection;
use crate::components::loading::Loading;
use crate::components::markdown::Markdown;
use crate::components::maturity_badge::MaturityBadge;
use crate::components::nda_wall::NdaWall;
use crate::components::share_buttons::ShareButtons;
use crate::components::stoke_button::StokeButton;
use crate::components::subscribe_button::SubscribeButton;
use crate::components::suggestion_section::SuggestionSection;
use crate::components::task_board::TaskBoard;
use crate::components::team_panel::TeamPanel;
use crate::components::visibility_badge::VisibilityBadge;
use crate::state::auth::AuthState;

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
                            let author_name = idea.author_name.clone().unwrap_or_else(|| "Unknown".to_string());
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

                            // Check if current user is the author
                            let author_id_check = idea.author_id.clone();
                            let auth = expect_context::<AuthState>();
                            let is_author = auth
                                .user
                                .get_untracked()
                                .map_or(false, |u| u.id == author_id_check);

                            // Editing state
                            let editing = RwSignal::new(false);
                            let edit_saving = RwSignal::new(false);
                            let edit_error = RwSignal::new(String::new());

                            // Pre-fill edit fields
                            let edit_title = RwSignal::new(idea.title.clone());
                            let edit_summary = RwSignal::new(idea.summary.clone());
                            let edit_description = RwSignal::new(idea.description.clone());
                            let edit_openness = RwSignal::new(idea.openness.clone());
                            let edit_lifecycle = RwSignal::new(
                                if idea.lifecycle.is_empty() { "not_started".to_string() } else { idea.lifecycle.clone() }
                            );

                            let idea_id_edit = idea.id.clone();
                            let refetch = refetch_trigger;

                            // Owned clones for reactive closures (avoid borrowing &Idea)
                            let title_view = idea.title.clone();
                            let summary_view = idea.summary.clone();
                            let description_view = idea.description.clone();
                            let maturity_view = idea.maturity.clone();

                            view! {
                                <div class="idea-detail">
                                    <div class="idea-detail-header">
                                        // Title: editable or static
                                        {move || {
                                            if editing.get() {
                                                view! {
                                                    <input
                                                        class="form-input idea-edit-title"
                                                        type="text"
                                                        value=edit_title.get_untracked()
                                                        on:input=move |ev| {
                                                            edit_title.set(event_target_value(&ev));
                                                        }
                                                    />
                                                }
                                                    .into_any()
                                            } else {
                                                view! {
                                                    <h1 class="idea-detail-title">
                                                        {title_view.clone()}
                                                    </h1>
                                                }
                                                    .into_any()
                                            }
                                        }}

                                        <div class="idea-detail-badges">
                                            <MaturityBadge maturity=maturity_view.clone() />
                                            <VisibilityBadge openness=openness_for_badge />
                                            <details class="maturity-explainer">
                                                <summary>"How does this advance?"</summary>
                                                <p>
                                                    "An idea's maturity isn't set by hand — it reflects how much
                                                    the community has engaged. The stages go "
                                                    <strong>"Spark \u{2192} Half-Baked \u{2192} Thought Through \u{2192}
                                                    Serious Proposal \u{2192} In Work \u{2192} Almost Finished \u{2192}
                                                    Completed"</strong>
                                                    "."
                                                </p>
                                                <p>
                                                    "Progression is earned through stokes, comments, suggestions,
                                                    team members joining, and tasks being completed. Keep building
                                                    and sharing and the idea will grow."
                                                </p>
                                            </details>
                                        </div>
                                        <div class="idea-detail-meta">
                                            <span>
                                                "by "
                                                <A href=format!(
                                                    "/profile/{author_id}",
                                                )>{author_name}</A>
                                            </span>
                                            <span>"Created " {created_date}</span>
                                            <span>"Updated " {updated_date}</span>
                                            // Edit button for author
                                            {is_author.then(|| {
                                                view! {
                                                    <button
                                                        class="btn btn-ghost btn-sm"
                                                        on:click=move |_| {
                                                            editing.set(!editing.get_untracked());
                                                        }
                                                    >
                                                        {move || {
                                                            if editing.get() {
                                                                "Cancel Edit"
                                                            } else {
                                                                "\u{270F}\u{FE0F} Edit"
                                                            }
                                                        }}
                                                    </button>
                                                }
                                            })}
                                        </div>
                                    </div>

                                    // Edit form for summary, description, openness
                                    {move || {
                                        if editing.get() {
                                            let id_for_save = idea_id_edit.clone();
                                            let on_save = move |ev: web_sys::SubmitEvent| {
                                                ev.prevent_default();
                                                if edit_saving.get_untracked() {
                                                    return;
                                                }
                                                edit_saving.set(true);
                                                edit_error.set(String::new());
                                                let id = id_for_save.clone();
                                                let req = UpdateIdeaRequest {
                                                    title: Some(edit_title.get_untracked()),
                                                    summary: Some(edit_summary.get_untracked()),
                                                    description: Some(edit_description.get_untracked()),
                                                    openness: Some(edit_openness.get_untracked()),
                                                    lifecycle: Some(edit_lifecycle.get_untracked()),
                                                    category_id: None,
                                                };
                                                wasm_bindgen_futures::spawn_local(async move {
                                                    match api::ideas::update_idea(&id, req).await {
                                                        Ok(_) => {
                                                            editing.set(false);
                                                            refetch.set(refetch.get_untracked() + 1);
                                                        }
                                                        Err(e) => edit_error.set(e.message),
                                                    }
                                                    edit_saving.set(false);
                                                });
                                            };
                                            let on_cancel = move |_: web_sys::MouseEvent| {
                                                editing.set(false);
                                                edit_error.set(String::new());
                                            };
                                            view! {
                                                <form
                                                    class="idea-edit-form card mb-lg"
                                                    on:submit=on_save
                                                >
                                                    // Edit error
                                                    {move || {
                                                        let err = edit_error.get();
                                                        if err.is_empty() {
                                                            view! { <div></div> }.into_any()
                                                        } else {
                                                            view! {
                                                                <div class="form-error">{err}</div>
                                                            }
                                                                .into_any()
                                                        }
                                                    }}

                                                    <div class="form-group">
                                                        <label class="form-label">"Summary"</label>
                                                        <textarea
                                                            class="form-input"
                                                            rows="2"
                                                            on:input=move |ev| {
                                                                edit_summary
                                                                    .set(event_target_value(&ev));
                                                            }
                                                        >
                                                            {edit_summary.get_untracked()}
                                                        </textarea>
                                                    </div>

                                                    <div class="form-group">
                                                        <label class="form-label">
                                                            "Description"
                                                        </label>
                                                        <textarea
                                                            class="form-input"
                                                            rows="10"
                                                            on:input=move |ev| {
                                                                edit_description
                                                                    .set(event_target_value(&ev));
                                                            }
                                                        >
                                                            {edit_description.get_untracked()}
                                                        </textarea>
                                                    </div>

                                                    <div class="form-group">
                                                        <label class="form-label">"Openness"</label>
                                                        <select
                                                            class="form-select"
                                                            on:change=move |ev| {
                                                                edit_openness
                                                                    .set(event_target_value(&ev));
                                                            }
                                                        >
                                                            <option
                                                                value="open"
                                                                selected=move || {
                                                                    edit_openness.get() == "open"
                                                                }
                                                            >
                                                                "Open"
                                                            </option>
                                                            <option
                                                                value="collaborative"
                                                                selected=move || {
                                                                    edit_openness.get()
                                                                        == "collaborative"
                                                                }
                                                            >
                                                                "Collaborative"
                                                            </option>
                                                            <option
                                                                value="commercial"
                                                                selected=move || {
                                                                    edit_openness.get()
                                                                        == "commercial"
                                                                }
                                                            >
                                                                "Commercial"
                                                            </option>
                                                            <option
                                                                value="private"
                                                                selected=move || {
                                                                    edit_openness.get() == "private"
                                                                }
                                                            >
                                                                "Private"
                                                            </option>
                                                        </select>
                                                    </div>

                                                    <div class="form-group">
                                                        <label class="form-label">"Status"</label>
                                                        <select
                                                            class="form-select"
                                                            on:change=move |ev| {
                                                                edit_lifecycle
                                                                    .set(event_target_value(&ev));
                                                            }
                                                        >
                                                            <option
                                                                value="not_started"
                                                                selected=move || {
                                                                    edit_lifecycle.get()
                                                                        == "not_started"
                                                                }
                                                            >
                                                                "Not started"
                                                            </option>
                                                            <option
                                                                value="ongoing"
                                                                selected=move || {
                                                                    edit_lifecycle.get()
                                                                        == "ongoing"
                                                                }
                                                            >
                                                                "Ongoing"
                                                            </option>
                                                            <option
                                                                value="finished"
                                                                selected=move || {
                                                                    edit_lifecycle.get()
                                                                        == "finished"
                                                                }
                                                            >
                                                                "Finished"
                                                            </option>
                                                        </select>
                                                    </div>

                                                    <div class="idea-edit-actions">
                                                        <button
                                                            class="btn btn-primary btn-sm"
                                                            type="submit"
                                                            disabled=move || edit_saving.get()
                                                        >
                                                            {move || {
                                                                if edit_saving.get() {
                                                                    "Saving..."
                                                                } else {
                                                                    "Save Changes"
                                                                }
                                                            }}
                                                        </button>
                                                        <button
                                                            class="btn btn-ghost btn-sm"
                                                            type="button"
                                                            on:click=on_cancel
                                                        >
                                                            "Cancel"
                                                        </button>
                                                    </div>
                                                </form>
                                            }
                                                .into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }
                                    }}

                                    // Summary card (only in view mode)
                                    {move || {
                                        if !editing.get() {
                                            view! {
                                                <div class="card mb-lg">
                                                    <h4>"Summary"</h4>
                                                    <p
                                                        class="mt-sm"
                                                        style="color: var(--text-secondary)"
                                                    >
                                                        {summary_view.clone()}
                                                    </p>
                                                </div>
                                            }
                                                .into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }
                                    }}

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
                                            // Description (only in view mode)
                                            {move || {
                                                if !editing.get() {
                                                    view! {
                                                        <Markdown
                                                            content=description_view.clone()
                                                            class="idea-detail-body".to_string()
                                                        />
                                                    }
                                                        .into_any()
                                                } else {
                                                    view! { <div></div> }.into_any()
                                                }
                                            }}

                                            <div class="idea-detail-actions">
                                                <StokeButton
                                                    idea_id=idea_id.clone()
                                                    initial_count=idea.stoke_count
                                                    initial_stoked=has_stoked
                                                    prominent=true
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
