use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Devices::Table)
                    .if_not_exists()
                    .col(string(Devices::Id).primary_key())
                    .col(string(Devices::AccountId))
                    .col(text(Devices::FcmToken))
                    .col(string_null(Devices::DeviceName))
                    .col(string(Devices::Platform))
                    .col(boolean(Devices::IsActive).default(true))
                    .col(timestamp(Devices::CreatedAt))
                    .col(timestamp(Devices::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Devices::Table, Devices::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Devices::Table)
                    .name("idx_devices_account_id")
                    .col(Devices::AccountId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Devices::Table)
                    .name("idx_devices_fcm_token")
                    .col(Devices::FcmToken)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Devices::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Devices {
    Table,
    Id,
    AccountId,
    FcmToken,
    DeviceName,
    Platform,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,
}
