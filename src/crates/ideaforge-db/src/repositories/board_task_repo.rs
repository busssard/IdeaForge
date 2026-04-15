use sea_orm::*;
use uuid::Uuid;

use crate::entities::board_task;
use crate::entities::enums::{TaskPriority, TaskStatus};

pub struct BoardTaskRepository<'a> {
    pub db: &'a DatabaseConnection,
}

impl<'a> BoardTaskRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Create a new board task.
    pub async fn create(
        &self,
        id: Uuid,
        idea_id: Uuid,
        title: &str,
        description: Option<&str>,
        priority: TaskPriority,
        assignee_id: Option<Uuid>,
        created_by: Uuid,
        skill_tags: serde_json::Value,
        due_date: Option<chrono::NaiveDate>,
        position: i32,
        budget_cents: i64,
        currency: &str,
    ) -> Result<board_task::Model, DbErr> {
        let now = chrono::Utc::now().fixed_offset();
        let status = if assignee_id.is_some() {
            TaskStatus::Assigned
        } else {
            TaskStatus::Open
        };
        let model = board_task::ActiveModel {
            id: Set(id),
            idea_id: Set(idea_id),
            title: Set(title.to_string()),
            description: Set(description.map(|s| s.to_string())),
            status: Set(status),
            priority: Set(priority),
            assignee_id: Set(assignee_id),
            created_by: Set(created_by),
            skill_tags: Set(skill_tags),
            due_date: Set(due_date),
            position: Set(position),
            budget_cents: Set(budget_cents),
            currency: Set(currency.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            completed_at: Set(None),
        };
        model.insert(self.db).await
    }

    /// Find a task by ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<board_task::Model>, DbErr> {
        board_task::Entity::find_by_id(id).one(self.db).await
    }

    /// List tasks for an idea (paginated, with optional filters).
    pub async fn list_for_idea(
        &self,
        idea_id: Uuid,
        status_filter: Option<TaskStatus>,
        assignee_filter: Option<Uuid>,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<board_task::Model>, u64), DbErr> {
        let mut query = board_task::Entity::find().filter(board_task::Column::IdeaId.eq(idea_id));

        if let Some(status) = status_filter {
            query = query.filter(board_task::Column::Status.eq(status));
        }

        if let Some(assignee) = assignee_filter {
            query = query.filter(board_task::Column::AssigneeId.eq(assignee));
        }

        query = query
            .order_by_asc(board_task::Column::Position)
            .order_by_desc(board_task::Column::CreatedAt);

        let paginator = query.paginate(self.db, per_page);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page.saturating_sub(1)).await?;

        Ok((items, total))
    }

    /// List all tasks for an idea (no pagination, for kanban view).
    pub async fn list_all_for_idea(&self, idea_id: Uuid) -> Result<Vec<board_task::Model>, DbErr> {
        board_task::Entity::find()
            .filter(board_task::Column::IdeaId.eq(idea_id))
            .order_by_asc(board_task::Column::Position)
            .order_by_desc(board_task::Column::CreatedAt)
            .all(self.db)
            .await
    }

    /// Update the status of a task. Sets completed_at when status is Done.
    pub async fn update_status(
        &self,
        id: Uuid,
        status: TaskStatus,
    ) -> Result<board_task::Model, DbErr> {
        let model = board_task::Entity::find_by_id(id)
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Board task not found".to_string()))?;

        let now = chrono::Utc::now().fixed_offset();
        let mut active: board_task::ActiveModel = model.into();
        active.status = Set(status.clone());
        active.updated_at = Set(now);

        if status == TaskStatus::Done {
            active.completed_at = Set(Some(now));
        } else {
            active.completed_at = Set(None);
        }

        active.update(self.db).await
    }

    /// Update task fields (all optional).
    pub async fn update(
        &self,
        id: Uuid,
        title: Option<&str>,
        description: Option<Option<&str>>,
        priority: Option<TaskPriority>,
        assignee_id: Option<Option<Uuid>>,
        skill_tags: Option<serde_json::Value>,
        due_date: Option<Option<chrono::NaiveDate>>,
        position: Option<i32>,
        budget_cents: Option<i64>,
        currency: Option<&str>,
    ) -> Result<board_task::Model, DbErr> {
        let model = board_task::Entity::find_by_id(id)
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Board task not found".to_string()))?;

        let mut active: board_task::ActiveModel = model.into();

        if let Some(t) = title {
            active.title = Set(t.to_string());
        }
        if let Some(d) = description {
            active.description = Set(d.map(|s| s.to_string()));
        }
        if let Some(p) = priority {
            active.priority = Set(p);
        }
        if let Some(a) = assignee_id {
            active.assignee_id = Set(a);
        }
        if let Some(s) = skill_tags {
            active.skill_tags = Set(s);
        }
        if let Some(d) = due_date {
            active.due_date = Set(d);
        }
        if let Some(p) = position {
            active.position = Set(p);
        }
        if let Some(b) = budget_cents {
            active.budget_cents = Set(b);
        }
        if let Some(c) = currency {
            active.currency = Set(c.to_string());
        }

        active.updated_at = Set(chrono::Utc::now().fixed_offset());
        active.update(self.db).await
    }

    /// Delete a task by ID.
    pub async fn delete(&self, id: Uuid) -> Result<DeleteResult, DbErr> {
        board_task::Entity::delete_by_id(id).exec(self.db).await
    }

    /// Count tasks for an idea.
    pub async fn count_for_idea(&self, idea_id: Uuid) -> Result<u64, DbErr> {
        board_task::Entity::find()
            .filter(board_task::Column::IdeaId.eq(idea_id))
            .count(self.db)
            .await
    }
}
