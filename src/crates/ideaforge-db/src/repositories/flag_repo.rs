use sea_orm::*;
use uuid::Uuid;

use crate::entities::enums::{FlagStatus, FlagTargetType};
use crate::entities::flag;

pub struct FlagRepository<'a> {
    pub db: &'a DatabaseConnection,
}

impl<'a> FlagRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Create a new flag/report.
    pub async fn create(
        &self,
        id: Uuid,
        flagger_id: Uuid,
        target_type: FlagTargetType,
        target_id: Uuid,
        reason: &str,
    ) -> Result<flag::Model, DbErr> {
        let model = flag::ActiveModel {
            id: Set(id),
            flagger_id: Set(flagger_id),
            target_type: Set(target_type),
            target_id: Set(target_id),
            reason: Set(reason.to_string()),
            status: Set(FlagStatus::Pending),
            reviewed_by: Set(None),
            created_at: Set(chrono::Utc::now().fixed_offset()),
        };
        model.insert(self.db).await
    }

    /// List pending flags for admin review (paginated).
    pub async fn list_pending(
        &self,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<flag::Model>, u64), DbErr> {
        let query = flag::Entity::find()
            .filter(flag::Column::Status.eq(FlagStatus::Pending))
            .order_by_desc(flag::Column::CreatedAt);

        let paginator = query.paginate(self.db, per_page);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page.saturating_sub(1)).await?;

        Ok((items, total))
    }

    /// Admin reviews a flag: mark as reviewed or dismissed.
    pub async fn review(
        &self,
        flag_id: Uuid,
        reviewer_id: Uuid,
        status: FlagStatus,
    ) -> Result<flag::Model, DbErr> {
        let model = flag::Entity::find_by_id(flag_id)
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Flag not found".to_string()))?;

        let mut active: flag::ActiveModel = model.into();
        active.status = Set(status);
        active.reviewed_by = Set(Some(reviewer_id));
        active.update(self.db).await
    }

    /// Count flags for a specific target.
    pub async fn count_for_target(
        &self,
        target_type: FlagTargetType,
        target_id: Uuid,
    ) -> Result<u64, DbErr> {
        flag::Entity::find()
            .filter(flag::Column::TargetType.eq(target_type))
            .filter(flag::Column::TargetId.eq(target_id))
            .count(self.db)
            .await
    }

    /// Find a flag by ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<flag::Model>, DbErr> {
        flag::Entity::find_by_id(id).one(self.db).await
    }
}
