use sea_orm::*;
use uuid::Uuid;

use crate::entities::bot_endorsement;

pub struct BotEndorsementRepository<'a> {
    pub db: &'a DatabaseConnection,
}

impl<'a> BotEndorsementRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        id: Uuid,
        bot_id: Uuid,
        idea_id: Uuid,
        reason: &str,
    ) -> Result<bot_endorsement::Model, DbErr> {
        let model = bot_endorsement::ActiveModel {
            id: Set(id),
            bot_id: Set(bot_id),
            idea_id: Set(idea_id),
            reason: Set(reason.to_string()),
            created_at: Set(chrono::Utc::now().fixed_offset()),
        };
        model.insert(self.db).await
    }

    pub async fn list_for_idea(&self, idea_id: Uuid) -> Result<Vec<bot_endorsement::Model>, DbErr> {
        bot_endorsement::Entity::find()
            .filter(bot_endorsement::Column::IdeaId.eq(idea_id))
            .order_by_desc(bot_endorsement::Column::CreatedAt)
            .all(self.db)
            .await
    }

    pub async fn count_for_idea(&self, idea_id: Uuid) -> Result<u64, DbErr> {
        bot_endorsement::Entity::find()
            .filter(bot_endorsement::Column::IdeaId.eq(idea_id))
            .count(self.db)
            .await
    }

    pub async fn exists(&self, bot_id: Uuid, idea_id: Uuid) -> Result<bool, DbErr> {
        let count = bot_endorsement::Entity::find()
            .filter(bot_endorsement::Column::BotId.eq(bot_id))
            .filter(bot_endorsement::Column::IdeaId.eq(idea_id))
            .count(self.db)
            .await?;
        Ok(count > 0)
    }
}
