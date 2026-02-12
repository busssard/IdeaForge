//! Team formation handlers -- THE KILLER FEATURE.
//!
//! Provides task boards, team member management, and application
//! workflows that let Entrepreneurs build teams around their ideas.

use axum::{
    extract::Path,
    routing::{get, put, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;
use ideaforge_core::{
    BoardTaskStatus, TaskPriority, TeamMemberRole, TeamApplicationStatus,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        // Task board (one per idea for MVP)
        .route("/:id/board", get(get_board).post(create_board).put(update_board))
        // Board tasks
        .route("/:id/board/tasks", get(list_tasks).post(create_task))
        .route("/:id/board/tasks/:tid", get(get_task).put(update_task).delete(delete_task))
        // Team applications
        .route("/:id/team/apply", axum::routing::post(apply_to_team))
        .route("/:id/team/applications", get(list_applications))
        .route("/:id/team/applications/:aid", put(review_application))
        // Team members
        .route("/:id/team", get(list_team_members))
        .route("/:id/team/:uid", delete(remove_team_member))
}

// =============================================================================
// Task Board
// =============================================================================

#[derive(Debug, Serialize)]
pub struct BoardResponse {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub name: String,
    pub description: String,
    pub task_count: i32,
    pub open_tasks: i32,
}

async fn get_board(Path(_id): Path<Uuid>) -> &'static str {
    // TODO: Get or 404 the task board for this idea
    "get board"
}

async fn create_board(Path(_id): Path<Uuid>) -> &'static str {
    // TODO: Create a task board (Entrepreneur only, one per idea)
    "create board"
}

async fn update_board(Path(_id): Path<Uuid>) -> &'static str {
    // TODO: Update board metadata (name, description)
    "update board"
}

// =============================================================================
// Board Tasks
// =============================================================================

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: BoardTaskStatus,
    pub assignee_id: Option<Uuid>,
    pub skill_tags: Vec<String>,
    pub priority: TaskPriority,
}

async fn list_tasks(Path(_id): Path<Uuid>) -> &'static str {
    // TODO: List all tasks for the idea's board, with filtering
    "list tasks"
}

async fn create_task(Path(_id): Path<Uuid>) -> &'static str {
    // TODO: Create a task on the board (Entrepreneur or team lead only)
    "create task"
}

async fn get_task(Path((_id, _tid)): Path<(Uuid, Uuid)>) -> &'static str {
    // TODO: Get a single task
    "get task"
}

async fn update_task(Path((_id, _tid)): Path<(Uuid, Uuid)>) -> &'static str {
    // TODO: Update task (status, assignee, details)
    // Team members can claim/unclaim tasks
    // Lead can assign tasks and change details
    "update task"
}

async fn delete_task(Path((_id, _tid)): Path<(Uuid, Uuid)>) -> &'static str {
    // TODO: Delete a task (Entrepreneur only)
    "delete task"
}

// =============================================================================
// Team Applications
// =============================================================================

#[derive(Debug, Serialize)]
pub struct ApplicationResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_display_name: String,
    pub role: TeamMemberRole,
    pub pitch: String,
    pub status: TeamApplicationStatus,
}

async fn apply_to_team(Path(_id): Path<Uuid>) -> &'static str {
    // TODO: Submit application (Maker role required)
    // One application per user per idea
    "apply to team"
}

async fn list_applications(Path(_id): Path<Uuid>) -> &'static str {
    // TODO: List applications (Entrepreneur/lead only)
    "list applications"
}

async fn review_application(Path((_id, _aid)): Path<(Uuid, Uuid)>) -> &'static str {
    // TODO: Accept or reject an application (Entrepreneur/lead only)
    // Accepting creates a TeamMember record and sends notification
    "review application"
}

// =============================================================================
// Team Members
// =============================================================================

#[derive(Debug, Serialize)]
pub struct TeamMemberResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_display_name: String,
    pub role: TeamMemberRole,
    pub joined_at: String,
}

async fn list_team_members(Path(_id): Path<Uuid>) -> &'static str {
    // TODO: List all active team members for an idea
    "list team members"
}

async fn remove_team_member(Path((_id, _uid)): Path<(Uuid, Uuid)>) -> &'static str {
    // TODO: Remove a team member (Entrepreneur/lead only)
    "remove team member"
}
