use sea_orm::*;
use uuid::Uuid;

use crate::entities::contribution;
use crate::entities::enums::ContributionKind;

pub struct ContributionRepository<'a> {
    pub db: &'a DatabaseConnection,
}

impl<'a> ContributionRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        id: Uuid,
        idea_id: Uuid,
        user_id: Uuid,
        contribution_type: ContributionKind,
        title: Option<String>,
        body: &str,
        parent_id: Option<Uuid>,
    ) -> Result<contribution::Model, DbErr> {
        let now = chrono::Utc::now().fixed_offset();
        let model = contribution::ActiveModel {
            id: Set(id),
            idea_id: Set(idea_id),
            user_id: Set(user_id),
            parent_id: Set(parent_id),
            contribution_type: Set(contribution_type),
            title: Set(title),
            body: Set(body.to_string()),
            is_bot: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(self.db).await
    }

    pub async fn list_for_idea(
        &self,
        idea_id: Uuid,
        contribution_type: Option<ContributionKind>,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<contribution::Model>, u64), DbErr> {
        let mut query = contribution::Entity::find()
            .filter(contribution::Column::IdeaId.eq(idea_id));

        if let Some(ct) = contribution_type {
            query = query.filter(contribution::Column::ContributionType.eq(ct));
        }

        query = query.order_by_desc(contribution::Column::CreatedAt);

        let paginator = query.paginate(self.db, per_page);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page.saturating_sub(1)).await?;

        Ok((items, total))
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<contribution::Model>, DbErr> {
        contribution::Entity::find_by_id(id).one(self.db).await
    }

    pub async fn count_for_idea(&self, idea_id: Uuid) -> Result<u64, DbErr> {
        contribution::Entity::find()
            .filter(contribution::Column::IdeaId.eq(idea_id))
            .count(self.db)
            .await
    }
}
