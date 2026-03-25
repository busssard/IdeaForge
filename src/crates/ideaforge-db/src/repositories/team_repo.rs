use sea_orm::*;
use uuid::Uuid;

use crate::entities::team_application;
use crate::entities::team_member;
use crate::entities::enums::{ApplicationStatus, TeamMemberRole};

// =============================================================================
// TeamApplicationRepository
// =============================================================================

pub struct TeamApplicationRepository<'a> {
    pub db: &'a DatabaseConnection,
}

impl<'a> TeamApplicationRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        id: Uuid,
        idea_id: Uuid,
        user_id: Uuid,
        message: &str,
    ) -> Result<team_application::Model, DbErr> {
        let now = chrono::Utc::now().fixed_offset();
        let model = team_application::ActiveModel {
            id: Set(id),
            idea_id: Set(idea_id),
            user_id: Set(user_id),
            message: Set(message.to_string()),
            status: Set(ApplicationStatus::Pending),
            reviewed_by: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(self.db).await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<team_application::Model>, DbErr> {
        team_application::Entity::find_by_id(id)
            .one(self.db)
            .await
    }

    pub async fn list_for_idea(
        &self,
        idea_id: Uuid,
        status: Option<ApplicationStatus>,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<team_application::Model>, u64), DbErr> {
        let mut query = team_application::Entity::find()
            .filter(team_application::Column::IdeaId.eq(idea_id));

        if let Some(s) = status {
            query = query.filter(team_application::Column::Status.eq(s));
        }

        query = query.order_by_desc(team_application::Column::CreatedAt);

        let paginator = query.paginate(self.db, per_page);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page.saturating_sub(1)).await?;

        Ok((items, total))
    }

    pub async fn exists(&self, user_id: Uuid, idea_id: Uuid) -> Result<bool, DbErr> {
        let count = team_application::Entity::find()
            .filter(team_application::Column::UserId.eq(user_id))
            .filter(team_application::Column::IdeaId.eq(idea_id))
            .filter(
                team_application::Column::Status.is_in([
                    ApplicationStatus::Pending,
                    ApplicationStatus::Accepted,
                ]),
            )
            .count(self.db)
            .await?;
        Ok(count > 0)
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: ApplicationStatus,
        reviewed_by: Uuid,
    ) -> Result<team_application::Model, DbErr> {
        let model = team_application::Entity::find_by_id(id)
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Application not found".to_string()))?;

        let mut active: team_application::ActiveModel = model.into();
        active.status = Set(status);
        active.reviewed_by = Set(Some(reviewed_by));
        active.updated_at = Set(chrono::Utc::now().fixed_offset());
        active.update(self.db).await
    }
}

// =============================================================================
// TeamMemberRepository
// =============================================================================

pub struct TeamMemberRepository<'a> {
    pub db: &'a DatabaseConnection,
}

impl<'a> TeamMemberRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        id: Uuid,
        idea_id: Uuid,
        user_id: Uuid,
        role: TeamMemberRole,
    ) -> Result<team_member::Model, DbErr> {
        let model = team_member::ActiveModel {
            id: Set(id),
            idea_id: Set(idea_id),
            user_id: Set(user_id),
            role: Set(role),
            role_label: Set(None),
            joined_at: Set(chrono::Utc::now().fixed_offset()),
        };
        model.insert(self.db).await
    }

    pub async fn list_for_idea(
        &self,
        idea_id: Uuid,
    ) -> Result<Vec<team_member::Model>, DbErr> {
        team_member::Entity::find()
            .filter(team_member::Column::IdeaId.eq(idea_id))
            .order_by_asc(team_member::Column::JoinedAt)
            .all(self.db)
            .await
    }

    pub async fn exists(&self, user_id: Uuid, idea_id: Uuid) -> Result<bool, DbErr> {
        let count = team_member::Entity::find()
            .filter(team_member::Column::UserId.eq(user_id))
            .filter(team_member::Column::IdeaId.eq(idea_id))
            .count(self.db)
            .await?;
        Ok(count > 0)
    }

    pub async fn remove(
        &self,
        idea_id: Uuid,
        user_id: Uuid,
    ) -> Result<DeleteResult, DbErr> {
        team_member::Entity::delete_many()
            .filter(team_member::Column::IdeaId.eq(idea_id))
            .filter(team_member::Column::UserId.eq(user_id))
            .exec(self.db)
            .await
    }

    pub async fn count_for_idea(&self, idea_id: Uuid) -> Result<u64, DbErr> {
        team_member::Entity::find()
            .filter(team_member::Column::IdeaId.eq(idea_id))
            .count(self.db)
            .await
    }

    /// Update the custom role label for a team member.
    pub async fn update_role_label(
        &self,
        idea_id: Uuid,
        user_id: Uuid,
        role_label: Option<&str>,
    ) -> Result<team_member::Model, DbErr> {
        let model = team_member::Entity::find()
            .filter(team_member::Column::IdeaId.eq(idea_id))
            .filter(team_member::Column::UserId.eq(user_id))
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Team member not found".to_string()))?;

        let mut active: team_member::ActiveModel = model.into();
        active.role_label = Set(role_label.map(|s| s.to_string()));
        active.update(self.db).await
    }

    /// Update both the permission role and custom label for a team member.
    pub async fn update_role(
        &self,
        idea_id: Uuid,
        user_id: Uuid,
        role: TeamMemberRole,
        role_label: Option<&str>,
    ) -> Result<team_member::Model, DbErr> {
        let model = team_member::Entity::find()
            .filter(team_member::Column::IdeaId.eq(idea_id))
            .filter(team_member::Column::UserId.eq(user_id))
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Team member not found".to_string()))?;

        let mut active: team_member::ActiveModel = model.into();
        active.role = Set(role);
        active.role_label = Set(role_label.map(|s| s.to_string()));
        active.update(self.db).await
    }
}
