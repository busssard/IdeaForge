use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A category for organizing ideas, supporting hierarchical nesting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub icon: Option<String>,
    pub parent_id: Option<Uuid>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

/// Category with nested children for tree display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTree {
    #[serde(flatten)]
    pub category: Category,
    pub children: Vec<CategoryTree>,
}
