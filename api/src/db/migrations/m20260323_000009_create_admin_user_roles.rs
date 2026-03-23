use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AdminUserRoles::Table)
                    .if_not_exists()
                    .col(string(AdminUserRoles::Id).primary_key())
                    .col(string(AdminUserRoles::AdminUserId))
                    .col(string(AdminUserRoles::RoleId))
                    .col(timestamp(AdminUserRoles::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .from(AdminUserRoles::Table, AdminUserRoles::AdminUserId)
                            .to(AdminUsers::Table, AdminUsers::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(AdminUserRoles::Table, AdminUserRoles::RoleId)
                            .to(Roles::Table, Roles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(AdminUserRoles::Table)
                    .name("idx_admin_user_roles_unique")
                    .col(AdminUserRoles::AdminUserId)
                    .col(AdminUserRoles::RoleId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AdminUserRoles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AdminUserRoles {
    Table,
    Id,
    AdminUserId,
    RoleId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AdminUsers {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Roles {
    Table,
    Id,
}
