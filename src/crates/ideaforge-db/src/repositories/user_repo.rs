use sea_orm::{prelude::Expr, *};
use uuid::Uuid;

use crate::entities::enums::UserRole;
use crate::entities::user;
use crate::entities::{idea, stoke};

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

    /// Create a bot account with bot-specific fields.
    pub async fn create_bot(
        &self,
        id: Uuid,
        email: &str,
        display_name: &str,
        bot_operator: &str,
        bot_description: &str,
        bot_api_key_hash: &str,
    ) -> Result<user::Model, DbErr> {
        let now = chrono::Utc::now().fixed_offset();
        let model = user::ActiveModel {
            id: Set(id),
            email: Set(email.to_string()),
            password_hash: Set(String::new()), // Bots don't use passwords
            display_name: Set(display_name.to_string()),
            bio: Set(bot_description.to_string()),
            avatar_url: Set(None),
            role: Set(UserRole::Curious), // Bots use the "curious" role
            email_verified: Set(true),    // Bots are pre-verified
            is_bot: Set(true),
            bot_operator: Set(Some(bot_operator.to_string())),
            bot_description: Set(Some(bot_description.to_string())),
            bot_api_key_hash: Set(Some(bot_api_key_hash.to_string())),
            skills: Set(serde_json::json!([])),
            looking_for: Set(None),
            availability: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(self.db).await
    }

    /// Find a bot user by the SHA-256 hash of its API key.
    pub async fn find_bot_by_api_key_hash(
        &self,
        api_key_hash: &str,
    ) -> Result<Option<user::Model>, DbErr> {
        user::Entity::find()
            .filter(user::Column::IsBot.eq(true))
            .filter(user::Column::BotApiKeyHash.eq(api_key_hash))
            .one(self.db)
            .await
    }

    /// List all bot users.
    pub async fn list_bots(&self) -> Result<Vec<user::Model>, DbErr> {
        user::Entity::find()
            .filter(user::Column::IsBot.eq(true))
            .order_by_desc(user::Column::CreatedAt)
            .all(self.db)
            .await
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
        skills: Option<&serde_json::Value>,
        looking_for: Option<Option<&str>>,
        availability: Option<&str>,
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
        if let Some(s) = skills {
            active.skills = Set(s.clone());
        }
        if let Some(lf) = looking_for {
            active.looking_for = Set(lf.map(|s| s.to_string()));
        }
        if let Some(av) = availability {
            active.availability = Set(Some(av.to_string()));
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());
        active.update(self.db).await
    }

    /// List users with filtering and pagination.
    ///
    /// # Arguments
    /// * `role` - Filter by user role
    /// * `skills` - Filter by skills (JSONB array contains all provided skills)
    /// * `include_bots` - Include bot accounts in results
    /// * `sort` - Sorting option ("recently_joined", "most_active", etc.)
    /// * `page` - Page number (1-indexed)
    /// * `per_page` - Items per page
    ///
    /// # Returns
    /// Tuple of (users, total_count)
    pub async fn list(
        &self,
        role: Option<UserRole>,
        skills: Option<Vec<String>>,
        include_bots: bool,
        sort: &str,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<UserWithStats>, u64), DbErr> {
        let mut query = user::Entity::find();

        // Filter by role
        if let Some(r) = role {
            query = query.filter(user::Column::Role.eq(r));
        }

        // Filter by bot status
        if !include_bots {
            query = query.filter(user::Column::IsBot.eq(false));
        }

        // Filter by skills using JSONB containment
        if let Some(skill_list) = skills {
            if !skill_list.is_empty() {
                // Build JSONB array literal for containment check
                let skills_json = serde_json::to_string(&skill_list).unwrap_or_else(|_| "[]".to_string());
                query = query.filter(
                    Expr::cust_with_values(
                        "skills @> ?::jsonb",
                        [skills_json]
                    )
                );
            }
        }

        // Sorting
        match sort {
            "recently_joined" => {
                query = query.order_by_desc(user::Column::CreatedAt);
            }
            "most_active" | _ => {
                // Default to recently_joined for now
                query = query.order_by_desc(user::Column::CreatedAt);
            }
        }

        // Count total
        let total = query.clone().count(self.db).await?;

        // Paginate
        let offset = (page.saturating_sub(1)) * per_page;
        let users = query
            .offset(offset)
            .limit(per_page)
            .all(self.db)
            .await?;

        // Fetch stats for each user
        let mut results = Vec::with_capacity(users.len());
        for user in users {
            let idea_count = idea::Entity::find()
                .filter(idea::Column::AuthorId.eq(user.id))
                .filter(idea::Column::ArchivedAt.is_null())
                .count(self.db)
                .await
                .unwrap_or(0);

            let stoke_count = stoke::Entity::find()
                .filter(stoke::Column::UserId.eq(user.id))
                .count(self.db)
                .await
                .unwrap_or(0);

            results.push(UserWithStats {
                user,
                idea_count,
                stoke_count,
            });
        }

        Ok((results, total))
    }
}

/// User model with aggregated stats.
#[derive(Debug, Clone)]
pub struct UserWithStats {
    pub user: user::Model,
    pub idea_count: u64,
    pub stoke_count: u64,
}
