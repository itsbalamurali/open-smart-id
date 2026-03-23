use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RelyingParties::Table)
                    .if_not_exists()
                    .col(string(RelyingParties::Id).primary_key())
                    .col(string_uniq(RelyingParties::Uuid))
                    .col(string(RelyingParties::Name))
                    .col(boolean(RelyingParties::IsActive).default(true))
                    .col(timestamp(RelyingParties::CreatedAt))
                    .col(timestamp(RelyingParties::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RelyingParties::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RelyingParties {
    Table,
    Id,
    Uuid,
    Name,
    IsActive,
    CreatedAt,
    UpdatedAt,
}
