use sea_orm_migration::prelude::*;

mod m20260211_000001_create_users;
mod m20260211_000002_create_categories;
mod m20260211_000003_create_ideas;
mod m20260211_000004_create_stokes;
mod m20260211_000005_create_contributions;
mod m20260212_000001_create_team_members;
mod m20260212_000002_create_team_applications;
mod m20260212_000003_create_subscriptions;
mod m20260212_100001_phase2_extensions;
mod m20260325_000001_nda_system;
mod m20260325_000002_task_boards_and_team_labels;
mod m20260325_000003_task_budget;
mod m20260414_000001_idea_lifecycle;
mod m20260414_000002_mls_messaging;
mod m20260414_000003_mls_keystore;
mod m20260414_000004_message_notification;
mod m20260414_000005_notification_related_user;
mod m20260416_000001_profile_locations_education;

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
            Box::new(m20260212_000001_create_team_members::Migration),
            Box::new(m20260212_000002_create_team_applications::Migration),
            Box::new(m20260212_000003_create_subscriptions::Migration),
            Box::new(m20260212_100001_phase2_extensions::Migration),
            Box::new(m20260325_000001_nda_system::Migration),
            Box::new(m20260325_000002_task_boards_and_team_labels::Migration),
            Box::new(m20260325_000003_task_budget::Migration),
            Box::new(m20260414_000001_idea_lifecycle::Migration),
            Box::new(m20260414_000002_mls_messaging::Migration),
            Box::new(m20260414_000003_mls_keystore::Migration),
            Box::new(m20260414_000004_message_notification::Migration),
            Box::new(m20260414_000005_notification_related_user::Migration),
            Box::new(m20260416_000001_profile_locations_education::Migration),
        ]
    }
}
