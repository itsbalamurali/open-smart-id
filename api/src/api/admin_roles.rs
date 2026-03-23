use poem::web::Data;
use poem_openapi::{Object, OpenApi, param::Path, param::Query, payload::Json};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::models::*;
use crate::services::auth::Permission;
use crate::services::audit_log::{AuditEntry, AuditLogService};
use crate::services::role::RoleService;

use super::admin::PaginatedResponse;
use super::admin_users::RoleResponse;
use super::auth_guard::{AdminBearerAuth, require_permission};

pub struct AdminRolesApi;

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct CreateRoleRequest {
    pub name: String,
    #[oai(skip_serializing_if_is_none)]
    pub description: Option<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct UpdateRoleRequest {
    #[oai(skip_serializing_if_is_none)]
    pub name: Option<String>,
    #[oai(skip_serializing_if_is_none)]
    pub description: Option<String>,
    #[oai(skip_serializing_if_is_none)]
    pub permissions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct PermissionItem {
    pub name: String,
}

fn audit(state: &AppState, auth: &AdminBearerAuth, action: &str, resource_id: &str) {
    let db = state.db.clone();
    let entry = AuditEntry {
        admin_user_id: Some(auth.0.sub.clone()),
        admin_user_email: Some(auth.0.email.clone()),
        action: action.to_string(),
        resource_type: Some("role".to_string()),
        resource_id: Some(resource_id.to_string()),
        details: None,
        ip_address: None,
    };
    tokio::spawn(async move { AuditLogService::log(&db, entry).await });
}

fn normalize_page(page: Option<u64>) -> u64 { page.unwrap_or(1).clamp(1, u64::MAX) }
fn normalize_per_page(per_page: Option<u64>) -> u64 { per_page.unwrap_or(20).clamp(1, 100) }

#[OpenApi(prefix_path = "/admin/roles", tag = "super::ApiTags::Admin")]
impl AdminRolesApi {
    /// List all roles
    #[oai(path = "/", method = "get", operation_id = "adminListRoles")]
    async fn list(
        &self,
        state: Data<&AppState>,
        auth: AdminBearerAuth,
        page: Query<Option<u64>>,
        per_page: Query<Option<u64>>,
    ) -> Result<Json<PaginatedResponse<RoleResponse>>, ApiErrorResponse> {
        require_permission(&auth.0, "roles:read")?;
        let page = normalize_page(page.0);
        let per_page = normalize_per_page(per_page.0);
        let (items, total) = RoleService::list(&state.db, page, per_page).await?;
        Ok(Json(PaginatedResponse {
            items: items.into_iter().map(Into::into).collect(),
            total, page, per_page,
        }))
    }

    /// Get role by ID
    #[oai(path = "/:id", method = "get", operation_id = "adminGetRole")]
    async fn get(
        &self,
        state: Data<&AppState>,
        auth: AdminBearerAuth,
        id: Path<String>,
    ) -> Result<Json<RoleResponse>, ApiErrorResponse> {
        require_permission(&auth.0, "roles:read")?;
        let r = RoleService::find_by_id(&state.db, &id.0).await?;
        Ok(Json(r.into()))
    }

    /// Create a role
    #[oai(path = "/", method = "post", operation_id = "adminCreateRole")]
    async fn create(
        &self,
        state: Data<&AppState>,
        auth: AdminBearerAuth,
        body: Json<CreateRoleRequest>,
    ) -> Result<Json<RoleResponse>, ApiErrorResponse> {
        require_permission(&auth.0, "roles:write")?;
        let r = RoleService::create(&state.db, &body.name, body.description.clone(), &body.permissions).await?;
        audit(&state, &auth, "role.create", &r.id);
        Ok(Json(r.into()))
    }

    /// Update a role
    #[oai(path = "/:id", method = "patch", operation_id = "adminUpdateRole")]
    async fn update(
        &self,
        state: Data<&AppState>,
        auth: AdminBearerAuth,
        id: Path<String>,
        body: Json<UpdateRoleRequest>,
    ) -> Result<Json<RoleResponse>, ApiErrorResponse> {
        require_permission(&auth.0, "roles:write")?;
        let r = RoleService::update(&state.db, &id.0, body.name.clone(), body.description.clone(), body.permissions.clone()).await?;
        audit(&state, &auth, "role.update", &id.0);
        Ok(Json(r.into()))
    }

    /// Delete a role
    #[oai(path = "/:id", method = "delete", operation_id = "adminDeleteRole")]
    async fn delete(
        &self,
        state: Data<&AppState>,
        auth: AdminBearerAuth,
        id: Path<String>,
    ) -> Result<Json<serde_json::Value>, ApiErrorResponse> {
        require_permission(&auth.0, "roles:write")?;
        RoleService::delete(&state.db, &id.0).await?;
        audit(&state, &auth, "role.delete", &id.0);
        Ok(Json(serde_json::json!({ "deleted": true })))
    }

    /// List all available permissions
    #[oai(path = "/permissions", method = "get", operation_id = "adminListPermissions")]
    async fn list_permissions(
        &self,
        auth: AdminBearerAuth,
    ) -> Result<Json<Vec<PermissionItem>>, ApiErrorResponse> {
        require_permission(&auth.0, "roles:read")?;
        let items = Permission::all()
            .iter()
            .map(|p| PermissionItem {
                name: serde_json::to_value(p)
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_default(),
            })
            .collect();
        Ok(Json(items))
    }
}
