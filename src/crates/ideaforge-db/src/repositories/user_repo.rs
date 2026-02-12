use sea_orm::*;
use uuid::Uuid;

use crate::entities::enums::UserRole;
use crate::entities::user;

pub struct UserRepository<'a> {
    pub db: &'a DatabaseConnection,
}

impl<'a> UserRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        id: Uuid,
        email: &str,
        password_hash: &str,
        display_name: &str,
        role: UserRole,
    ) -> Result<user::Model, DbErr> {
        let now = chrono::Utc::now().fixed_offset();
        let model = user::ActiveModel {
            id: Set(id),
            email: Set(email.to_string()),
            password_hash: Set(password_hash.to_string()),
            display_name: Set(display_name.to_string()),
            bio: Set(String::new()),
            avatar_url: Set(None),
            role: Set(role),
            email_verified: Set(false),
            is_bot: Set(false),
            bot_operator: Set(None),
            bot_description: Set(None),
            bot_api_key_hash: Set(None),
            skills: Set(serde_json::json!([])),
            looking_for: Set(None),
            availability: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(self.db).await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<user::Model>, DbErr> {
        user::Entity::find_by_id(id).one(self.db).await
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<user::Model>, DbErr> {
        user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .one(self.db)
            .await
    }

    pub async fn update(
        &self,
        id: Uuid,
        display_name: Option<&str>,
        bio: Option<&str>,
        avatar_url: Option<Option<&str>>,
    ) -> Result<user::Model, DbErr> {
        let user = user::Entity::find_by_id(id)
            .one(self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("User not found".to_string()))?;

        let mut active: user::ActiveModel = user.into();
        if let Some(name) = display_name {
            active.display_name = Set(name.to_string());
        }
        if let Some(b) = bio {
            active.bio = Set(b.to_string());
        }
        if let Some(url) = avatar_url {
            active.avatar_url = Set(url.map(|s| s.to_string()));
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());
        active.update(self.db).await
    }
}
