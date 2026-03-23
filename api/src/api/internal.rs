use poem::web::Data;
use poem_openapi::{Object, OpenApi, param::Path, payload::Json};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::models::*;
use crate::services::account::AccountService;
use crate::services::session::{SessionCompletion, SessionService};

pub struct InternalApi;

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct CompleteSessionRequest {
    #[oai(rename = "endResult")]
    pub end_result: String,
    #[oai(rename = "flowType")]
    pub flow_type: String,
    #[oai(rename = "interactionTypeUsed", skip_serializing_if_is_none)]
    pub interaction_type_used: Option<String>,
    #[oai(rename = "deviceIpAddress", skip_serializing_if_is_none)]
    pub device_ip_address: Option<String>,
    #[oai(rename = "signatureValue", skip_serializing_if_is_none)]
    pub signature_value: Option<String>,
    #[oai(rename = "userChallenge", skip_serializing_if_is_none)]
    pub user_challenge: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct CompleteSessionResponse {
    #[oai(rename = "sessionID")]
    pub session_id: String,
    pub state: String,
    #[oai(rename = "endResult")]
    pub end_result: String,
    #[oai(rename = "documentNumber", skip_serializing_if_is_none)]
    pub document_number: Option<String>,
}

#[OpenApi(prefix_path = "/internal", tag = "super::ApiTags::Internal")]
impl InternalApi {
    /// Complete a session (simulates mobile device confirming/refusing)
    #[oai(
        path = "/sessions/:session_id/complete",
        method = "post",
        operation_id = "completeSession"
    )]
    async fn complete_session(
        &self,
        state: Data<&AppState>,
        session_id: Path<String>,
        body: Json<CompleteSessionRequest>,
    ) -> Result<Json<CompleteSessionResponse>, ApiErrorResponse> {
        let session = SessionService::find(&state.db, &session_id.0)
            .await?
            .ok_or_else(|| {
                ApiErrorResponse::NotFound(Json(ProblemDetails::new(
                    404,
                    "Not Found",
                    &format!("session '{}' not found", session_id.0),
                )))
            })?;

        let end_result = SessionEndResult::from_str(&body.end_result);
        let flow_type = FlowType::from_str(&body.flow_type);
        let interaction_type = body
            .interaction_type_used
            .as_deref()
            .and_then(InteractionType::from_str);

        let (document_number, cert_value, cert_level) = if end_result == SessionEndResult::Ok {
            let account = match &session.account_id {
                Some(id) => {
                    use sea_orm::EntityTrait;
                    crate::db::entities::account::Entity::find_by_id(id)
                        .one(&state.db)
                        .await
                        .map_err(|e| {
                            ApiErrorResponse::InternalServerError(Json(ProblemDetails::new(
                                500,
                                "Internal Server Error",
                                &e.to_string(),
                            )))
                        })?
                }
                None => Some(AccountService::create_anonymous(&state.db).await?),
            };

            if let Some(acct) = account {
                match session.kind.as_str() {
                    "authentication" => {
                        let cert = state
                            .certificate
                            .get_or_issue_auth_cert(&state.db, &acct.id, &acct.document_number)
                            .await?;
                        (
                            Some(acct.document_number),
                            Some(cert.cert_value),
                            Some(cert.cert_level),
                        )
                    }
                    "signing" => {
                        let cert = state
                            .certificate
                            .get_or_issue_signing_cert(&state.db, &acct.id, &acct.document_number)
                            .await?;
                        (
                            Some(acct.document_number),
                            Some(cert.cert_value),
                            Some(cert.cert_level),
                        )
                    }
                    _ => (Some(acct.document_number), None, None),
                }
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

        let completed = SessionService::complete(
            &state.db,
            &state.notifier,
            &session_id.0,
            SessionCompletion {
                end_result,
                document_number: document_number.clone(),
                flow_type,
                interaction_type_used: interaction_type,
                device_ip_address: body.device_ip_address.clone(),
                signature_value: body.signature_value.clone(),
                user_challenge: body.user_challenge.clone(),
                cert_value,
                cert_level,
            },
        )
        .await?;

        Ok(Json(CompleteSessionResponse {
            session_id: completed.id,
            state: completed.state,
            end_result: completed.end_result.unwrap_or_default(),
            document_number,
        }))
    }
}
