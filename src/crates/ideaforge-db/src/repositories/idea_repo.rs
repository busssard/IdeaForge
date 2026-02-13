use sea_orm::*;
use uuid::Uuid;

use crate::entities::enums::{IdeaMaturity, IdeaOpenness};
use crate::entities::idea;

pub struct IdeaRepository<'a> {
    pub db: &'a DatabaseConnection,
}

impl<'a> IdeaRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        id: Uuid,
        author_id: Uuid,
        title: &str,
        summary: &str,
        description: &str,
        maturity: IdeaMaturity,
        openness: IdeaOpenness,
        category_id: Option<Uuid>,
    ) -> Result<idea::Model, DbErr> {
        let now = chrono::Utc::now().fixed_offset();
        let model = idea::ActiveModel {
            id: Set(id),
            author_id: Set(author_id),
            title: Set(title.to_string()),
            summary: Set(summary.to_string()),
            description: Set(description.to_string()),
            maturity: Set(maturity),
            openness: Set(openness),
            category_id: Set(category_id),
            stoke_count: Set(0),
            looking_for_skills: Set(serde_json::json!([])),
            created_at: Set(now),
            updated_at: Set(now),
            archived_at: Set(None),
        };
        model.insert(self.db).await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<idea::Model>, DbErr> {
        idea::Entity::find_by_id(id)
            .filter(idea::Column::ArchivedAt.is_null())
            .one(self.db)
            .await
    }

    pub async fn list(
        &self,
        maturity: Option<IdeaMaturity>,
        openness: Option<IdeaOpenness>,
        category_id: Option<Uuid>,
        author_id: Option<Uuid>,
        exclude_private: bool,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<idea::Model>, u64), DbErr> {
        let mut query = idea::Entity::find().filter(idea::Column::ArchivedAt.is_null());

        if let Some(m) = maturity {
            query = query.filter(idea::Column::Maturity.eq(m));
        }
        if let Some(o) = openness {
            query = query.filter(idea::Column::Openness.eq(o));
        }
        if let Some(cid) = category_id {
            query = query.filter(idea::Column::CategoryId.eq(cid));
        }
        if let Some(aid) = author_id {
            query = query.filter(idea::Column::AuthorId.eq(aid));
        }
        if exclude_private {
            query = query.filter(idea::Column::Openness.ne(IdeaOpenness::Private));
        }

        query = query.order_by_desc(idea::Column::CreatedAt);

        let paginator = query.paginate(self.db, per_page);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page.saturating_sub(1)).await?;

        Ok((items, total))
    }

    pub async fn update(
        &self,
        id: Uuid,
        title: Option<&str>,
        summary: Option<&str>,
        description: Option<&str>,
        openness: Option<IdeaOpenness>,
        category_id: Option<Option<Uuid>>,
    ) -> Result<idea::Model, DbErr> {
        let model = idea::Entity::find_by_id(id)
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Idea not found".to_string()))?;

        let mut active: idea::ActiveModel = model.into();
        if let Some(t) = title {
            active.title = Set(t.to_string());
        }
        if let Some(s) = summary {
            active.summary = Set(s.to_string());
        }
        if let Some(d) = description {
            active.description = Set(d.to_string());
        }
        if let Some(o) = openness {
            active.openness = Set(o);
        }
        if let Some(cid) = category_id {
            active.category_id = Set(cid);
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());
        active.update(self.db).await
    }

    pub async fn archive(&self, id: Uuid) -> Result<idea::Model, DbErr> {
        let model = idea::Entity::find_by_id(id)
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Idea not found".to_string()))?;

        let mut active: idea::ActiveModel = model.into();
        active.archived_at = Set(Some(chrono::Utc::now().fixed_offset()));
        active.updated_at = Set(chrono::Utc::now().fixed_offset());
        active.update(self.db).await
    }

    pub async fn update_stoke_count(&self, idea_id: Uuid, count: i32) -> Result<(), DbErr> {
        let model = idea::Entity::find_by_id(idea_id)
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Idea not found".to_string()))?;

        let mut active: idea::ActiveModel = model.into();
        active.stoke_count = Set(count);
        active.update(self.db).await?;
        Ok(())
    }
}
