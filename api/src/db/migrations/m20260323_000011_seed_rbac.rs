use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use argon2::{Argon2, PasswordHasher, password_hash::SaltString, password_hash::rand_core::OsRng};

        let all_permissions = serde_json::json!([
            "admin_users:read", "admin_users:write",
            "roles:read", "roles:write",
            "audit_logs:read",
            "relying_parties:read", "relying_parties:write",
            "accounts:read", "accounts:write",
            "sessions:read", "sessions:write",
            "devices:read", "devices:write",
            "certificates:read", "certificates:write"
        ]);

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(b"changeme123", &salt)
            .map_err(|e| DbErr::Custom(format!("password hash failed: {e}")))?
            .to_string();

        let now = chrono::Utc::now().to_rfc3339();
        let role_id = uuid::Uuid::new_v4().to_string();
        let user_id = uuid::Uuid::new_v4().to_string();
        let user_role_id = uuid::Uuid::new_v4().to_string();

        // Insert super_admin role
        manager
            .exec_stmt(
                Query::insert()
                    .into_table(Roles::Table)
                    .columns([
                        Roles::Id,
                        Roles::Name,
                        Roles::Description,
                        Roles::Permissions,
                        Roles::CreatedAt,
                        Roles::UpdatedAt,
                    ])
                    .values_panic([
                        role_id.clone().into(),
                        "super_admin".into(),
                        "Full platform access".into(),
                        all_permissions.to_string().into(),
                        now.clone().into(),
                        now.clone().into(),
                    ])
                    .to_owned(),
            )
            .await?;

        // Insert default admin user
        manager
            .exec_stmt(
                Query::insert()
                    .into_table(AdminUsers::Table)
                    .columns([
                        AdminUsers::Id,
                        AdminUsers::Email,
                        AdminUsers::PasswordHash,
                        AdminUsers::DisplayName,
                        AdminUsers::IsActive,
                        AdminUsers::CreatedAt,
                        AdminUsers::UpdatedAt,
                    ])
                    .values_panic([
                        user_id.clone().into(),
                        "admin@smartid.local".into(),
                        password_hash.into(),
                        "Super Admin".into(),
                        true.into(),
                        now.clone().into(),
                        now.clone().into(),
                    ])
                    .to_owned(),
            )
            .await?;

        // Assign super_admin role to the default user
        manager
            .exec_stmt(
                Query::insert()
                    .into_table(AdminUserRoles::Table)
                    .columns([
                        AdminUserRoles::Id,
                        AdminUserRoles::AdminUserId,
                        AdminUserRoles::RoleId,
                        AdminUserRoles::CreatedAt,
                    ])
                    .values_panic([
                        user_role_id.into(),
                        user_id.into(),
                        role_id.into(),
                        now.into(),
                    ])
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .exec_stmt(
                Query::delete()
                    .from_table(AdminUserRoles::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::delete()
                    .from_table(AdminUsers::Table)
                    .and_where(Expr::col(AdminUsers::Email).eq("admin@smartid.local"))
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::delete()
                    .from_table(Roles::Table)
                    .and_where(Expr::col(Roles::Name).eq("super_admin"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Roles {
    Table,
    Id,
    Name,
    Description,
    Permissions,
    CreatedAt,
    UpdatedAt,
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

#[derive(DeriveIden)]
enum AdminUserRoles {
    Table,
    Id,
    AdminUserId,
    RoleId,
    CreatedAt,
}
