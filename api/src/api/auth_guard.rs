use poem::Request;
use poem_openapi::{SecurityScheme, auth::Bearer, payload::Json};

use crate::models::*;
use crate::services::auth::{AdminClaims, AuthService};

/// Bearer token authentication for admin endpoints.
#[derive(SecurityScheme)]
#[oai(ty = "bearer", key_name = "Authorization", checker = "check_bearer")]
pub struct AdminBearerAuth(pub AdminClaims);

async fn check_bearer(_req: &Request, bearer: Bearer) -> Option<AdminClaims> {
    AuthService::validate_token(&bearer.token).ok()
}

/// Check that the authenticated user has a specific permission.
pub fn require_permission(claims: &AdminClaims, permission: &str) -> Result<(), ApiErrorResponse> {
    if claims.permissions.iter().any(|p| p == permission) {
        Ok(())
    } else {
        Err(ApiErrorResponse::Forbidden(Json(ProblemDetails::new(
            403,
            "Forbidden",
            "Insufficient permissions",
        ))))
    }
}
