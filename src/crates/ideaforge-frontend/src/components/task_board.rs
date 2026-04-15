use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::api;
use crate::api::types::{BoardColumns, CreateTaskRequest, TaskResponse};
use crate::components::markdown::Markdown;
use crate::state::auth::AuthState;

fn format_budget(cents: i64, currency: &str) -> String {
    if cents == 0 {
        return String::new();
    }
    if currency == "ADA" {
        format!("{} ADA", cents as f64 / 1_000_000.0)
    } else {
        format!("${:.2}", cents as f64 / 100.0)
    }
}

#[component]
pub fn TaskBoard(idea_id: String, author_id: String) -> impl IntoView {
    let auth = expect_context::<AuthState>();

    let columns = RwSignal::new(Option::<BoardColumns>::None);
    let total_budget = RwSignal::new(0i64);
    let loading = RwSignal::new(true);
    let error_msg = RwSignal::new(String::new());
    let show_form = RwSignal::new(false);
    let creating = RwSignal::new(false);

    let idea_id_stored = StoredValue::new(idea_id.clone());
    let _author_id_stored = StoredValue::new(author_id.clone());

    let title_ref = NodeRef::<leptos::html::Input>::new();
    let desc_ref = NodeRef::<leptos::html::Textarea>::new();
    let priority_ref = NodeRef::<leptos::html::Select>::new();
    let budget_ref = NodeRef::<leptos::html::Input>::new();
    let currency_ref = NodeRef::<leptos::html::Select>::new();

    // Fetch board data
    let fetch_board = move || {
        let idea_id = idea_id_stored.get_value();
        loading.set(true);
        error_msg.set(String::new());
        wasm_bindgen_futures::spawn_local(async move {
            match api::board::get_board(&idea_id).await {
                Ok(board) => {
                    total_budget.set(board.total_budget_cents);
                    columns.set(Some(board.columns));
                }
                Err(e) => {
                    // 404 is fine — no tasks yet
                    if e.status == 404 {
                        columns.set(Some(BoardColumns {
                            open: vec![],
                            assigned: vec![],
                            in_review: vec![],
                            done: vec![],
                        }));
                        total_budget.set(0);
                    } else {
                        error_msg.set(e.message);
                    }
                }
            }
            loading.set(false);
        });
    };

    // Initial load
    fetch_board();

    // Toggle form
    let toggle_form = move |_: web_sys::MouseEvent| {
        show_form.set(!show_form.get_untracked());
    };

    // Create task handler
    let on_create = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if creating.get_untracked() {
            return;
        }

        let title = title_ref
            .get()
            .map(|el| {
                let el: &web_sys::HtmlInputElement = el.unchecked_ref();
                el.value()
            })
            .unwrap_or_default();

        if title.trim().is_empty() {
            return;
        }

        let description = desc_ref
            .get()
            .map(|el| {
                let el: &web_sys::HtmlTextAreaElement = el.unchecked_ref();
                let v = el.value();
                if v.trim().is_empty() { None } else { Some(v) }
            })
            .unwrap_or(None);

        let priority = priority_ref
            .get()
            .map(|el| {
                let el: &web_sys::HtmlSelectElement = el.unchecked_ref();
                let v = el.value();
                if v.is_empty() { None } else { Some(v) }
            })
            .unwrap_or(None);

        // Parse budget: input is in dollars (or ADA), convert to cents (or lovelace)
        let budget_str = budget_ref
            .get()
            .map(|el| {
                let el: &web_sys::HtmlInputElement = el.unchecked_ref();
                el.value()
            })
            .unwrap_or_default();

        let currency_val = currency_ref
            .get()
            .map(|el| {
                let el: &web_sys::HtmlSelectElement = el.unchecked_ref();
                el.value()
            })
            .unwrap_or_else(|| "USD".to_string());

        let (budget_cents, currency) = if budget_str.trim().is_empty() {
            (None, None)
        } else if let Ok(amount) = budget_str.trim().parse::<f64>() {
            if amount <= 0.0 {
                (None, None)
            } else if currency_val == "ADA" {
                (Some((amount * 1_000_000.0) as i64), Some("ADA".to_string()))
            } else {
                (Some((amount * 100.0) as i64), Some("USD".to_string()))
            }
        } else {
            (None, None)
        };

        creating.set(true);
        let idea_id = idea_id_stored.get_value();

        let req = CreateTaskRequest {
            title,
            description,
            priority,
            assignee_id: None,
            skill_tags: None,
            due_date: None,
            budget_cents,
            currency,
        };

        wasm_bindgen_futures::spawn_local(async move {
            match api::board::create_task(&idea_id, req).await {
                Ok(_) => {
                    show_form.set(false);
                    // Clear form
                    // Re-fetch
                }
                Err(e) => error_msg.set(e.message),
            }
            creating.set(false);
            // Re-fetch board
            let idea_id = idea_id_stored.get_value();
            match api::board::get_board(&idea_id).await {
                Ok(board) => {
                    total_budget.set(board.total_budget_cents);
                    columns.set(Some(board.columns));
                }
                Err(_) => {}
            }
        });
    };

    // Move task helper — returns a closure
    let move_task = move |task_id: String, new_status: &'static str| {
        let idea_id = idea_id_stored.get_value();
        let task_id = task_id.clone();
        move |_: web_sys::MouseEvent| {
            let idea_id = idea_id.clone();
            let task_id = task_id.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = api::board::update_task_status(&idea_id, &task_id, new_status).await;
                // Re-fetch board
                match api::board::get_board(&idea_id).await {
                    Ok(board) => {
                        total_budget.set(board.total_budget_cents);
                        columns.set(Some(board.columns));
                    }
                    Err(_) => {}
                }
            });
        }
    };

    // Delete task helper
    let delete_task = move |task_id: String| {
        let idea_id = idea_id_stored.get_value();
        move |_: web_sys::MouseEvent| {
            let idea_id = idea_id.clone();
            let task_id = task_id.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = api::board::delete_task(&idea_id, &task_id).await;
                // Re-fetch board
                match api::board::get_board(&idea_id).await {
                    Ok(board) => {
                        total_budget.set(board.total_budget_cents);
                        columns.set(Some(board.columns));
                    }
                    Err(_) => {}
                }
            });
        }
    };

    // Render a single task card with actions based on status
    let render_card = move |task: TaskResponse, status: &'static str| {
        let priority_class = format!("priority-badge priority-{}", task.priority);
        let desc = task.description.clone();
        let due = task.due_date.clone().map(|d| {
            d.split('T').next().unwrap_or("").to_string()
        });
        let tags = task.skill_tags.clone();
        let task_id = task.id.clone();
        let task_id_del = task.id.clone();
        let budget_display = format_budget(task.budget_cents, &task.currency);

        // Build action buttons based on column
        let actions = match status {
            "open" => {
                let on_start = move_task(task_id.clone(), "assigned");
                view! {
                    <button class="btn btn-xs btn-ghost" on:click=on_start>
                        "Start \u{2192}"
                    </button>
                }
                .into_any()
            }
            "assigned" => {
                let on_reopen = move_task(task_id.clone(), "open");
                let on_review = move_task(task_id.clone(), "in_review");
                view! {
                    <button class="btn btn-xs btn-ghost" on:click=on_reopen>
                        "\u{2190} Reopen"
                    </button>
                    <button class="btn btn-xs btn-ghost" on:click=on_review>
                        "Review \u{2192}"
                    </button>
                }
                .into_any()
            }
            "in_review" => {
                let on_back = move_task(task_id.clone(), "assigned");
                let on_done = move_task(task_id.clone(), "done");
                view! {
                    <button class="btn btn-xs btn-ghost" on:click=on_back>
                        "\u{2190} Back"
                    </button>
                    <button class="btn btn-xs btn-ghost" on:click=on_done>
                        "Done \u{2713}"
                    </button>
                }
                .into_any()
            }
            "done" => {
                let on_reopen = move_task(task_id.clone(), "open");
                view! {
                    <button class="btn btn-xs btn-ghost" on:click=on_reopen>
                        "\u{2190} Reopen"
                    </button>
                }
                .into_any()
            }
            _ => view! {}.into_any(),
        };

        let on_delete = delete_task(task_id_del);

        view! {
            <div class="task-card">
                <div class="task-card-header">
                    <span class="task-title">{task.title.clone()}</span>
                    <span class=priority_class>{task.priority.clone()}</span>
                </div>
                {desc
                    .map(|d| {
                        view! { <Markdown content=d class="task-desc".to_string() /> }.into_any()
                    })
                    .unwrap_or_else(|| view! {}.into_any())}
                {(!budget_display.is_empty())
                    .then(|| {
                        view! { <span class="task-budget">{budget_display}</span> }
                    })}
                <div class="task-card-footer">
                    {due
                        .map(|d| {
                            view! { <span class="task-due">{d}</span> }.into_any()
                        })
                        .unwrap_or_else(|| view! {}.into_any())}
                    {tags
                        .into_iter()
                        .map(|t| {
                            view! { <span class="skill-tag">{t}</span> }
                        })
                        .collect::<Vec<_>>()}
                </div>
                <div class="task-card-actions">
                    {actions}
                    <button class="btn btn-xs btn-ghost" on:click=on_delete>
                        "\u{00D7}"
                    </button>
                </div>
            </div>
        }
    };

    view! {
        <div class="task-board">
            <div class="task-board-header">
                <div>
                    <h3>"Project Board"</h3>
                    {move || {
                        let budget = total_budget.get();
                        if budget > 0 {
                            // Default to USD display for total
                            let display = format!("${:.2}", budget as f64 / 100.0);
                            view! {
                                <span class="board-total-budget">
                                    "Total Budget: " {display}
                                </span>
                            }
                                .into_any()
                        } else {
                            view! {}.into_any()
                        }
                    }}
                </div>
                {move || {
                    if auth.is_authenticated() {
                        view! {
                            <button class="btn btn-sm btn-primary" on:click=toggle_form>
                                "+ Add Task"
                            </button>
                        }
                            .into_any()
                    } else {
                        view! {}.into_any()
                    }
                }}
            </div>

            // Error display
            {move || {
                let err = error_msg.get();
                if err.is_empty() {
                    view! {}.into_any()
                } else {
                    view! { <div class="form-error">{err}</div> }.into_any()
                }
            }}

            // Add task form (collapsible)
            <div class="task-add-form" style:display=move || {
                if show_form.get() { "block" } else { "none" }
            }>
                <form on:submit=on_create>
                    <div class="task-add-form-inner">
                        <input
                            node_ref=title_ref
                            type="text"
                            placeholder="Task title"
                            required
                        />
                        <textarea
                            node_ref=desc_ref
                            placeholder="Description (optional)"
                            rows="2"
                        ></textarea>
                        <select node_ref=priority_ref>
                            <option value="">"Priority..."</option>
                            <option value="low">"Low"</option>
                            <option value="normal">"Normal"</option>
                            <option value="high">"High"</option>
                            <option value="urgent">"Urgent"</option>
                        </select>
                        <div class="task-budget-input-group">
                            <input
                                node_ref=budget_ref
                                type="number"
                                step="0.01"
                                min="0"
                                placeholder="Budget (optional)"
                            />
                            <select node_ref=currency_ref>
                                <option value="USD">"USD ($)"</option>
                                <option value="ADA">"ADA"</option>
                            </select>
                        </div>
                        <button
                            class="btn btn-primary btn-sm"
                            type="submit"
                            disabled=move || creating.get()
                        >
                            {move || {
                                if creating.get() { "Creating..." } else { "Create Task" }
                            }}
                        </button>
                    </div>
                </form>
            </div>

            // Board columns
            {move || {
                if loading.get() {
                    view! { <p class="text-muted">"Loading board..."</p> }.into_any()
                } else {
                    match columns.get() {
                        Some(cols) => {
                            let open_count = cols.open.len();
                            let assigned_count = cols.assigned.len();
                            let in_review_count = cols.in_review.len();
                            let done_count = cols.done.len();

                            let open_cards = cols
                                .open
                                .into_iter()
                                .map(|t| render_card(t, "open"))
                                .collect::<Vec<_>>();
                            let assigned_cards = cols
                                .assigned
                                .into_iter()
                                .map(|t| render_card(t, "assigned"))
                                .collect::<Vec<_>>();
                            let in_review_cards = cols
                                .in_review
                                .into_iter()
                                .map(|t| render_card(t, "in_review"))
                                .collect::<Vec<_>>();
                            let done_cards = cols
                                .done
                                .into_iter()
                                .map(|t| render_card(t, "done"))
                                .collect::<Vec<_>>();

                            view! {
                                <div class="task-columns">
                                    // Open column
                                    <div class="task-column">
                                        <div class="task-column-header">
                                            <span>"Open"</span>
                                            <span class="task-count">
                                                {open_count.to_string()}
                                            </span>
                                        </div>
                                        <div class="task-column-body">{open_cards}</div>
                                    </div>

                                    // Assigned column
                                    <div class="task-column">
                                        <div class="task-column-header">
                                            <span>"Assigned"</span>
                                            <span class="task-count">
                                                {assigned_count.to_string()}
                                            </span>
                                        </div>
                                        <div class="task-column-body">{assigned_cards}</div>
                                    </div>

                                    // In Review column
                                    <div class="task-column">
                                        <div class="task-column-header">
                                            <span>"In Review"</span>
                                            <span class="task-count">
                                                {in_review_count.to_string()}
                                            </span>
                                        </div>
                                        <div class="task-column-body">{in_review_cards}</div>
                                    </div>

                                    // Done column
                                    <div class="task-column">
                                        <div class="task-column-header">
                                            <span>"Done"</span>
                                            <span class="task-count">
                                                {done_count.to_string()}
                                            </span>
                                        </div>
                                        <div class="task-column-body">{done_cards}</div>
                                    </div>
                                </div>
                            }
                                .into_any()
                        }
                        None => view! {}.into_any(),
                    }
                }
            }}
        </div>
    }
}
