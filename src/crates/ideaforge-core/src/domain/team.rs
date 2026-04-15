use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// Task Board -- the organizational backbone of team formation
// =============================================================================

/// A task board attached to an idea. One board per idea for MVP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBoard {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A task on a board. Tasks can be created by the idea author (Entrepreneur)
/// and claimed/completed by team members (Makers).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardTask {
    pub id: Uuid,
    pub board_id: Uuid,
    pub title: String,
    pub description: String,
    pub status: BoardTaskStatus,
    pub assignee_id: Option<Uuid>,
    pub skill_tags: Vec<String>,
    pub priority: TaskPriority,
    pub due_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum BoardTaskStatus {
    /// Available for someone to claim
    #[default]
    Open,
    /// Someone is working on it
    Assigned,
    /// Work submitted, pending review by lead
    InReview,
    /// Completed
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum TaskPriority {
    #[default]
    Normal = 0,
    High = 1,
    Urgent = 2,
}

// =============================================================================
// Team Members -- who is working on this idea
// =============================================================================

/// A team member on an idea.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub user_id: Uuid,
    pub role: TeamMemberRole,
    pub status: TeamMemberStatus,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum TeamMemberRole {
    /// Idea author / project lead
    Lead,
    /// Accepted maker working on tasks
    #[default]
    Builder,
    /// Non-building contributor / advisor
    Advisor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum TeamMemberStatus {
    #[default]
    Active,
    Inactive,
    Removed,
}

// =============================================================================
// Team Applications -- how makers join ideas
// =============================================================================

/// An application from a user to join an idea's team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamApplication {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub user_id: Uuid,
    pub role: TeamMemberRole,
    pub pitch: String,
    pub status: TeamApplicationStatus,
    pub reviewed_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum TeamApplicationStatus {
    #[default]
    Pending,
    Accepted,
    Rejected,
    Withdrawn,
}

// =============================================================================
// API request/response types for team formation
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateBoardRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub skill_tags: Option<Vec<String>>,
    pub priority: Option<TaskPriority>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<BoardTaskStatus>,
    pub assignee_id: Option<Uuid>,
    pub skill_tags: Option<Vec<String>>,
    pub priority: Option<TaskPriority>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct ApplyToTeamRequest {
    pub role: Option<TeamMemberRole>,
    pub pitch: String,
}

#[derive(Debug, Deserialize)]
pub struct ReviewApplicationRequest {
    pub accepted: bool,
}
