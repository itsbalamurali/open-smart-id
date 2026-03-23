use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AdminUsers::Table)
                    .if_not_exists()
                    .col(string(AdminUsers::Id).primary_key())
                    .col(string_uniq(AdminUsers::Email))
                    .col(text(AdminUsers::PasswordHash))
                    .col(string(AdminUsers::DisplayName))
                    .col(boolean(AdminUsers::IsActive).default(true))
                    .col(timestamp(AdminUsers::CreatedAt))
                    .col(timestamp(AdminUsers::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AdminUsers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AdminUsers {
    Table,
    Id,
    Email,
    PasswordHash,
    DisplayName,
    IsActive,
    CreatedAt,
    UpdatedAt,
}
