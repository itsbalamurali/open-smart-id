use poem::web::Data;
use poem_openapi::{Enum, Object, OpenApi, param::Path, param::Query, payload::Json};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::models::*;
use crate::services::account::AccountService;
use crate::services::device::DeviceService;
use crate::services::relying_party::RelyingPartyService;
use crate::services::session::{
    FlowType, InteractionType, SessionCompletion, SessionEndResult, SessionService,
};

use super::helpers::resolve_cert_for_completion;

// ── Request/Response types ──

/// Mobile device platform.
#[derive(Debug, Clone, Serialize, Deserialize, Enum, PartialEq, Eq)]
pub enum DevicePlatform {
    /// Apple iOS
    #[oai(rename = "ios")]
    Ios,
    /// Google Android
    #[oai(rename = "android")]
    Android,
}

impl DevicePlatform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ios => "ios",
            Self::Android => "android",
        }
    }
}

/// Register a mobile device for push notifications.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct RegisterDeviceRequest {
    /// ETSI semantic identifier (e.g. "PNOEE-48010010101")
    #[oai(rename = "semanticId")]
    pub semantic_id: String,
    /// Firebase Cloud Messaging registration token
    #[oai(rename = "fcmToken")]
    pub fcm_token: String,
    /// Human-readable device name
    #[oai(rename = "deviceName", skip_serializing_if_is_none)]
    pub device_name: Option<String>,
    /// Device platform
    pub platform: DevicePlatform,
}

/// Successful device registration response.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct RegisterDeviceResponse {
    /// Unique device registration ID
    #[oai(rename = "deviceId")]
    pub device_id: String,
    /// Account ID this device is linked to
    #[oai(rename = "accountId")]
    pub account_id: String,
    /// Document number assigned to the account
    #[oai(rename = "documentNumber")]
    pub document_number: String,
}

/// Update a device's FCM token or name.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct UpdateDeviceRequest {
    /// Updated FCM registration token
    #[oai(rename = "fcmToken", skip_serializing_if_is_none)]
    pub fcm_token: Option<String>,
    /// Updated device name
    #[oai(rename = "deviceName", skip_serializing_if_is_none)]
    pub device_name: Option<String>,
}

/// Device update confirmation.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct UpdateDeviceResponse {
    /// Device ID
    #[oai(rename = "deviceId")]
    pub device_id: String,
    /// Current status: "active" or "inactive"
    pub status: String,
}

/// Device deactivation confirmation.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct DeactivateDeviceResponse {
    /// Deactivation status
    pub status: String,
}

/// Summary of a pending session.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct PendingSessionSummary {
    /// Session ID
    #[oai(rename = "sessionId")]
    pub session_id: String,
    /// Session type: "authentication", "signing", or "certificate_choice"
    pub kind: String,
    /// Relying party name that initiated the session
    #[oai(rename = "relyingPartyName", skip_serializing_if_is_none)]
    pub relying_party_name: Option<String>,
    /// ISO 8601 creation timestamp
    #[oai(rename = "createdAt")]
    pub created_at: String,
}

/// List of pending sessions for the device's account.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct PendingSessionsResponse {
    /// Sessions in RUNNING state
    pub sessions: Vec<PendingSessionSummary>,
}

/// Full session details for the mobile app.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct AppSessionDetailResponse {
    /// Session ID
    #[oai(rename = "sessionId")]
    pub session_id: String,
    /// Session type
    pub kind: String,
    /// Current state: "RUNNING" or "COMPLETE"
    pub state: String,
    /// Relying party name
    #[oai(rename = "relyingPartyName", skip_serializing_if_is_none)]
    pub relying_party_name: Option<String>,
    /// Base64-encoded interactions JSON
    #[oai(skip_serializing_if_is_none)]
    pub interactions: Option<String>,
    /// Verification code for notification signing
    #[oai(skip_serializing_if_is_none)]
    pub vc: Option<VC>,
    /// Requested hash algorithm
    #[oai(rename = "hashAlgorithm", skip_serializing_if_is_none)]
    pub hash_algorithm: Option<String>,
    /// ISO 8601 creation timestamp
    #[oai(rename = "createdAt")]
    pub created_at: String,
}

