use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use poem_openapi::{Enum, Object};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use super::admin_user::AdminUserService;
use super::relying_party::ServiceError;

/// All permissions in the system. Stored as JSON array of string values in the roles table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Enum, Hash)]
pub enum Permission {
    #[oai(rename = "admin_users:read")]
    #[serde(rename = "admin_users:read")]
    AdminUsersRead,
    #[oai(rename = "admin_users:write")]
    #[serde(rename = "admin_users:write")]
    AdminUsersWrite,
    #[oai(rename = "roles:read")]
    #[serde(rename = "roles:read")]
    RolesRead,
    #[oai(rename = "roles:write")]
    #[serde(rename = "roles:write")]
    RolesWrite,
    #[oai(rename = "audit_logs:read")]
    #[serde(rename = "audit_logs:read")]
    AuditLogsRead,
    #[oai(rename = "relying_parties:read")]
    #[serde(rename = "relying_parties:read")]
    RelyingPartiesRead,
    #[oai(rename = "relying_parties:write")]
    #[serde(rename = "relying_parties:write")]
    RelyingPartiesWrite,
    #[oai(rename = "accounts:read")]
    #[serde(rename = "accounts:read")]
    AccountsRead,
    #[oai(rename = "accounts:write")]
    #[serde(rename = "accounts:write")]
    AccountsWrite,
    #[oai(rename = "sessions:read")]
    #[serde(rename = "sessions:read")]
    SessionsRead,
    #[oai(rename = "sessions:write")]
    #[serde(rename = "sessions:write")]
    SessionsWrite,
    #[oai(rename = "devices:read")]
    #[serde(rename = "devices:read")]
    DevicesRead,
    #[oai(rename = "devices:write")]
    #[serde(rename = "devices:write")]
    DevicesWrite,
    #[oai(rename = "certificates:read")]
    #[serde(rename = "certificates:read")]
    CertificatesRead,
    #[oai(rename = "certificates:write")]
    #[serde(rename = "certificates:write")]
    CertificatesWrite,
}

impl Permission {
    /// Return all defined permissions.
    pub fn all() -> &'static [Permission] {
        use Permission::*;
        &[
            AdminUsersRead, AdminUsersWrite,
            RolesRead, RolesWrite,
            AuditLogsRead,
            RelyingPartiesRead, RelyingPartiesWrite,
            AccountsRead, AccountsWrite,
            SessionsRead, SessionsWrite,
            DevicesRead, DevicesWrite,
            CertificatesRead, CertificatesWrite,
        ]
    }
}

/// JWT claims for admin authentication.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct AdminClaims {
    pub sub: String,
    pub email: String,
    pub permissions: Vec<String>,
    pub exp: usize,
    pub iat: usize,
}

fn jwt_secret() -> &'static [u8] {
    static SECRET: OnceLock<Vec<u8>> = OnceLock::new();
    SECRET.get_or_init(|| {
        match std::env::var("ADMIN_JWT_SECRET") {
            Ok(s) => s.into_bytes(),
            Err(_) => {
                tracing::warn!("ADMIN_JWT_SECRET not set, using insecure default for development");
                b"smartid-dev-secret-change-me".to_vec()
            }
        }
    })
}

pub struct AuthService;

impl AuthService {
    pub fn generate_token(
        user_id: &str,
        email: &str,
        permissions: Vec<String>,
    ) -> Result<String, ServiceError> {
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = AdminClaims {
            sub: user_id.to_string(),
            email: email.to_string(),
            permissions,
            iat: now,
            exp: now + 8 * 3600, // 8 hours
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret()),
        )
        .map_err(|e| ServiceError::Forbidden(format!("token generation failed: {e}")))
    }

    pub fn validate_token(token: &str) -> Result<AdminClaims, ServiceError> {
        decode::<AdminClaims>(
            token,
            &DecodingKey::from_secret(jwt_secret()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| ServiceError::Forbidden(format!("invalid token: {e}")))
    }

    pub async fn login(
        db: &DatabaseConnection,
        email: &str,
        password: &str,
    ) -> Result<(String, crate::db::entities::admin_user::Model), ServiceError> {
        let user = AdminUserService::verify_password(db, email, password).await?;
        let permissions = AdminUserService::get_permissions(db, &user.id).await?;
        let token = Self::generate_token(&user.id, &user.email, permissions)?;
        Ok((token, user))
    }
}
