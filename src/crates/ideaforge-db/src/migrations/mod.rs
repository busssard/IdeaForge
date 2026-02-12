use sea_orm_migration::prelude::*;

mod m20260211_000001_create_users;
mod m20260211_000002_create_categories;
mod m20260211_000003_create_ideas;
mod m20260211_000004_create_stokes;
mod m20260211_000005_create_contributions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260211_000001_create_users::Migration),
            Box::new(m20260211_000002_create_categories::Migration),
            Box::new(m20260211_000003_create_ideas::Migration),
            Box::new(m20260211_000004_create_stokes::Migration),
            Box::new(m20260211_000005_create_contributions::Migration),
        ]
    }
}
