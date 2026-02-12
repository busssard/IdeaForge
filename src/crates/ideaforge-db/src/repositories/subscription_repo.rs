use sea_orm::*;
use uuid::Uuid;

use crate::entities::subscription;

pub struct SubscriptionRepository<'a> {
    pub db: &'a DatabaseConnection,
}

impl<'a> SubscriptionRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        id: Uuid,
        user_id: Uuid,
        idea_id: Uuid,
    ) -> Result<subscription::Model, DbErr> {
        let model = subscription::ActiveModel {
            id: Set(id),
            user_id: Set(user_id),
            idea_id: Set(idea_id),
            created_at: Set(chrono::Utc::now().fixed_offset()),
        };
        model.insert(self.db).await
    }

    pub async fn delete(&self, user_id: Uuid, idea_id: Uuid) -> Result<DeleteResult, DbErr> {
        subscription::Entity::delete_many()
            .filter(subscription::Column::UserId.eq(user_id))
            .filter(subscription::Column::IdeaId.eq(idea_id))
            .exec(self.db)
            .await
    }

    pub async fn exists(&self, user_id: Uuid, idea_id: Uuid) -> Result<bool, DbErr> {
        let count = subscription::Entity::find()
            .filter(subscription::Column::UserId.eq(user_id))
            .filter(subscription::Column::IdeaId.eq(idea_id))
            .count(self.db)
            .await?;
        Ok(count > 0)
    }

    pub async fn list_for_user(
        &self,
        user_id: Uuid,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<subscription::Model>, u64), DbErr> {
        let query = subscription::Entity::find()
            .filter(subscription::Column::UserId.eq(user_id))
            .order_by_desc(subscription::Column::CreatedAt);

        let paginator = query.paginate(self.db, per_page);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page.saturating_sub(1)).await?;

        Ok((items, total))
    }
}
