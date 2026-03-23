use poem_openapi::Tags;

pub mod admin;
pub mod admin_audit_logs;
pub mod admin_auth;
pub mod admin_roles;
pub mod admin_users;
pub mod app;
pub mod auth_guard;
pub mod authentication;
pub mod helpers;
pub mod internal;
pub mod session;
pub mod signature;

#[derive(Tags)]
pub enum ApiTags {
    /// RP API: Authentication session endpoints
    Authentication,
    /// RP API: Signature session endpoints
    Signature,
    /// RP API: Session status endpoint
    Session,
    /// Internal endpoints (mobile device simulation)
    Internal,
    /// Mobile app endpoints
    App,
    /// Admin: Platform management endpoints
    Admin,
}

pub use admin::AdminApi;
pub use admin_audit_logs::AdminAuditLogsApi;
pub use admin_auth::AdminAuthApi;
pub use admin_roles::AdminRolesApi;
pub use admin_users::AdminUsersApi;
pub use app::AppApi;
pub use authentication::AuthenticationApi;
pub use internal::InternalApi;
pub use session::SessionApi;
pub use signature::SignatureApi;
