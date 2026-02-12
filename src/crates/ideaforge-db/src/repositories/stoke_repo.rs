use sea_orm::*;
use uuid::Uuid;

use crate::entities::stoke;

pub struct StokeRepository<'a> {
    pub db: &'a DatabaseConnection,
}

impl<'a> StokeRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        id: Uuid,
        user_id: Uuid,
        idea_id: Uuid,
    ) -> Result<stoke::Model, DbErr> {
        let model = stoke::ActiveModel {
            id: Set(id),
            user_id: Set(user_id),
            idea_id: Set(idea_id),
            created_at: Set(chrono::Utc::now().fixed_offset()),
        };
        model.insert(self.db).await
    }

    pub async fn delete(&self, user_id: Uuid, idea_id: Uuid) -> Result<DeleteResult, DbErr> {
        stoke::Entity::delete_many()
            .filter(stoke::Column::UserId.eq(user_id))
            .filter(stoke::Column::IdeaId.eq(idea_id))
            .exec(self.db)
            .await
    }

    pub async fn exists(&self, user_id: Uuid, idea_id: Uuid) -> Result<bool, DbErr> {
        let count = stoke::Entity::find()
            .filter(stoke::Column::UserId.eq(user_id))
            .filter(stoke::Column::IdeaId.eq(idea_id))
            .count(self.db)
            .await?;
        Ok(count > 0)
    }

    pub async fn count_for_idea(&self, idea_id: Uuid) -> Result<u64, DbErr> {
        stoke::Entity::find()
            .filter(stoke::Column::IdeaId.eq(idea_id))
            .count(self.db)
            .await
    }

    pub async fn list_for_idea(
        &self,
        idea_id: Uuid,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<stoke::Model>, u64), DbErr> {
        let query = stoke::Entity::find()
            .filter(stoke::Column::IdeaId.eq(idea_id))
            .order_by_desc(stoke::Column::CreatedAt);

        let paginator = query.paginate(self.db, per_page);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page.saturating_sub(1)).await?;

        Ok((items, total))
    }
}
