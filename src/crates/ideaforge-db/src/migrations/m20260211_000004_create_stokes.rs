use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Stokes::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Stokes::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Stokes::UserId).uuid().not_null())
                    .col(ColumnDef::new(Stokes::IdeaId).uuid().not_null())
                    .col(
                        ColumnDef::new(Stokes::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_stokes_user")
                            .from(Stokes::Table, Stokes::UserId)
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_stokes_idea")
                            .from(Stokes::Table, Stokes::IdeaId)
                            .to(Alias::new("ideas"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique constraint: one stoke per user per idea
        manager
            .create_index(
                Index::create()
                    .name("idx_stokes_user_idea_unique")
                    .table(Stokes::Table)
                    .col(Stokes::UserId)
                    .col(Stokes::IdeaId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Index for counting stokes per idea
        manager
            .create_index(
                Index::create()
                    .name("idx_stokes_idea")
                    .table(Stokes::Table)
                    .col(Stokes::IdeaId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Stokes::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Stokes {
    Table,
    Id,
    UserId,
    IdeaId,
    CreatedAt,
}
