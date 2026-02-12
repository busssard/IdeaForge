use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TYPE contribution_type AS ENUM ('comment', 'suggestion', 'design', 'code', 'research', 'other')",
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Contributions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Contributions::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Contributions::IdeaId).uuid().not_null())
                    .col(ColumnDef::new(Contributions::UserId).uuid().not_null())
                    .col(ColumnDef::new(Contributions::ParentId).uuid())
                    .col(
                        ColumnDef::new(Contributions::ContributionType)
                            .custom(Alias::new("contribution_type"))
                            .not_null()
                            .default("comment"),
                    )
                    .col(ColumnDef::new(Contributions::Title).string_len(200))
                    .col(ColumnDef::new(Contributions::Body).text().not_null())
                    .col(
                        ColumnDef::new(Contributions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Contributions::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_contributions_idea")
                            .from(Contributions::Table, Contributions::IdeaId)
                            .to(Alias::new("ideas"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_contributions_user")
                            .from(Contributions::Table, Contributions::UserId)
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_contributions_parent")
                            .from(Contributions::Table, Contributions::ParentId)
                            .to(Contributions::Table, Contributions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_contributions_idea")
                    .table(Contributions::Table)
                    .col(Contributions::IdeaId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Contributions::Table).to_owned())
            .await?;
        manager
            .get_connection()
            .execute_unprepared("DROP TYPE IF EXISTS contribution_type")
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Contributions {
    Table,
    Id,
    IdeaId,
    UserId,
    ParentId,
    ContributionType,
    Title,
    Body,
    CreatedAt,
    UpdatedAt,
}
