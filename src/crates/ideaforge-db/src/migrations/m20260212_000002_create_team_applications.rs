use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TYPE application_status AS ENUM ('pending', 'accepted', 'rejected', 'withdrawn')",
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TeamApplications::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TeamApplications::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TeamApplications::IdeaId).uuid().not_null())
                    .col(ColumnDef::new(TeamApplications::UserId).uuid().not_null())
                    .col(ColumnDef::new(TeamApplications::Message).text().not_null())
                    .col(
                        ColumnDef::new(TeamApplications::Status)
                            .custom(Alias::new("application_status"))
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(TeamApplications::ReviewedBy).uuid())
                    .col(
                        ColumnDef::new(TeamApplications::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(TeamApplications::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_team_applications_idea")
                            .from(TeamApplications::Table, TeamApplications::IdeaId)
                            .to(Alias::new("ideas"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_team_applications_user")
                            .from(TeamApplications::Table, TeamApplications::UserId)
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_team_applications_reviewer")
                            .from(TeamApplications::Table, TeamApplications::ReviewedBy)
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Index on (idea_id, status)
        manager
            .create_index(
                Index::create()
                    .name("idx_team_applications_idea_status")
                    .table(TeamApplications::Table)
                    .col(TeamApplications::IdeaId)
                    .col(TeamApplications::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TeamApplications::Table).to_owned())
            .await?;
        manager
            .get_connection()
            .execute_unprepared("DROP TYPE IF EXISTS application_status")
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum TeamApplications {
    Table,
    Id,
    IdeaId,
    UserId,
    Message,
    Status,
    ReviewedBy,
    CreatedAt,
    UpdatedAt,
}
