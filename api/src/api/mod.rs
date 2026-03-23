use poem_openapi::Tags;

pub mod admin;
pub mod app;
pub mod authentication;
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
pub use app::AppApi;
pub use authentication::AuthenticationApi;
pub use internal::InternalApi;
pub use session::SessionApi;
pub use signature::SignatureApi;
