use poem::web::Data;
use poem_openapi::{Object, OpenApi, payload::Json};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::models::*;
use crate::services::audit_log::{AuditEntry, AuditLogService};
use crate::services::auth::AuthService;

use super::auth_guard::AdminBearerAuth;

pub struct AdminAuthApi;

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct LoginResponse {
    pub token: String,
    #[oai(rename = "userId")]
    pub user_id: String,
    pub email: String,
    #[oai(rename = "displayName")]
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct MeResponse {
    pub id: String,
    pub email: String,
    #[oai(rename = "displayName")]
    pub display_name: String,
    pub permissions: Vec<String>,
}

#[OpenApi(prefix_path = "/admin/auth", tag = "super::ApiTags::Admin")]
impl AdminAuthApi {
    /// Login with email and password
    #[oai(path = "/login", method = "post", operation_id = "adminLogin")]
    async fn login(
        &self,
        state: Data<&AppState>,
        body: Json<LoginRequest>,
    ) -> Result<Json<LoginResponse>, ApiErrorResponse> {
        match AuthService::login(&state.db, &body.email, &body.password).await {
            Ok((token, user)) => {
                let db = state.db.clone();
                let user_id = user.id.clone();
                let email = user.email.clone();
                tokio::spawn(async move {
                    AuditLogService::log(&db, AuditEntry {
                        admin_user_id: Some(user_id),
                        admin_user_email: Some(email),
                        action: "login.success".to_string(),
                        resource_type: None,
                        resource_id: None,
                        details: None,
                        ip_address: None,
                    }).await;
                });
                Ok(Json(LoginResponse {
                    token,
                    user_id: user.id,
                    email: user.email,
                    display_name: user.display_name,
                }))
            }
            Err(e) => {
                let db = state.db.clone();
                let email = body.email.clone();
                tokio::spawn(async move {
                    AuditLogService::log(&db, AuditEntry {
                        admin_user_id: None,
                        admin_user_email: Some(email),
                        action: "login.failed".to_string(),
                        resource_type: None,
                        resource_id: None,
                        details: None,
                        ip_address: None,
                    }).await;
                });
                Err(e.into())
            }
        }
    }

    /// Get current authenticated user info
    #[oai(path = "/me", method = "get", operation_id = "adminMe")]
    async fn me(
        &self,
        auth: AdminBearerAuth,
    ) -> Result<Json<MeResponse>, ApiErrorResponse> {
        Ok(Json(MeResponse {
            id: auth.0.sub,
            email: auth.0.email,
            display_name: String::new(), // Claims don't carry display_name, just return email
            permissions: auth.0.permissions,
        }))
    }
}
