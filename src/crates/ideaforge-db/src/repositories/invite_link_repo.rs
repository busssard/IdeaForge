use sea_orm::*;
use uuid::Uuid;

use crate::entities::enums::InvitePermission;
use crate::entities::invite_link;

pub struct InviteLinkRepository<'a> {
    pub db: &'a DatabaseConnection,
}

impl<'a> InviteLinkRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        id: Uuid,
        idea_id: Uuid,
        token: &str,
        permission: InvitePermission,
        created_by: Uuid,
    ) -> Result<invite_link::Model, DbErr> {
        let model = invite_link::ActiveModel {
            id: Set(id),
            idea_id: Set(idea_id),
            token: Set(token.to_string()),
            permission: Set(permission),
            created_by: Set(created_by),
            expires_at: Set(None),
            revoked_at: Set(None),
            access_count: Set(0),
            created_at: Set(chrono::Utc::now().fixed_offset()),
        };
        model.insert(self.db).await
    }

    pub async fn find_by_token(&self, token: &str) -> Result<Option<invite_link::Model>, DbErr> {
        invite_link::Entity::find()
            .filter(invite_link::Column::Token.eq(token))
            .filter(invite_link::Column::RevokedAt.is_null())
            .one(self.db)
            .await
    }

    pub async fn list_for_idea(&self, idea_id: Uuid) -> Result<Vec<invite_link::Model>, DbErr> {
        invite_link::Entity::find()
            .filter(invite_link::Column::IdeaId.eq(idea_id))
            .order_by_desc(invite_link::Column::CreatedAt)
            .all(self.db)
            .await
    }

    pub async fn revoke(&self, token: &str) -> Result<invite_link::Model, DbErr> {
        let model = invite_link::Entity::find()
            .filter(invite_link::Column::Token.eq(token))
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Invite link not found".to_string()))?;

        let mut active: invite_link::ActiveModel = model.into();
        active.revoked_at = Set(Some(chrono::Utc::now().fixed_offset()));
        active.update(self.db).await
    }

    pub async fn increment_access_count(&self, token: &str) -> Result<(), DbErr> {
        let model = invite_link::Entity::find()
            .filter(invite_link::Column::Token.eq(token))
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Invite link not found".to_string()))?;

        let new_count = model.access_count + 1;
        let mut active: invite_link::ActiveModel = model.into();
        active.access_count = Set(new_count);
        active.update(self.db).await?;
        Ok(())
    }
}
