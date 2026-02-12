use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Categories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Categories::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Categories::Name)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Categories::Slug)
                            .string_len(100)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Categories::Description)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(ColumnDef::new(Categories::Icon).string_len(50))
                    .col(ColumnDef::new(Categories::ParentId).uuid())
                    .col(
                        ColumnDef::new(Categories::SortOrder)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Categories::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_categories_parent")
                            .from(Categories::Table, Categories::ParentId)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Categories::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Categories {
    Table,
    Id,
    Name,
    Slug,
    Description,
    Icon,
    ParentId,
    SortOrder,
    CreatedAt,
}
