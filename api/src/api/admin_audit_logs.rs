use poem::web::Data;
use poem_openapi::{Object, OpenApi, param::Path, param::Query, payload::Json};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::db::entities::audit_log;
use crate::models::*;
use crate::services::audit_log::AuditLogService;

use super::admin::PaginatedResponse;
use super::auth_guard::{AdminBearerAuth, require_permission};

pub struct AdminAuditLogsApi;

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct AuditLogResponse {
    pub id: String,
    #[oai(rename = "adminUserId", skip_serializing_if_is_none)]
    pub admin_user_id: Option<String>,
    #[oai(rename = "adminUserEmail", skip_serializing_if_is_none)]
    pub admin_user_email: Option<String>,
    pub action: String,
    #[oai(rename = "resourceType", skip_serializing_if_is_none)]
    pub resource_type: Option<String>,
    #[oai(rename = "resourceId", skip_serializing_if_is_none)]
    pub resource_id: Option<String>,
    #[oai(skip_serializing_if_is_none)]
    pub details: Option<String>,
    #[oai(rename = "ipAddress", skip_serializing_if_is_none)]
    pub ip_address: Option<String>,
    #[oai(rename = "createdAt")]
    pub created_at: String,
}

impl From<audit_log::Model> for AuditLogResponse {
    fn from(m: audit_log::Model) -> Self {
        Self {
            id: m.id,
            admin_user_id: m.admin_user_id,
            admin_user_email: m.admin_user_email,
            action: m.action,
            resource_type: m.resource_type,
            resource_id: m.resource_id,
            details: m.details,
            ip_address: m.ip_address,
            created_at: m.created_at.to_rfc3339(),
        }
    }
}

fn normalize_page(page: Option<u64>) -> u64 { page.unwrap_or(1).clamp(1, u64::MAX) }
fn normalize_per_page(per_page: Option<u64>) -> u64 { per_page.unwrap_or(20).clamp(1, 100) }

#[OpenApi(prefix_path = "/admin/audit-logs", tag = "super::ApiTags::Admin")]
impl AdminAuditLogsApi {
    /// List audit logs
    #[oai(path = "/", method = "get", operation_id = "adminListAuditLogs")]
    async fn list(
        &self,
        state: Data<&AppState>,
        auth: AdminBearerAuth,
        page: Query<Option<u64>>,
        per_page: Query<Option<u64>>,
        #[oai(name = "action")]
        action: Query<Option<String>>,
        #[oai(name = "userId")]
        user_id: Query<Option<String>>,
        #[oai(name = "resourceType")]
        resource_type: Query<Option<String>>,
    ) -> Result<Json<PaginatedResponse<AuditLogResponse>>, ApiErrorResponse> {
        require_permission(&auth.0, "audit_logs:read")?;
        let page = normalize_page(page.0);
        let per_page = normalize_per_page(per_page.0);
        let (items, total) = AuditLogService::list(
            &state.db,
            page,
            per_page,
            action.0.as_deref(),
            user_id.0.as_deref(),
            resource_type.0.as_deref(),
        )
        .await?;
        Ok(Json(PaginatedResponse {
            items: items.into_iter().map(Into::into).collect(),
            total, page, per_page,
        }))
    }

    /// Get audit log entry by ID
    #[oai(path = "/:id", method = "get", operation_id = "adminGetAuditLog")]
    async fn get(
        &self,
        state: Data<&AppState>,
        auth: AdminBearerAuth,
        id: Path<String>,
    ) -> Result<Json<AuditLogResponse>, ApiErrorResponse> {
        require_permission(&auth.0, "audit_logs:read")?;
        let log = AuditLogService::find_by_id(&state.db, &id.0).await?;
        Ok(Json(log.into()))
    }
}
