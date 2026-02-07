use ideaforge_core::Pledge;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

/// Repository for pledge-related database operations.
pub struct PledgeRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> PledgeRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_by_idea(&self, _idea_id: Uuid) -> Result<Vec<Pledge>, sea_orm::DbErr> {
        todo!("Implement after running sea-orm-cli generate entity")
    }

    pub async fn find_by_user(&self, _user_id: Uuid) -> Result<Vec<Pledge>, sea_orm::DbErr> {
        todo!("Implement after running sea-orm-cli generate entity")
    }
}
