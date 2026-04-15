use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create enum types
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TYPE idea_maturity AS ENUM ('spark', 'building', 'in_work')",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TYPE idea_openness AS ENUM ('open', 'collaborative', 'commercial')",
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Ideas::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Ideas::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Ideas::AuthorId).uuid().not_null())
                    .col(ColumnDef::new(Ideas::Title).string_len(200).not_null())
                    .col(ColumnDef::new(Ideas::Summary).string_len(500).not_null())
                    .col(ColumnDef::new(Ideas::Description).text().not_null())
                    .col(
                        ColumnDef::new(Ideas::Maturity)
                            .custom(Alias::new("idea_maturity"))
                            .not_null()
                            .default("spark"),
                    )
                    .col(
                        ColumnDef::new(Ideas::Openness)
                            .custom(Alias::new("idea_openness"))
                            .not_null()
                            .default("open"),
                    )
                    .col(ColumnDef::new(Ideas::CategoryId).uuid())
                    .col(
                        ColumnDef::new(Ideas::StokeCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Ideas::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Ideas::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Ideas::ArchivedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ideas_author")
                            .from(Ideas::Table, Ideas::AuthorId)
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ideas_category")
                            .from(Ideas::Table, Ideas::CategoryId)
                            .to(Alias::new("categories"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Indexes for common queries
        manager
            .create_index(
                Index::create()
                    .name("idx_ideas_author")
                    .table(Ideas::Table)
                    .col(Ideas::AuthorId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ideas_category")
                    .table(Ideas::Table)
                    .col(Ideas::CategoryId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ideas_maturity")
                    .table(Ideas::Table)
                    .col(Ideas::Maturity)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Ideas::Table).to_owned())
            .await?;
        manager
            .get_connection()
            .execute_unprepared("DROP TYPE IF EXISTS idea_openness")
            .await?;
        manager
            .get_connection()
            .execute_unprepared("DROP TYPE IF EXISTS idea_maturity")
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Ideas {
    Table,
    Id,
    AuthorId,
    Title,
    Summary,
    Description,
    Maturity,
    Openness,
    CategoryId,
    StokeCount,
    CreatedAt,
    UpdatedAt,
    ArchivedAt,
}
