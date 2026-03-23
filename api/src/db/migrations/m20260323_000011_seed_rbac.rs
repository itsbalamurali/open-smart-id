use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // All permissions as JSON array
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

        // Hash the default password using argon2
        use argon2::{Argon2, PasswordHasher, password_hash::SaltString, password_hash::rand_core::OsRng};
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(b"changeme123", &salt)
            .map_err(|e| DbErr::Custom(format!("password hash failed: {e}")))?
            .to_string();

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let role_id = uuid::Uuid::new_v4().to_string();
        let user_id = uuid::Uuid::new_v4().to_string();
        let user_role_id = uuid::Uuid::new_v4().to_string();

        // Insert super_admin role with all permissions
        db.execute_unprepared(&format!(
            "INSERT INTO roles (id, name, description, permissions, created_at, updated_at) VALUES ('{role_id}', 'super_admin', 'Full platform access', '{}', '{now}', '{now}')",
            all_permissions.to_string().replace('\'', "''")
        ))
        .await?;

        // Insert default admin user
        db.execute_unprepared(&format!(
            "INSERT INTO admin_users (id, email, password_hash, display_name, is_active, created_at, updated_at) VALUES ('{user_id}', 'admin@smartid.local', '{}', 'Super Admin', 1, '{now}', '{now}')",
            password_hash.replace('\'', "''")
        ))
        .await?;

        // Assign super_admin role to the default user
        db.execute_unprepared(&format!(
            "INSERT INTO admin_user_roles (id, admin_user_id, role_id, created_at) VALUES ('{user_role_id}', '{user_id}', '{role_id}', '{now}')"
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DELETE FROM admin_user_roles").await?;
        db.execute_unprepared("DELETE FROM admin_users WHERE email = 'admin@smartid.local'").await?;
        db.execute_unprepared("DELETE FROM roles WHERE name = 'super_admin'").await?;
        Ok(())
    }
}
