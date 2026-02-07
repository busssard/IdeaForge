use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A todo item suggested for an idea.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub author_id: Uuid,
    pub assignee_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub status: TodoStatus,
    pub priority: TodoPriority,
    pub due_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    Suggested,
    Accepted,
    InProgress,
    Done,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoPriority {
    Normal = 0,
    High = 1,
    Urgent = 2,
}

impl Default for TodoPriority {
    fn default() -> Self {
        Self::Normal
    }
}