/// Confirm a session from the mobile app.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct ConfirmSessionRequest {
    /// Base64-encoded digital signature produced by the device
    #[oai(rename = "signatureValue")]
    pub signature_value: String,
    /// Base64URL-encoded user challenge (ACSP_V2 authentication)
    #[oai(rename = "userChallenge", skip_serializing_if_is_none)]
    pub user_challenge: Option<String>,
    /// Interaction type used in the app
    #[oai(rename = "interactionTypeUsed", skip_serializing_if_is_none)]
    pub interaction_type_used: Option<String>,
    /// Device IP address
    #[oai(rename = "deviceIpAddress", skip_serializing_if_is_none)]
    pub device_ip_address: Option<String>,
}

/// Result of confirming or refusing a session.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct AppSessionActionResponse {
    /// Session ID
    #[oai(rename = "sessionId")]
    pub session_id: String,
    /// New session state
    pub state: String,
    /// End result
    #[oai(rename = "endResult")]
    pub end_result: String,
    /// Document number (present on OK)
    #[oai(rename = "documentNumber", skip_serializing_if_is_none)]
    pub document_number: Option<String>,
}

// ── API ──

pub struct AppApi;

#[OpenApi(prefix_path = "/app/v1", tag = "super::ApiTags::App")]
impl AppApi {
    /// Register a mobile device for push notifications.
    ///
    /// Links the device to an account identified by ETSI semantic ID and stores
    /// the FCM token for push notification delivery.
    #[oai(
        path = "/devices/register",
        method = "post",
        operation_id = "appRegisterDevice"
    )]
    async fn register_device(
        &self,
        state: Data<&AppState>,
        body: Json<RegisterDeviceRequest>,
    ) -> Result<Json<RegisterDeviceResponse>, ApiErrorResponse> {
        let account =
            AccountService::find_or_create_by_semantic_id(&state.db, &body.semantic_id).await?;
        let device = DeviceService::register(
            &state.db,
            &account.id,
            &body.fcm_token,
            body.device_name.as_deref(),
            body.platform.as_str(),
        )
        .await?;
        Ok(Json(RegisterDeviceResponse {
            device_id: device.id,
            account_id: account.id,
            document_number: account.document_number,
        }))
    }

    /// Update a device's FCM token or name.
    ///
    /// Called when the FCM token is refreshed by the platform or the user
    /// renames their device.
    #[oai(
        path = "/devices/:device_id",
        method = "put",
        operation_id = "appUpdateDevice"
    )]
    async fn update_device(
        &self,
        state: Data<&AppState>,
        device_id: Path<String>,
        body: Json<UpdateDeviceRequest>,
    ) -> Result<Json<UpdateDeviceResponse>, ApiErrorResponse> {
        let device = DeviceService::update(
            &state.db,
            &device_id.0,
            body.fcm_token.as_deref(),
            body.device_name.as_deref(),
        )
        .await?;
        Ok(Json(UpdateDeviceResponse {
            device_id: device.id,
            status: if device.is_active {
                "active"
            } else {
                "inactive"
            }
            .to_string(),
        }))
    }

    /// Deactivate a device (soft delete).
    ///
    /// The device will no longer receive push notifications.
    #[oai(
        path = "/devices/:device_id",
        method = "delete",
        operation_id = "appDeactivateDevice"
    )]
    async fn deactivate_device(
        &self,
        state: Data<&AppState>,
        device_id: Path<String>,
    ) -> Result<Json<DeactivateDeviceResponse>, ApiErrorResponse> {
        DeviceService::deactivate(&state.db, &device_id.0).await?;
        Ok(Json(DeactivateDeviceResponse {
            status: "deactivated".to_string(),
        }))
    }

    /// List pending sessions for the device's account.
    ///
    /// Returns all sessions in RUNNING state for the account linked to the
    /// given device. The app calls this after receiving a push notification.
    #[oai(
        path = "/sessions/pending",
        method = "get",
        operation_id = "appListPendingSessions"
    )]
    async fn list_pending_sessions(
        &self,
        state: Data<&AppState>,
        #[oai(name = "deviceId")] device_id: Query<String>,
    ) -> Result<Json<PendingSessionsResponse>, ApiErrorResponse> {
        let dev = DeviceService::find_by_id(&state.db, &device_id.0).await?;

        let sessions =
            SessionService::find_running_by_account(&state.db, &dev.account_id).await?;

        let mut summaries = Vec::with_capacity(sessions.len());
        for s in sessions {
            let rp_name = RelyingPartyService::find_by_id(&state.db, &s.relying_party_id)
                .await
                .ok()
                .map(|rp| rp.name);
            summaries.push(PendingSessionSummary {
                session_id: s.id,
                kind: s.kind,
                relying_party_name: rp_name,
                created_at: s.created_at.to_rfc3339(),
            });
        }

        Ok(Json(PendingSessionsResponse {
            sessions: summaries,
        }))
    }

    /// Get full details of a session.
    ///
    /// Returns all data the mobile app needs to display the authentication
    /// or signing confirmation screen.
    #[oai(
        path = "/sessions/:session_id",
        method = "get",
        operation_id = "appGetSessionDetail"
    )]
    async fn get_session_detail(
        &self,
        state: Data<&AppState>,
        session_id: Path<String>,
    ) -> Result<Json<AppSessionDetailResponse>, ApiErrorResponse> {
        let sess = SessionService::find_by_id(&state.db, &session_id.0).await?;

        let rp_name = RelyingPartyService::find_by_id(&state.db, &sess.relying_party_id)
            .await
            .ok()
            .map(|rp| rp.name);

        let vc = sess.vc_value.as_ref().map(|val| VC {
            vc_type: VerificationCodeType::Numeric4,
            value: val.clone(),
        });

        Ok(Json(AppSessionDetailResponse {
            session_id: sess.id,
            kind: sess.kind,
            state: sess.state,
            relying_party_name: rp_name,
            interactions: sess.interactions,
            vc,
            hash_algorithm: sess.hash_algorithm,
            created_at: sess.created_at.to_rfc3339(),
        }))
    }

    /// Confirm a session (user approves).
    ///
    /// The app calls this after the user reviews and approves the request.
    /// Issues a certificate, completes the session with OK, and notifies
    /// long-polling RPs.
    #[oai(
        path = "/sessions/:session_id/confirm",
        method = "post",
        operation_id = "appConfirmSession"
    )]
    async fn confirm_session(
        &self,
        state: Data<&AppState>,
        session_id: Path<String>,
        body: Json<ConfirmSessionRequest>,
    ) -> Result<Json<AppSessionActionResponse>, ApiErrorResponse> {
        let sess = SessionService::find_by_id(&state.db, &session_id.0).await?;

        let (document_number, cert_value, cert_level) =
            resolve_cert_for_completion(&state, &sess).await?;

        let interaction_type = body
            .interaction_type_used
            .as_deref()
            .and_then(InteractionType::from_str);

        let completed = SessionService::complete(
            &state.db,
            &state.notifier,
            &session_id.0,
            SessionCompletion {
                end_result: SessionEndResult::Ok,
                document_number: document_number.clone(),
                flow_type: Some(FlowType::Notification),
                interaction_type_used: interaction_type,
                device_ip_address: body.device_ip_address.clone(),
                signature_value: Some(body.signature_value.clone()),
                user_challenge: body.user_challenge.clone(),
                cert_value,
                cert_level,
            },
        )
        .await?;

        Ok(Json(AppSessionActionResponse {
            session_id: completed.id,
            state: completed.state,
            end_result: completed.end_result.unwrap_or_default(),
            document_number,
        }))
    }

    /// Refuse a session (user declines).
    ///
    /// Completes the session with USER_REFUSED. The RP will see this
    /// when polling the session status.
    #[oai(
        path = "/sessions/:session_id/refuse",
        method = "post",
        operation_id = "appRefuseSession"
    )]
    async fn refuse_session(
        &self,
        state: Data<&AppState>,
        session_id: Path<String>,
    ) -> Result<Json<AppSessionActionResponse>, ApiErrorResponse> {
        let completed = SessionService::complete(
            &state.db,
            &state.notifier,
            &session_id.0,
            SessionCompletion {
                end_result: SessionEndResult::UserRefused,
                document_number: None,
                flow_type: Some(FlowType::Notification),
                interaction_type_used: None,
                device_ip_address: None,
                signature_value: None,
                user_challenge: None,
                cert_value: None,
                cert_level: None,
            },
        )
        .await?;

        Ok(Json(AppSessionActionResponse {
            session_id: completed.id,
            state: completed.state,
            end_result: completed.end_result.unwrap_or_default(),
            document_number: None,
        }))
    }
}
