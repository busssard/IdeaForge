use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // --- 1. Extend idea_openness enum with 'nda_protected' ---
        conn.execute_unprepared("ALTER TYPE idea_openness ADD VALUE IF NOT EXISTS 'nda_protected'")
            .await?;

        // --- 2. Create nda_templates table ---
        manager
            .create_table(
                Table::create()
                    .table(NdaTemplates::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(NdaTemplates::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(NdaTemplates::IdeaId).uuid().not_null().unique_key())
                    .col(
                        ColumnDef::new(NdaTemplates::Title)
                            .string_len(255)
                            .not_null()
                            .default("Standard Non-Disclosure Agreement"),
                    )
                    .col(ColumnDef::new(NdaTemplates::Body).text().not_null())
                    .col(
                        ColumnDef::new(NdaTemplates::ConfidentialityPeriodDays)
                            .integer()
                            .not_null()
                            .default(730),
                    )
                    .col(ColumnDef::new(NdaTemplates::Jurisdiction).string_len(100))
                    .col(ColumnDef::new(NdaTemplates::CreatedBy).uuid().not_null())
                    .col(
                        ColumnDef::new(NdaTemplates::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(NdaTemplates::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nda_templates_idea")
                            .from(NdaTemplates::Table, NdaTemplates::IdeaId)
                            .to(Alias::new("ideas"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nda_templates_creator")
                            .from(NdaTemplates::Table, NdaTemplates::CreatedBy)
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        // --- 3. Create nda_signatures table ---
        manager
            .create_table(
                Table::create()
                    .table(NdaSignatures::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(NdaSignatures::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(NdaSignatures::NdaTemplateId).uuid().not_null())
                    .col(ColumnDef::new(NdaSignatures::IdeaId).uuid().not_null())
                    .col(ColumnDef::new(NdaSignatures::SignerId).uuid().not_null())
                    .col(ColumnDef::new(NdaSignatures::SignerName).string_len(255).not_null())
                    .col(ColumnDef::new(NdaSignatures::SignerEmail).string_len(255).not_null())
                    .col(ColumnDef::new(NdaSignatures::IpAddress).string_len(45))
                    .col(
                        ColumnDef::new(NdaSignatures::SignedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(NdaSignatures::ExpiresAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(NdaSignatures::RevokedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nda_signatures_template")
                            .from(NdaSignatures::Table, NdaSignatures::NdaTemplateId)
                            .to(NdaTemplates::Table, NdaTemplates::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nda_signatures_idea")
                            .from(NdaSignatures::Table, NdaSignatures::IdeaId)
                            .to(Alias::new("ideas"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nda_signatures_signer")
                            .from(NdaSignatures::Table, NdaSignatures::SignerId)
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // --- 4. Unique constraint: one signature per person per NDA ---
        manager
            .create_index(
                Index::create()
                    .name("idx_nda_signatures_unique")
                    .table(NdaSignatures::Table)
                    .col(NdaSignatures::NdaTemplateId)
                    .col(NdaSignatures::SignerId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // --- 5. Indexes ---
        manager
            .create_index(
                Index::create()
                    .name("idx_nda_signatures_idea")
                    .table(NdaSignatures::Table)
                    .col(NdaSignatures::IdeaId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_nda_signatures_signer")
                    .table(NdaSignatures::Table)
                    .col(NdaSignatures::SignerId)
                    .to_owned(),
            )
            .await?;

        // --- 6. Extend notification_kind enum with 'nda_signed' ---
        conn.execute_unprepared("ALTER TYPE notification_kind ADD VALUE IF NOT EXISTS 'nda_signed'")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NdaSignatures::Table).if_exists().to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(NdaTemplates::Table).if_exists().to_owned())
            .await?;

        // Note: Cannot remove enum values in PostgreSQL, only drop+recreate

        Ok(())
    }
}

#[derive(DeriveIden)]
enum NdaTemplates {
    Table,
    Id,
    IdeaId,
    Title,
    Body,
    ConfidentialityPeriodDays,
    Jurisdiction,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum NdaSignatures {
    Table,
    Id,
    NdaTemplateId,
    IdeaId,
    SignerId,
    SignerName,
    SignerEmail,
    IpAddress,
    SignedAt,
    ExpiresAt,
    RevokedAt,
}
