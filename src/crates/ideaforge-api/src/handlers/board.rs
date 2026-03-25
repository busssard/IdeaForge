//! Task board handlers for idea kanban boards.
//!
//! Provides CRUD for board tasks nested under ideas,
//! plus a kanban-style board view grouped by status columns.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::{AuthUser, OptionalAuth};
use crate::state::AppState;
use ideaforge_db::entities::board_task;
use ideaforge_db::entities::enums::{TaskPriority, TaskStatus};
use ideaforge_db::repositories::board_task_repo::BoardTaskRepository;
use ideaforge_db::repositories::idea_repo::IdeaRepository;
use ideaforge_db::repositories::team_repo::TeamMemberRepository;
use sea_orm::*;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:id/board", get(get_board))
        .route("/:id/board/tasks", get(list_tasks).post(create_task))
        .route(
            "/:id/board/tasks/:task_id",
            get(get_task).put(update_task).delete(delete_task),
        )
        .route("/:id/board/tasks/:task_id/status", put(update_task_status))
}

// --- DTOs ---

#[derive(Debug, Deserialize)]
struct TaskPath {
    id: Uuid,
    task_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub assignee_id: Option<Uuid>,
    pub skill_tags: Option<Vec<String>>,
    pub due_date: Option<String>,
    pub budget_cents: Option<i64>,
    pub currency: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub assignee_id: Option<Option<Uuid>>,
    pub skill_tags: Option<Vec<String>>,
    pub due_date: Option<Option<String>>,
    pub position: Option<i32>,
    pub budget_cents: Option<i64>,
    pub currency: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ListTasksQuery {
    pub status: Option<String>,
    pub assignee_id: Option<Uuid>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub assignee_id: Option<Uuid>,
    pub created_by: Uuid,
    pub skill_tags: Vec<String>,
    pub due_date: Option<String>,
    pub position: i32,
    pub budget_cents: i64,
    pub currency: String,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BoardResponse {
    pub idea_id: Uuid,
    pub columns: BoardColumns,
    pub total_tasks: u64,
    pub total_budget_cents: i64,
}

#[derive(Debug, Serialize)]
pub struct BoardColumns {
    pub open: Vec<TaskResponse>,
    pub assigned: Vec<TaskResponse>,
    pub in_review: Vec<TaskResponse>,
    pub done: Vec<TaskResponse>,
}

#[derive(Debug, Serialize)]
pub struct TaskListResponse {
    pub data: Vec<TaskResponse>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

fn err(status: StatusCode, code: &str, message: &str) -> impl IntoResponse {
    (
        status,
        Json(serde_json::json!({
            "error": { "code": code, "message": message }
        })),
    )
        .into_response()
}

fn task_response(m: &board_task::Model) -> TaskResponse {
    let skill_tags: Vec<String> = match &m.skill_tags {
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => vec![],
    };
    TaskResponse {
        id: m.id,
        idea_id: m.idea_id,
        title: m.title.clone(),
        description: m.description.clone(),
        status: m.status.to_string(),
        priority: m.priority.to_string(),
        assignee_id: m.assignee_id,
        created_by: m.created_by,
        skill_tags,
        due_date: m.due_date.map(|d| d.to_string()),
        position: m.position,
        budget_cents: m.budget_cents,
        currency: m.currency.clone(),
        created_at: m.created_at.to_rfc3339(),
        updated_at: m.updated_at.to_rfc3339(),
        completed_at: m.completed_at.map(|dt| dt.to_rfc3339()),
    }
}

fn parse_priority(s: &str) -> Option<TaskPriority> {
    TaskPriority::from_str_opt(s)
}

fn parse_status(s: &str) -> Option<TaskStatus> {
    TaskStatus::from_str_opt(s)
}

fn parse_due_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// Check whether the user is the idea author or a team member.
async fn is_author_or_team_member(
    db: &DatabaseConnection,
    user_id: Uuid,
    idea_id: Uuid,
) -> Result<bool, DbErr> {
    let idea_repo = IdeaRepository::new(db);
    match idea_repo.find_by_id(idea_id).await? {
        Some(idea) if idea.author_id == user_id => Ok(true),
        Some(_) => {
            let team_repo = TeamMemberRepository::new(db);
            team_repo.exists(user_id, idea_id).await
        }
        None => Ok(false),
    }
}

// --- Handlers ---

async fn get_board(
    State(state): State<AppState>,
    _opt_auth: OptionalAuth,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Verify idea exists
    let idea_repo = IdeaRepository::new(state.db.connection());
    match idea_repo.find_by_id(id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to find idea: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    let task_repo = BoardTaskRepository::new(state.db.connection());
    match task_repo.list_all_for_idea(id).await {
        Ok(tasks) => {
            let total_tasks = tasks.len() as u64;
            let total_budget_cents: i64 = tasks.iter().map(|t| t.budget_cents).sum();
            let mut open = Vec::new();
            let mut assigned = Vec::new();
            let mut in_review = Vec::new();
            let mut done = Vec::new();

            for task in &tasks {
                let resp = task_response(task);
                match task.status {
                    TaskStatus::Open => open.push(resp),
                    TaskStatus::Assigned => assigned.push(resp),
                    TaskStatus::InReview => in_review.push(resp),
                    TaskStatus::Done => done.push(resp),
                }
            }

            Json(BoardResponse {
                idea_id: id,
                columns: BoardColumns {
                    open,
                    assigned,
                    in_review,
                    done,
                },
                total_tasks,
                total_budget_cents,
            })
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list board tasks: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn list_tasks(
    State(state): State<AppState>,
    _opt_auth: OptionalAuth,
    Path(id): Path<Uuid>,
    Query(params): Query<ListTasksQuery>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    let status_filter = params.status.as_deref().and_then(parse_status);

    let task_repo = BoardTaskRepository::new(state.db.connection());
    match task_repo
        .list_for_idea(id, status_filter, params.assignee_id, page, per_page)
        .await
    {
        Ok((tasks, total)) => {
            let total_pages = if total == 0 {
                0
            } else {
                (total + per_page - 1) / per_page
            };
            Json(TaskListResponse {
                data: tasks.iter().map(task_response).collect(),
                meta: PaginationMeta {
                    total,
                    page,
                    per_page,
                    total_pages,
                },
            })
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list tasks: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn create_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    // Verify user is author or team member
    match is_author_or_team_member(state.db.connection(), auth.user_id, id).await {
        Ok(true) => {}
        Ok(false) => {
            return err(
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Only the idea author or team members can create tasks",
            )
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to check permissions: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    // Validate title
    let title = body.title.trim();
    if title.is_empty() || title.len() > 255 {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Title is required (max 255 chars)",
        )
        .into_response();
    }

    // Validate priority
    let priority = match body.priority.as_deref() {
        None => TaskPriority::Normal,
        Some(p) => match parse_priority(p) {
            Some(v) => v,
            None => {
                return err(
                    StatusCode::BAD_REQUEST,
                    "VALIDATION_ERROR",
                    "Invalid priority. Must be: low, normal, high, or urgent",
                )
                .into_response();
            }
        },
    };

    // Parse due_date
    let due_date = match body.due_date.as_deref() {
        None => None,
        Some(d) => match parse_due_date(d) {
            Some(date) => Some(date),
            None => {
                return err(
                    StatusCode::BAD_REQUEST,
                    "VALIDATION_ERROR",
                    "Invalid due_date. Must be YYYY-MM-DD format",
                )
                .into_response();
            }
        },
    };

    // If assignee_id is provided, verify they're a team member (or the idea author)
    if let Some(assignee_id) = body.assignee_id {
        match is_author_or_team_member(state.db.connection(), assignee_id, id).await {
            Ok(true) => {}
            Ok(false) => {
                return err(
                    StatusCode::BAD_REQUEST,
                    "VALIDATION_ERROR",
                    "Assignee must be a team member or the idea author",
                )
                .into_response()
            }
            Err(e) => {
                tracing::error!("Failed to verify assignee: {e}");
                return err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Internal server error",
                )
                .into_response();
            }
        }
    }

    // Compute next position: max existing position + 1
    let task_repo = BoardTaskRepository::new(state.db.connection());
    let position = match board_task::Entity::find()
        .filter(board_task::Column::IdeaId.eq(id))
        .order_by_desc(board_task::Column::Position)
        .one(state.db.connection())
        .await
    {
        Ok(Some(last)) => last.position + 1,
        Ok(None) => 0,
        Err(e) => {
            tracing::error!("Failed to compute task position: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    };

    let skill_tags = body
        .skill_tags
        .as_ref()
        .map(|tags| serde_json::json!(tags))
        .unwrap_or(serde_json::json!([]));

    let budget_cents = body.budget_cents.unwrap_or(0);
    let currency = body.currency.as_deref().unwrap_or("USD");

    match task_repo
        .create(
            Uuid::new_v4(),
            id,
            title,
            body.description.as_deref(),
            priority,
            body.assignee_id,
            auth.user_id,
            skill_tags,
            due_date,
            position,
            budget_cents,
            currency,
        )
        .await
    {
        Ok(task) => (StatusCode::CREATED, Json(task_response(&task))).into_response(),
        Err(e) => {
            tracing::error!("Failed to create task: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn get_task(
    State(state): State<AppState>,
    _opt_auth: OptionalAuth,
    Path(path): Path<TaskPath>,
) -> impl IntoResponse {
    let task_repo = BoardTaskRepository::new(state.db.connection());
    match task_repo.find_by_id(path.task_id).await {
        Ok(Some(task)) if task.idea_id == path.id => Json(task_response(&task)).into_response(),
        Ok(Some(_)) => {
            err(
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "Task not found for this idea",
            )
            .into_response()
        }
        Ok(None) => err(StatusCode::NOT_FOUND, "NOT_FOUND", "Task not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to get task: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn update_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(path): Path<TaskPath>,
    Json(body): Json<UpdateTaskRequest>,
) -> impl IntoResponse {
    // Verify user is author or team member
    match is_author_or_team_member(state.db.connection(), auth.user_id, path.id).await {
        Ok(true) => {}
        Ok(false) => {
            return err(
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Only the idea author or team members can update tasks",
            )
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to check permissions: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    // Verify task exists and belongs to this idea
    let task_repo = BoardTaskRepository::new(state.db.connection());
    match task_repo.find_by_id(path.task_id).await {
        Ok(Some(task)) if task.idea_id == path.id => {}
        Ok(Some(_)) => {
            return err(
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "Task not found for this idea",
            )
            .into_response()
        }
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Task not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to find task: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    // Validate title if provided
    if let Some(ref t) = body.title {
        if t.trim().is_empty() || t.len() > 255 {
            return err(
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "Title must be 1-255 chars",
            )
            .into_response();
        }
    }

    // Validate priority if provided
    let priority = match body.priority.as_deref() {
        None => None,
        Some(p) => match parse_priority(p) {
            Some(v) => Some(v),
            None => {
                return err(
                    StatusCode::BAD_REQUEST,
                    "VALIDATION_ERROR",
                    "Invalid priority. Must be: low, normal, high, or urgent",
                )
                .into_response();
            }
        },
    };

    // Validate assignee if provided
    let assignee_id = match &body.assignee_id {
        None => None,
        Some(None) => Some(None), // explicit unassign
        Some(Some(aid)) => {
            match is_author_or_team_member(state.db.connection(), *aid, path.id).await {
                Ok(true) => Some(Some(*aid)),
                Ok(false) => {
                    return err(
                        StatusCode::BAD_REQUEST,
                        "VALIDATION_ERROR",
                        "Assignee must be a team member or the idea author",
                    )
                    .into_response()
                }
                Err(e) => {
                    tracing::error!("Failed to verify assignee: {e}");
                    return err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "INTERNAL_ERROR",
                        "Internal server error",
                    )
                    .into_response();
                }
            }
        }
    };

    // Parse due_date if provided
    let due_date: Option<Option<chrono::NaiveDate>> = match &body.due_date {
        None => None,
        Some(None) => Some(None), // explicit clear
        Some(Some(d)) => match parse_due_date(d) {
            Some(date) => Some(Some(date)),
            None => {
                return err(
                    StatusCode::BAD_REQUEST,
                    "VALIDATION_ERROR",
                    "Invalid due_date. Must be YYYY-MM-DD format",
                )
                .into_response();
            }
        },
    };

    // Build skill_tags
    let skill_tags = body
        .skill_tags
        .as_ref()
        .map(|tags| serde_json::json!(tags));

    // Build description update: only pass through if the field was present in the request
    let description: Option<Option<&str>> = body
        .description
        .as_ref()
        .map(|d| Some(d.as_str()));

    match task_repo
        .update(
            path.task_id,
            body.title.as_deref(),
            description,
            priority,
            assignee_id,
            skill_tags,
            due_date,
            body.position,
            body.budget_cents,
            body.currency.as_deref(),
        )
        .await
    {
        Ok(task) => Json(task_response(&task)).into_response(),
        Err(e) => {
            tracing::error!("Failed to update task: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn update_task_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(path): Path<TaskPath>,
    Json(body): Json<UpdateTaskStatusRequest>,
) -> impl IntoResponse {
    // Verify user is author or team member
    match is_author_or_team_member(state.db.connection(), auth.user_id, path.id).await {
        Ok(true) => {}
        Ok(false) => {
            return err(
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Only the idea author or team members can update task status",
            )
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to check permissions: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    // Validate status
    let status = match parse_status(&body.status) {
        Some(s) => s,
        None => {
            return err(
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "Invalid status. Must be: open, assigned, in_review, or done",
            )
            .into_response();
        }
    };

    // Verify task exists and belongs to this idea
    let task_repo = BoardTaskRepository::new(state.db.connection());
    match task_repo.find_by_id(path.task_id).await {
        Ok(Some(task)) if task.idea_id == path.id => {}
        Ok(Some(_)) => {
            return err(
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "Task not found for this idea",
            )
            .into_response()
        }
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Task not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to find task: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    match task_repo.update_status(path.task_id, status).await {
        Ok(task) => Json(task_response(&task)).into_response(),
        Err(e) => {
            tracing::error!("Failed to update task status: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn delete_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(path): Path<TaskPath>,
) -> impl IntoResponse {
    // Verify task exists and belongs to this idea
    let task_repo = BoardTaskRepository::new(state.db.connection());
    let task = match task_repo.find_by_id(path.task_id).await {
        Ok(Some(task)) if task.idea_id == path.id => task,
        Ok(Some(_)) => {
            return err(
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "Task not found for this idea",
            )
            .into_response()
        }
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Task not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to find task: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    };

    // Only idea author or task creator can delete
    let idea_repo = IdeaRepository::new(state.db.connection());
    let is_author = match idea_repo.find_by_id(path.id).await {
        Ok(Some(idea)) => idea.author_id == auth.user_id,
        _ => false,
    };

    if !is_author && task.created_by != auth.user_id {
        return err(
            StatusCode::FORBIDDEN,
            "FORBIDDEN",
            "Only the idea author or task creator can delete tasks",
        )
        .into_response();
    }

    match task_repo.delete(path.task_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete task: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}
