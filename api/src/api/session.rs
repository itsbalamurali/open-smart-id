use poem::web::Data;
use poem_openapi::{
    OpenApi,
    param::{Path, Query},
    payload::Json,
};

use crate::AppState;
use crate::models::*;
use crate::services::session::SessionService;

pub struct SessionApi;

#[OpenApi(prefix_path = "/v3/session", tag = "super::ApiTags::Session")]
impl SessionApi {
    /// Get session status (long poll)
    #[oai(path = "/:session_id", method = "get", operation_id = "sessionStatus")]
    async fn session_status(
        &self,
        state: Data<&AppState>,
        session_id: Path<String>,
        #[oai(name = "timeoutMs")] timeout_ms: Query<Option<i64>>,
    ) -> Result<Json<SessionStatusResponse>, ApiErrorResponse> {
        let response =
            SessionService::poll(&state.db, &state.notifier, &session_id.0, timeout_ms.0).await?;
        Ok(Json(response))
    }
}
