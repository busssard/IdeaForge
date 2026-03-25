use sea_orm::*;
use uuid::Uuid;

use crate::entities::enums::IdeaOpenness;
use crate::entities::{idea, nda_signature, nda_template};

pub struct NdaTemplateRepository<'a> {
    pub db: &'a DatabaseConnection,
}

impl<'a> NdaTemplateRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        id: Uuid,
        idea_id: Uuid,
        title: &str,
        body: &str,
        confidentiality_period_days: i32,
        jurisdiction: Option<&str>,
        created_by: Uuid,
    ) -> Result<nda_template::Model, DbErr> {
        let now = chrono::Utc::now().fixed_offset();
        let model = nda_template::ActiveModel {
            id: Set(id),
            idea_id: Set(idea_id),
            title: Set(title.to_string()),
            body: Set(body.to_string()),
            confidentiality_period_days: Set(confidentiality_period_days),
            jurisdiction: Set(jurisdiction.map(|s| s.to_string())),
            created_by: Set(created_by),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(self.db).await
    }

    pub async fn find_by_idea_id(
        &self,
        idea_id: Uuid,
    ) -> Result<Option<nda_template::Model>, DbErr> {
        nda_template::Entity::find()
            .filter(nda_template::Column::IdeaId.eq(idea_id))
            .one(self.db)
            .await
    }

    pub async fn update(
        &self,
        id: Uuid,
        title: Option<&str>,
        body: Option<&str>,
        confidentiality_period_days: Option<i32>,
        jurisdiction: Option<Option<&str>>,
    ) -> Result<nda_template::Model, DbErr> {
        let model = nda_template::Entity::find_by_id(id)
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("NDA template not found".to_string()))?;

        let mut active: nda_template::ActiveModel = model.into();
        if let Some(t) = title {
            active.title = Set(t.to_string());
        }
        if let Some(b) = body {
            active.body = Set(b.to_string());
        }
        if let Some(d) = confidentiality_period_days {
            active.confidentiality_period_days = Set(d);
        }
        if let Some(j) = jurisdiction {
            active.jurisdiction = Set(j.map(|s| s.to_string()));
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());
        active.update(self.db).await
    }
}

pub struct NdaSignatureRepository<'a> {
    pub db: &'a DatabaseConnection,
}

impl<'a> NdaSignatureRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        id: Uuid,
        nda_template_id: Uuid,
        idea_id: Uuid,
        signer_id: Uuid,
        signer_name: &str,
        signer_email: &str,
        ip_address: Option<&str>,
        expires_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    ) -> Result<nda_signature::Model, DbErr> {
        let model = nda_signature::ActiveModel {
            id: Set(id),
            nda_template_id: Set(nda_template_id),
            idea_id: Set(idea_id),
            signer_id: Set(signer_id),
            signer_name: Set(signer_name.to_string()),
            signer_email: Set(signer_email.to_string()),
            ip_address: Set(ip_address.map(|s| s.to_string())),
            signed_at: Set(chrono::Utc::now().fixed_offset()),
            expires_at: Set(expires_at),
            revoked_at: Set(None),
        };
        model.insert(self.db).await
    }

    pub async fn has_signed(&self, signer_id: Uuid, idea_id: Uuid) -> Result<bool, DbErr> {
        let count = nda_signature::Entity::find()
            .filter(nda_signature::Column::SignerId.eq(signer_id))
            .filter(nda_signature::Column::IdeaId.eq(idea_id))
            .filter(nda_signature::Column::RevokedAt.is_null())
            .count(self.db)
            .await?;
        Ok(count > 0)
    }

    pub async fn find_by_idea_and_signer(
        &self,
        idea_id: Uuid,
        signer_id: Uuid,
    ) -> Result<Option<nda_signature::Model>, DbErr> {
        nda_signature::Entity::find()
            .filter(nda_signature::Column::IdeaId.eq(idea_id))
            .filter(nda_signature::Column::SignerId.eq(signer_id))
            .filter(nda_signature::Column::RevokedAt.is_null())
            .one(self.db)
            .await
    }

    pub async fn list_for_idea(
        &self,
        idea_id: Uuid,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<nda_signature::Model>, u64), DbErr> {
        let query = nda_signature::Entity::find()
            .filter(nda_signature::Column::IdeaId.eq(idea_id))
            .order_by_desc(nda_signature::Column::SignedAt);

        let paginator = query.paginate(self.db, per_page);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page.saturating_sub(1)).await?;

        Ok((items, total))
    }

    pub async fn count_nda_ideas_by_user(&self, user_id: Uuid) -> Result<u64, DbErr> {
        idea::Entity::find()
            .filter(idea::Column::AuthorId.eq(user_id))
            .filter(idea::Column::Openness.eq(IdeaOpenness::NdaProtected))
            .filter(idea::Column::ArchivedAt.is_null())
            .count(self.db)
            .await
    }
}
