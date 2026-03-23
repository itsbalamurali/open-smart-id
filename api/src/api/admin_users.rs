use poem::web::Data;
use poem_openapi::{Object, OpenApi, param::Path, param::Query, payload::Json};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::db::entities::{admin_user, role};
use crate::models::*;
use crate::services::admin_user::AdminUserService;
use crate::services::audit_log::{AuditEntry, AuditLogService};

use super::admin::PaginatedResponse;
use super::auth_guard::{AdminBearerAuth, require_permission};

pub struct AdminUsersApi;

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct AdminUserResponse {
    pub id: String,
    pub email: String,
    #[oai(rename = "displayName")]
    pub display_name: String,
    #[oai(rename = "isActive")]
    pub is_active: bool,
    #[oai(rename = "createdAt")]
    pub created_at: String,
    #[oai(rename = "updatedAt")]
    pub updated_at: String,
}

impl From<admin_user::Model> for AdminUserResponse {
    fn from(m: admin_user::Model) -> Self {
        Self {
            id: m.id,
            email: m.email,
            display_name: m.display_name,
            is_active: m.is_active,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct CreateAdminUserRequest {
    pub email: String,
    pub password: String,
    #[oai(rename = "displayName")]
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct UpdateAdminUserRequest {
    #[oai(skip_serializing_if_is_none)]
    pub email: Option<String>,
    #[oai(rename = "displayName", skip_serializing_if_is_none)]
    pub display_name: Option<String>,
    #[oai(rename = "isActive", skip_serializing_if_is_none)]
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct ChangePasswordRequest {
    #[oai(rename = "newPassword")]
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct AssignRoleRequest {
    #[oai(rename = "roleId")]
    pub role_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct RoleResponse {
    pub id: String,
    pub name: String,
    #[oai(skip_serializing_if_is_none)]
    pub description: Option<String>,
    pub permissions: Vec<String>,
    #[oai(rename = "createdAt")]
    pub created_at: String,
}

impl From<role::Model> for RoleResponse {
    fn from(m: role::Model) -> Self {
        let permissions: Vec<String> = serde_json::from_str(&m.permissions).unwrap_or_default();
        Self {
            id: m.id,
            name: m.name,
            description: m.description,
            permissions,
            created_at: m.created_at.to_rfc3339(),
        }
    }
}

fn audit(state: &AppState, auth: &AdminBearerAuth, action: &str, resource_type: &str, resource_id: &str) {
    let db = state.db.clone();
    let entry = AuditEntry {
        admin_user_id: Some(auth.0.sub.clone()),
        admin_user_email: Some(auth.0.email.clone()),
        action: action.to_string(),
        resource_type: Some(resource_type.to_string()),
        resource_id: Some(resource_id.to_string()),
        details: None,
        ip_address: None,
    };
    tokio::spawn(async move { AuditLogService::log(&db, entry).await });
}

fn normalize_page(page: Option<u64>) -> u64 { page.unwrap_or(1).clamp(1, u64::MAX) }
fn normalize_per_page(per_page: Option<u64>) -> u64 { per_page.unwrap_or(20).clamp(1, 100) }

#[OpenApi(prefix_path = "/admin/users", tag = "super::ApiTags::Admin")]
impl AdminUsersApi {
    /// List admin users
    #[oai(path = "/", method = "get", operation_id = "adminListUsers")]
    async fn list(
        &self,
        state: Data<&AppState>,
        auth: AdminBearerAuth,
        page: Query<Option<u64>>,
        per_page: Query<Option<u64>>,
    ) -> Result<Json<PaginatedResponse<AdminUserResponse>>, ApiErrorResponse> {
        require_permission(&auth.0, "admin_users:read")?;
        let page = normalize_page(page.0);
        let per_page = normalize_per_page(per_page.0);
        let (items, total) = AdminUserService::list(&state.db, page, per_page).await?;
        Ok(Json(PaginatedResponse {
            items: items.into_iter().map(Into::into).collect(),
            total, page, per_page,
        }))
    }

    /// Get admin user by ID
    #[oai(path = "/:id", method = "get", operation_id = "adminGetUser")]
    async fn get(
        &self,
        state: Data<&AppState>,
        auth: AdminBearerAuth,
        id: Path<String>,
    ) -> Result<Json<AdminUserResponse>, ApiErrorResponse> {
        require_permission(&auth.0, "admin_users:read")?;
        let user = AdminUserService::find_by_id(&state.db, &id.0).await?;
        Ok(Json(user.into()))
    }

    /// Create admin user
    #[oai(path = "/", method = "post", operation_id = "adminCreateUser")]
    async fn create(
        &self,
        state: Data<&AppState>,
        auth: AdminBearerAuth,
        body: Json<CreateAdminUserRequest>,
    ) -> Result<Json<AdminUserResponse>, ApiErrorResponse> {
        require_permission(&auth.0, "admin_users:write")?;
        let user = AdminUserService::create(&state.db, &body.email, &body.password, &body.display_name).await?;
        audit(&state, &auth, "admin_user.create", "admin_user", &user.id);
        Ok(Json(user.into()))
    }

    /// Update admin user
    #[oai(path = "/:id", method = "patch", operation_id = "adminUpdateUser")]
    async fn update(
        &self,
        state: Data<&AppState>,
        auth: AdminBearerAuth,
        id: Path<String>,
        body: Json<UpdateAdminUserRequest>,
    ) -> Result<Json<AdminUserResponse>, ApiErrorResponse> {
        require_permission(&auth.0, "admin_users:write")?;
        let user = AdminUserService::update(&state.db, &id.0, body.email.clone(), body.display_name.clone(), body.is_active).await?;
        audit(&state, &auth, "admin_user.update", "admin_user", &id.0);
        Ok(Json(user.into()))
    }

    /// Delete admin user
    #[oai(path = "/:id", method = "delete", operation_id = "adminDeleteUser")]
    async fn delete(
        &self,
        state: Data<&AppState>,
        auth: AdminBearerAuth,
        id: Path<String>,
    ) -> Result<Json<serde_json::Value>, ApiErrorResponse> {
        require_permission(&auth.0, "admin_users:write")?;
        AdminUserService::delete(&state.db, &id.0).await?;
        audit(&state, &auth, "admin_user.delete", "admin_user", &id.0);
        Ok(Json(serde_json::json!({ "deleted": true })))
    }

    /// Change password
    #[oai(path = "/:id/password", method = "put", operation_id = "adminChangePassword")]
    async fn change_password(
        &self,
        state: Data<&AppState>,
        auth: AdminBearerAuth,
        id: Path<String>,
        body: Json<ChangePasswordRequest>,
    ) -> Result<Json<serde_json::Value>, ApiErrorResponse> {
        // Allow self or admin_users:write
        if auth.0.sub != id.0 {
            require_permission(&auth.0, "admin_users:write")?;
        }
        AdminUserService::change_password(&state.db, &id.0, &body.new_password).await?;
        audit(&state, &auth, "admin_user.change_password", "admin_user", &id.0);
        Ok(Json(serde_json::json!({ "changed": true })))
    }

    /// Assign role to user
    #[oai(path = "/:id/roles", method = "post", operation_id = "adminAssignRole")]
    async fn assign_role(
        &self,
        state: Data<&AppState>,
        auth: AdminBearerAuth,
        id: Path<String>,
        body: Json<AssignRoleRequest>,
    ) -> Result<Json<serde_json::Value>, ApiErrorResponse> {
        require_permission(&auth.0, "admin_users:write")?;
        AdminUserService::assign_role(&state.db, &id.0, &body.role_id).await?;
        audit(&state, &auth, "admin_user.assign_role", "admin_user", &id.0);
        Ok(Json(serde_json::json!({ "assigned": true })))
    }

    /// Remove role from user
    #[oai(path = "/:id/roles/:role_id", method = "delete", operation_id = "adminRemoveRole")]
    async fn remove_role(
        &self,
        state: Data<&AppState>,
        auth: AdminBearerAuth,
        id: Path<String>,
        role_id: Path<String>,
    ) -> Result<Json<serde_json::Value>, ApiErrorResponse> {
        require_permission(&auth.0, "admin_users:write")?;
        AdminUserService::remove_role(&state.db, &id.0, &role_id.0).await?;
        audit(&state, &auth, "admin_user.remove_role", "admin_user", &id.0);
        Ok(Json(serde_json::json!({ "removed": true })))
    }

    /// Get roles for a user
    #[oai(path = "/:id/roles", method = "get", operation_id = "adminGetUserRoles")]
    async fn get_roles(
        &self,
        state: Data<&AppState>,
        auth: AdminBearerAuth,
        id: Path<String>,
    ) -> Result<Json<Vec<RoleResponse>>, ApiErrorResponse> {
        require_permission(&auth.0, "admin_users:read")?;
        let roles = AdminUserService::get_roles(&state.db, &id.0).await?;
        Ok(Json(roles.into_iter().map(Into::into).collect()))
    }
}
