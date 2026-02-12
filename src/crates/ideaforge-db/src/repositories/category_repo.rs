use sea_orm::*;
use uuid::Uuid;

use crate::entities::category;

pub struct CategoryRepository<'a> {
    pub db: &'a DatabaseConnection,
}

impl<'a> CategoryRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn list_all(&self) -> Result<Vec<category::Model>, DbErr> {
        category::Entity::find()
            .order_by_asc(category::Column::SortOrder)
            .all(self.db)
            .await
    }

    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<category::Model>, DbErr> {
        category::Entity::find()
            .filter(category::Column::Slug.eq(slug))
            .one(self.db)
            .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<category::Model>, DbErr> {
        category::Entity::find_by_id(id).one(self.db).await
    }

    /// Insert a category if the slug doesn't already exist.
    pub async fn upsert_by_slug(
        &self,
        id: Uuid,
        name: &str,
        slug: &str,
        description: &str,
        icon: Option<&str>,
        sort_order: i32,
    ) -> Result<category::Model, DbErr> {
        // Check if exists
        if let Some(existing) = self.find_by_slug(slug).await? {
            return Ok(existing);
        }
        let model = category::ActiveModel {
            id: Set(id),
            name: Set(name.to_string()),
            slug: Set(slug.to_string()),
            description: Set(description.to_string()),
            icon: Set(icon.map(|s| s.to_string())),
            parent_id: Set(None),
            sort_order: Set(sort_order),
            created_at: Set(chrono::Utc::now().fixed_offset()),
        };
        model.insert(self.db).await
    }
}
