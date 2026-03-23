use chrono::Utc;
use sea_orm::*;

use crate::db::entities::role;
use super::relying_party::ServiceError;

pub struct RoleService;

impl RoleService {
    super::impl_crud_basics!(role::Entity, "Role");

    pub async fn list(
        db: &DatabaseConnection,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<role::Model>, u64), ServiceError> {
        let paginator = role::Entity::find()
            .order_by_desc(role::Column::CreatedAt)
            .paginate(db, per_page);
        let total = paginator.num_items().await.map_err(ServiceError::Db)?;
        let items = paginator.fetch_page(page - 1).await.map_err(ServiceError::Db)?;
        Ok((items, total))
    }

    pub async fn create(
        db: &DatabaseConnection,
        name: &str,
        description: Option<String>,
        permissions: &[String],
    ) -> Result<role::Model, ServiceError> {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();

        let model = role::ActiveModel {
            id: Set(id.clone()),
            name: Set(name.to_string()),
            description: Set(description),
            permissions: Set(serde_json::to_string(permissions).unwrap_or_else(|_| "[]".to_string())),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };

        role::Entity::insert(model).exec(db).await.map_err(ServiceError::Db)?;
        Self::find_by_id(db, &id).await
    }

    pub async fn update(
        db: &DatabaseConnection,
        id: &str,
        name: Option<String>,
        description: Option<String>,
        permissions: Option<Vec<String>>,
    ) -> Result<role::Model, ServiceError> {
        let r = Self::find_by_id(db, id).await?;
        let mut active: role::ActiveModel = r.into();
        if let Some(name) = name {
            active.name = Set(name);
        }
        if description.is_some() {
            active.description = Set(description);
        }
        if let Some(perms) = permissions {
            active.permissions = Set(serde_json::to_string(&perms).unwrap_or_else(|_| "[]".to_string()));
        }
        active.updated_at = Set(Utc::now().into());
        active.update(db).await.map_err(ServiceError::Db)
    }

    /// Parse the JSON permissions column into a list of strings.
    pub fn parse_permissions(role: &role::Model) -> Vec<String> {
        serde_json::from_str(&role.permissions).unwrap_or_default()
    }
}
