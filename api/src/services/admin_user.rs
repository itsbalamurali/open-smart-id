use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString, password_hash::rand_core::OsRng};
use chrono::Utc;
use sea_orm::*;

use crate::db::entities::{admin_user, admin_user_role, role};
use super::relying_party::ServiceError;

pub struct AdminUserService;

impl AdminUserService {
    super::impl_crud_basics!(admin_user::Entity, "Admin user");

    pub async fn list(
        db: &DatabaseConnection,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<admin_user::Model>, u64), ServiceError> {
        let paginator = admin_user::Entity::find()
            .order_by_desc(admin_user::Column::CreatedAt)
            .paginate(db, per_page);
        let total = paginator.num_items().await.map_err(ServiceError::Db)?;
        let items = paginator.fetch_page(page - 1).await.map_err(ServiceError::Db)?;
        Ok((items, total))
    }

    pub async fn find_by_email(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<admin_user::Model, ServiceError> {
        admin_user::Entity::find()
            .filter(admin_user::Column::Email.eq(email))
            .one(db)
            .await
            .map_err(ServiceError::Db)?
            .ok_or_else(|| ServiceError::NotFound("Admin user not found".into()))
    }

    pub async fn create(
        db: &DatabaseConnection,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> Result<admin_user::Model, ServiceError> {
        let password_hash = hash_password(password)?;
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();

        let model = admin_user::ActiveModel {
            id: Set(id.clone()),
            email: Set(email.to_string()),
            password_hash: Set(password_hash),
            display_name: Set(display_name.to_string()),
            is_active: Set(true),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };

        admin_user::Entity::insert(model).exec(db).await.map_err(ServiceError::Db)?;
        Self::find_by_id(db, &id).await
    }

    pub async fn update(
        db: &DatabaseConnection,
        id: &str,
        email: Option<String>,
        display_name: Option<String>,
        is_active: Option<bool>,
    ) -> Result<admin_user::Model, ServiceError> {
        let user = Self::find_by_id(db, id).await?;
        let mut active: admin_user::ActiveModel = user.into();
        if let Some(email) = email {
            active.email = Set(email);
        }
        if let Some(name) = display_name {
            active.display_name = Set(name);
        }
        if let Some(active_flag) = is_active {
            active.is_active = Set(active_flag);
        }
        active.updated_at = Set(Utc::now().into());
        active.update(db).await.map_err(ServiceError::Db)
    }

    pub async fn change_password(
        db: &DatabaseConnection,
        id: &str,
        new_password: &str,
    ) -> Result<(), ServiceError> {
        let user = Self::find_by_id(db, id).await?;
        let password_hash = hash_password(new_password)?;
        let mut active: admin_user::ActiveModel = user.into();
        active.password_hash = Set(password_hash);
        active.updated_at = Set(Utc::now().into());
        active.update(db).await.map_err(ServiceError::Db)?;
        Ok(())
    }

    pub async fn verify_password(
        db: &DatabaseConnection,
        email: &str,
        password: &str,
    ) -> Result<admin_user::Model, ServiceError> {
        let user = Self::find_by_email(db, email).await?;
        if !user.is_active {
            return Err(ServiceError::Forbidden("Account is disabled".into()));
        }
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| ServiceError::Forbidden(format!("invalid hash: {e}")))?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| ServiceError::Forbidden("Invalid email or password".into()))?;
        Ok(user)
    }

    /// Collect all permission strings for a user by joining through their roles.
    pub async fn get_permissions(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<String>, ServiceError> {
        let user_roles = admin_user_role::Entity::find()
            .filter(admin_user_role::Column::AdminUserId.eq(user_id))
            .all(db)
            .await
            .map_err(ServiceError::Db)?;

        let role_ids: Vec<String> = user_roles.iter().map(|ur| ur.role_id.clone()).collect();
        if role_ids.is_empty() {
            return Ok(vec![]);
        }

        let roles = role::Entity::find()
            .filter(role::Column::Id.is_in(&role_ids))
            .all(db)
            .await
            .map_err(ServiceError::Db)?;

        let mut permissions = std::collections::HashSet::new();
        for r in &roles {
            if let Ok(perms) = serde_json::from_str::<Vec<String>>(&r.permissions) {
                permissions.extend(perms);
            }
        }
        Ok(permissions.into_iter().collect())
    }

    pub async fn assign_role(
        db: &DatabaseConnection,
        user_id: &str,
        role_id: &str,
    ) -> Result<(), ServiceError> {
        // Verify both exist
        Self::find_by_id(db, user_id).await?;
        super::role::RoleService::find_by_id(db, role_id).await?;

        let id = uuid::Uuid::new_v4().to_string();
        let model = admin_user_role::ActiveModel {
            id: Set(id),
            admin_user_id: Set(user_id.to_string()),
            role_id: Set(role_id.to_string()),
            created_at: Set(Utc::now().into()),
        };
        admin_user_role::Entity::insert(model).exec(db).await.map_err(ServiceError::Db)?;
        Ok(())
    }

    pub async fn remove_role(
        db: &DatabaseConnection,
        user_id: &str,
        role_id: &str,
    ) -> Result<(), ServiceError> {
        let result = admin_user_role::Entity::delete_many()
            .filter(admin_user_role::Column::AdminUserId.eq(user_id))
            .filter(admin_user_role::Column::RoleId.eq(role_id))
            .exec(db)
            .await
            .map_err(ServiceError::Db)?;
        if result.rows_affected == 0 {
            return Err(ServiceError::NotFound("Role assignment not found".into()));
        }
        Ok(())
    }

    pub async fn get_roles(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<role::Model>, ServiceError> {
        let user_roles = admin_user_role::Entity::find()
            .filter(admin_user_role::Column::AdminUserId.eq(user_id))
            .all(db)
            .await
            .map_err(ServiceError::Db)?;

        let role_ids: Vec<String> = user_roles.iter().map(|ur| ur.role_id.clone()).collect();
        if role_ids.is_empty() {
            return Ok(vec![]);
        }

        role::Entity::find()
            .filter(role::Column::Id.is_in(&role_ids))
            .all(db)
            .await
            .map_err(ServiceError::Db)
    }
}

fn hash_password(password: &str) -> Result<String, ServiceError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| ServiceError::Forbidden(format!("password hashing failed: {e}")))
}
