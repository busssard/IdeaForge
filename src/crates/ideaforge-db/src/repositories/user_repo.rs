use ideaforge_core::User;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

/// Repository for user-related database operations.
pub struct UserRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> UserRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, _id: Uuid) -> Result<Option<User>, sea_orm::DbErr> {
        todo!("Implement after running sea-orm-cli generate entity")
    }

    pub async fn find_by_email(&self, _email: &str) -> Result<Option<User>, sea_orm::DbErr> {
        todo!("Implement after running sea-orm-cli generate entity")
    }
}
