use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Accounts::Table)
                    .if_not_exists()
                    .col(string(Accounts::Id).primary_key())
                    .col(string_null(Accounts::SemanticId).unique_key())
                    .col(string_uniq(Accounts::DocumentNumber))
                    .col(string_null(Accounts::IdentityType))
                    .col(string_null(Accounts::CountryCode))
                    .col(string_null(Accounts::NationalIdentityNumber))
                    .col(string(Accounts::Status).default("active"))
                    .col(timestamp(Accounts::CreatedAt))
                    .col(timestamp(Accounts::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Accounts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,
    SemanticId,
    DocumentNumber,
    IdentityType,
    CountryCode,
    NationalIdentityNumber,
    Status,
    CreatedAt,
    UpdatedAt,
}
