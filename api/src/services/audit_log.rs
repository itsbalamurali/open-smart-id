use chrono::Utc;
use sea_orm::*;

use crate::db::entities::audit_log;
use super::relying_party::ServiceError;

pub struct AuditEntry {
    pub admin_user_id: Option<String>,
    pub admin_user_email: Option<String>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub details: Option<String>,
    pub ip_address: Option<String>,
}

pub struct AuditLogService;

impl AuditLogService {
    pub async fn list(
        db: &DatabaseConnection,
        page: u64,
        per_page: u64,
        action_filter: Option<&str>,
        user_id_filter: Option<&str>,
        resource_type_filter: Option<&str>,
    ) -> Result<(Vec<audit_log::Model>, u64), ServiceError> {
        let mut query = audit_log::Entity::find().order_by_desc(audit_log::Column::CreatedAt);
        if let Some(action) = action_filter {
            query = query.filter(audit_log::Column::Action.eq(action));
        }
        if let Some(uid) = user_id_filter {
            query = query.filter(audit_log::Column::AdminUserId.eq(uid));
        }
        if let Some(rt) = resource_type_filter {
            query = query.filter(audit_log::Column::ResourceType.eq(rt));
        }
        let paginator = query.paginate(db, per_page);
        let total = paginator.num_items().await.map_err(ServiceError::Db)?;
        let items = paginator.fetch_page(page - 1).await.map_err(ServiceError::Db)?;
        Ok((items, total))
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: &str,
    ) -> Result<audit_log::Model, ServiceError> {
        audit_log::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(ServiceError::Db)?
            .ok_or_else(|| ServiceError::NotFound("Audit log not found".into()))
    }

    /// Log an audit entry. Errors are traced but not propagated.
    pub async fn log(db: &DatabaseConnection, entry: AuditEntry) {
        let id = uuid::Uuid::new_v4().to_string();
        let model = audit_log::ActiveModel {
            id: Set(id),
            admin_user_id: Set(entry.admin_user_id),
            admin_user_email: Set(entry.admin_user_email),
            action: Set(entry.action),
            resource_type: Set(entry.resource_type),
            resource_id: Set(entry.resource_id),
            details: Set(entry.details),
            ip_address: Set(entry.ip_address),
            created_at: Set(Utc::now().into()),
        };
        if let Err(e) = audit_log::Entity::insert(model).exec(db).await {
            tracing::error!("Failed to write audit log: {e}");
        }
    }
}
