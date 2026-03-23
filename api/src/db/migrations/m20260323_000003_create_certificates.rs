use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Certificates::Table)
                    .if_not_exists()
                    .col(string(Certificates::Id).primary_key())
                    .col(string(Certificates::AccountId))
                    .col(string(Certificates::CertType))
                    .col(text(Certificates::CertValue))
                    .col(string(Certificates::CertLevel))
                    .col(text(Certificates::KeyPairPem))
                    .col(boolean(Certificates::IsActive).default(true))
                    .col(timestamp(Certificates::CreatedAt))
                    .col(timestamp(Certificates::ExpiresAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Certificates::Table, Certificates::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Certificates::Table)
                    .name("idx_certificates_account_id")
                    .col(Certificates::AccountId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Certificates::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Certificates {
    Table,
    Id,
    AccountId,
    CertType,
    CertValue,
    CertLevel,
    KeyPairPem,
    IsActive,
    CreatedAt,
    ExpiresAt,
}

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,
}
