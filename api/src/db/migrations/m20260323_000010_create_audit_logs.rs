use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AuditLogs::Table)
                    .if_not_exists()
                    .col(string(AuditLogs::Id).primary_key())
                    .col(string_null(AuditLogs::AdminUserId))
                    .col(string_null(AuditLogs::AdminUserEmail))
                    .col(string(AuditLogs::Action))
                    .col(string_null(AuditLogs::ResourceType))
                    .col(string_null(AuditLogs::ResourceId))
                    .col(text_null(AuditLogs::Details))
                    .col(string_null(AuditLogs::IpAddress))
                    .col(timestamp(AuditLogs::CreatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(AuditLogs::Table)
                    .name("idx_audit_logs_action")
                    .col(AuditLogs::Action)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(AuditLogs::Table)
                    .name("idx_audit_logs_created_at")
                    .col(AuditLogs::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuditLogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AuditLogs {
    Table,
    Id,
    AdminUserId,
    AdminUserEmail,
    Action,
    ResourceType,
    ResourceId,
    Details,
    IpAddress,
    CreatedAt,
}
