use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chrono::Utc;
use poem_openapi::{Enum, Object, Union};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

use crate::db::entities::session;
use crate::models::{
    CertificateInfo, HashAlgorithm, MaskGenAlgorithm, MaskGenAlgorithmId,
    MaskGenAlgorithmParameters, SignatureAlgorithm, SignatureAlgorithmParametersInResponse, VC,
};
use crate::services::relying_party::ServiceError;

/// Current state of a session.
#[derive(Debug, Clone, Serialize, Deserialize, Enum, PartialEq, Eq)]
pub enum SessionState {
    /// Session is in progress and awaiting user action.
    #[oai(rename = "RUNNING")]
    Running,
    /// Session has finished (successfully or not).
    #[oai(rename = "COMPLETE")]
    Complete,
}

/// Final outcome of a completed session.
#[derive(Debug, Clone, Serialize, Deserialize, Enum, PartialEq, Eq)]
pub enum SessionEndResult {
    /// Session completed successfully.
    #[oai(rename = "OK")]
    Ok,
    /// User refused the request.
    #[oai(rename = "USER_REFUSED")]
    UserRefused,
    /// Session timed out.
    #[oai(rename = "TIMEOUT")]
    Timeout,
    /// User's certificate is unusable.
    #[oai(rename = "DOCUMENT_UNUSABLE")]
    DocumentUnusable,
    /// User entered the wrong verification code.
    #[oai(rename = "WRONG_VC")]
    WrongVc,
    /// App does not support the required interaction.
    #[oai(rename = "REQUIRED_INTERACTION_NOT_SUPPORTED_BY_APP")]
    RequiredInteractionNotSupportedByApp,
    /// User refused certificate choice.
    #[oai(rename = "USER_REFUSED_CERT_CHOICE")]
    UserRefusedCertChoice,
    /// User refused during interaction.
    #[oai(rename = "USER_REFUSED_INTERACTION")]
    UserRefusedInteraction,
    /// Protocol-level failure.
    #[oai(rename = "PROTOCOL_FAILURE")]
    ProtocolFailure,
    /// A linked session was expected but not provided.
    #[oai(rename = "EXPECTED_LINKED_SESSION")]
    ExpectedLinkedSession,
    /// Internal server error.
    #[oai(rename = "SERVER_ERROR")]
    ServerError,
}

impl SessionEndResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::UserRefused => "USER_REFUSED",
            Self::Timeout => "TIMEOUT",
            Self::DocumentUnusable => "DOCUMENT_UNUSABLE",
            Self::WrongVc => "WRONG_VC",
            Self::RequiredInteractionNotSupportedByApp => {
                "REQUIRED_INTERACTION_NOT_SUPPORTED_BY_APP"
            }
            Self::UserRefusedCertChoice => "USER_REFUSED_CERT_CHOICE",
            Self::UserRefusedInteraction => "USER_REFUSED_INTERACTION",
            Self::ProtocolFailure => "PROTOCOL_FAILURE",
            Self::ExpectedLinkedSession => "EXPECTED_LINKED_SESSION",
            Self::ServerError => "SERVER_ERROR",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "OK" => Self::Ok,
            "USER_REFUSED" => Self::UserRefused,
            "TIMEOUT" => Self::Timeout,
            "DOCUMENT_UNUSABLE" => Self::DocumentUnusable,
            "WRONG_VC" => Self::WrongVc,
            "REQUIRED_INTERACTION_NOT_SUPPORTED_BY_APP" => {
                Self::RequiredInteractionNotSupportedByApp
            }
            "USER_REFUSED_CERT_CHOICE" => Self::UserRefusedCertChoice,
            "USER_REFUSED_INTERACTION" => Self::UserRefusedInteraction,
            "PROTOCOL_FAILURE" => Self::ProtocolFailure,
            "EXPECTED_LINKED_SESSION" => Self::ExpectedLinkedSession,
            _ => Self::ServerError,
        }
    }
}

/// Signature protocol used in a session response.
#[derive(Debug, Clone, Serialize, Deserialize, Enum, PartialEq, Eq)]
pub enum SessionSignatureProtocol {
    /// ACSP version 2 authentication protocol.
    #[oai(rename = "ACSP_V2")]
    AcspV2,
    /// Raw digest signature protocol.
    #[oai(rename = "RAW_DIGEST_SIGNATURE")]
    RawDigestSignature,
}

impl SessionSignatureProtocol {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ACSP_V2" => Some(Self::AcspV2),
            "RAW_DIGEST_SIGNATURE" => Some(Self::RawDigestSignature),
            _ => None,
        }
    }
}

/// How the user's device was linked to the session.
#[derive(Debug, Clone, Serialize, Deserialize, Enum, PartialEq, Eq)]
pub enum FlowType {
    /// QR code scanning flow.
    QR,
    /// App-to-app linking flow.
    App2App,
    /// Web-to-app linking flow.
    Web2App,
    /// Push notification flow.
    Notification,
}

impl FlowType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::QR => "QR",
            Self::App2App => "App2App",
            Self::Web2App => "Web2App",
            Self::Notification => "Notification",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "QR" => Some(Self::QR),
            "App2App" => Some(Self::App2App),
            "Web2App" => Some(Self::Web2App),
            "Notification" => Some(Self::Notification),
            _ => None,
        }
    }
}

/// Interaction type used when presenting the request to the user.
#[derive(Debug, Clone, Serialize, Deserialize, Enum, PartialEq, Eq)]
pub enum InteractionType {
    /// Display text and require PIN entry.
    #[oai(rename = "displayTextAndPIN")]
    DisplayTextAndPin,
    /// Display a confirmation message.
    #[oai(rename = "confirmationMessage")]
    ConfirmationMessage,
    /// Confirmation message with verification code choice.
    #[oai(rename = "confirmationMessageAndVerificationCodeChoice")]
    ConfirmationMessageAndVerificationCodeChoice,
}

impl InteractionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DisplayTextAndPin => "displayTextAndPIN",
            Self::ConfirmationMessage => "confirmationMessage",
            Self::ConfirmationMessageAndVerificationCodeChoice => {
                "confirmationMessageAndVerificationCodeChoice"
            }
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "displayTextAndPIN" => Some(Self::DisplayTextAndPin),
            "confirmationMessage" => Some(Self::ConfirmationMessage),
            "confirmationMessageAndVerificationCodeChoice" => {
                Some(Self::ConfirmationMessageAndVerificationCodeChoice)
            }
            _ => None,
        }
    }
}

/// Additional details about the session result.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct SessionResultDetails {
    /// Interaction type that was used.
    #[oai(skip_serializing_if_is_none)]
    pub interaction: Option<String>,
}

/// Outcome of a completed session.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct SessionResult {
    /// Final result code.
    #[oai(rename = "endResult")]
    pub end_result: SessionEndResult,
    /// Document number of the user (present on success).
    #[oai(rename = "documentNumber", skip_serializing_if_is_none)]
    pub document_number: Option<String>,
    /// Additional result details.
    #[oai(skip_serializing_if_is_none)]
    pub details: Option<SessionResultDetails>,
}

/// ACSP v2 authentication signature returned in a session response.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct AcspV2Signature {
    /// Base64-encoded signature value.
    pub value: String,
    /// Server-generated random value used in signing.
    #[oai(rename = "serverRandom")]
    pub server_random: String,
    /// User challenge value used in signing.
    #[oai(rename = "userChallenge")]
    pub user_challenge: String,
    /// Flow type used to link the device.
    #[oai(rename = "flowType")]
    pub flow_type: FlowType,
    /// Signature algorithm used.
    #[oai(rename = "signatureAlgorithm")]
    pub signature_algorithm: SignatureAlgorithm,
    /// Signature algorithm parameters.
    #[oai(rename = "signatureAlgorithmParameters", skip_serializing_if_is_none)]
    pub signature_algorithm_parameters: Option<SignatureAlgorithmParametersInResponse>,
}

/// Raw digest signature returned in a signing session response.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct RawDigestSignature {
    /// Base64-encoded signature value.
    pub value: String,
    /// Flow type used to link the device.
    #[oai(rename = "flowType")]
    pub flow_type: FlowType,
    /// Signature algorithm used.
    #[oai(rename = "signatureAlgorithm")]
    pub signature_algorithm: SignatureAlgorithm,
    /// Signature algorithm parameters.
    #[oai(rename = "signatureAlgorithmParameters", skip_serializing_if_is_none)]
    pub signature_algorithm_parameters: Option<SignatureAlgorithmParametersInResponse>,
}

/// Signature placeholder for certificate choice sessions.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct CertificateChoiceSignature {
    /// Flow type used to link the device.
    #[oai(rename = "flowType")]
    pub flow_type: FlowType,
}

/// Signature data returned in a completed session, discriminated by protocol type.
#[derive(Debug, Clone, Serialize, Deserialize, Union)]
#[oai(discriminator_name = "type", one_of)]
pub enum SessionSignature {
    /// ACSP v2 authentication signature.
    AcspV2(AcspV2Signature),
    /// Raw digest signing signature.
    RawDigest(RawDigestSignature),
    /// Certificate choice result.
    CertificateChoice(CertificateChoiceSignature),
}

/// Response returned when polling session status.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct SessionStatusResponse {
    /// Current session state.
    pub state: SessionState,
    /// Session result (present when complete).
    #[oai(skip_serializing_if_is_none)]
    pub result: Option<SessionResult>,
    /// Signature protocol that was used.
    #[oai(rename = "signatureProtocol", skip_serializing_if_is_none)]
    pub signature_protocol: Option<SessionSignatureProtocol>,
    /// Signature data (present on successful completion).
    #[oai(skip_serializing_if_is_none)]
    pub signature: Option<SessionSignature>,
    /// Certificate used in the session.
    #[oai(skip_serializing_if_is_none)]
    pub cert: Option<CertificateInfo>,
    /// Interaction type that was used in the app.
    #[oai(rename = "interactionTypeUsed", skip_serializing_if_is_none)]
    pub interaction_type_used: Option<InteractionType>,
    /// IP address of the mobile device.
    #[oai(rename = "deviceIpAddress", skip_serializing_if_is_none)]
    pub device_ip_address: Option<String>,
    /// Properties from the request that were ignored.
    #[oai(rename = "ignoredProperties", skip_serializing_if_is_none)]
    pub ignored_properties: Option<Vec<String>>,
}

/// Response for device-link session initiation.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct DeviceLinkResponse {
    /// Unique session identifier.
    #[oai(rename = "sessionID")]
    pub session_id: String,
    /// Token used to construct the device link URL.
    #[oai(rename = "sessionToken")]
    pub session_token: String,
    /// Secret for authenticating device-link callbacks.
    #[oai(rename = "sessionSecret")]
    pub session_secret: String,
    /// Base URL for device link QR code generation.
    #[oai(rename = "deviceLinkBase")]
    pub device_link_base: String,
}

/// Response for notification-based authentication session initiation.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct NotificationAuthenticationResponse {
    /// Unique session identifier.
    #[oai(rename = "sessionID")]
    pub session_id: String,
}

/// Response for notification-based signing session initiation.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct NotificationSigningResponse {
    /// Unique session identifier.
    #[oai(rename = "sessionID")]
    pub session_id: String,
    /// Verification code to display to the user.
    pub vc: VC,
}

/// Response for notification-based linked signing session initiation.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct NotificationSigningLinkedResponse {
    /// Unique session identifier.
    #[oai(rename = "sessionID")]
    pub session_id: String,
}

/// Response for notification-based certificate choice session initiation.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct NotificationCertificateChoiceResponse {
    /// Unique session identifier.
    #[oai(rename = "sessionID")]
    pub session_id: String,
}

#[derive(Clone, Default)]
pub struct SessionNotifier {
    waiters: Arc<Mutex<HashMap<String, Arc<Notify>>>>,
}

impl SessionNotifier {
    pub fn new() -> Self {
        Self::default()
    }

    async fn get_or_create(&self, session_id: &str) -> Arc<Notify> {
        let mut map = self.waiters.lock().await;
        map.entry(session_id.to_string())
            .or_insert_with(|| Arc::new(Notify::new()))
            .clone()
    }

    async fn notify(&self, session_id: &str) {
        let map = self.waiters.lock().await;
        if let Some(n) = map.get(session_id) {
            n.notify_waiters();
        }
    }

    async fn remove(&self, session_id: &str) {
        self.waiters.lock().await.remove(session_id);
    }
}

pub struct SessionService;

impl SessionService {
    pub async fn create(
        db: &DatabaseConnection,
        params: CreateSessionParams,
    ) -> Result<session::Model, ServiceError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let model = session::ActiveModel {
            id: Set(id.clone()),
            relying_party_id: Set(params.relying_party_id),
            account_id: Set(params.account_id),
            kind: Set(params.kind.as_str().to_string()),
            state: Set("RUNNING".to_string()),
            session_token: Set(params.session_token),
            session_secret: Set(params.session_secret),
            device_link_base: Set(params.device_link_base),
            signature_protocol: Set(params.signature_protocol.map(|p| p.to_string())),
            signature_algorithm: Set(params
                .signature_algorithm
                .as_ref()
                .map(|a| a.as_str().to_string())),
            hash_algorithm: Set(params
                .hash_algorithm
                .as_ref()
                .map(|h| h.as_str().to_string())),
            certificate_level: Set(params.certificate_level),
            challenge_or_digest: Set(params.challenge_or_digest),
            interactions: Set(params.interactions),
            nonce: Set(params.nonce),
            initial_callback_url: Set(params.initial_callback_url),
            linked_session_id: Set(params.linked_session_id),
            flow_type: Set(None),
            vc_type: Set(params.vc_type),
            vc_value: Set(params.vc_value),
            share_md_client_ip_address: Set(params.share_md_client_ip_address),
            end_result: Set(None),
            document_number: Set(None),
            signature_json: Set(None),
            server_random: Set(None),
            user_challenge: Set(None),
            cert_value: Set(None),
            cert_level: Set(None),
            interaction_type_used: Set(None),
            device_ip_address: Set(None),
            ignored_properties_json: Set(None),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };

        session::Entity::insert(model)
            .exec(db)
            .await
            .map_err(ServiceError::Db)?;

        session::Entity::find_by_id(&id)
            .one(db)
            .await
            .map_err(ServiceError::Db)?
            .ok_or(ServiceError::NotFound("session".to_string()))
    }

    pub async fn find(
        db: &DatabaseConnection,
        session_id: &str,
    ) -> Result<Option<session::Model>, ServiceError> {
        session::Entity::find_by_id(session_id)
            .one(db)
            .await
            .map_err(ServiceError::Db)
    }

    pub async fn poll(
        db: &DatabaseConnection,
        notifier: &SessionNotifier,
        session_id: &str,
        timeout_ms: Option<i64>,
    ) -> Result<SessionStatusResponse, ServiceError> {
        let row = session::Entity::find_by_id(session_id)
            .one(db)
            .await
            .map_err(ServiceError::Db)?
            .ok_or(ServiceError::NotFound(format!(
                "session '{session_id}' not found"
            )))?;

        if row.state == "COMPLETE" {
            notifier.remove(session_id).await;
            return Ok(model_to_response(&row));
        }

        let notify = notifier.get_or_create(session_id).await;
        let timeout = std::time::Duration::from_millis(
            timeout_ms.unwrap_or(60500).clamp(1000, 120000) as u64,
        );

        let _ = tokio::time::timeout(timeout, notify.notified()).await;

        let row = session::Entity::find_by_id(session_id)
            .one(db)
            .await
            .map_err(ServiceError::Db)?
            .ok_or(ServiceError::NotFound(format!(
                "session '{session_id}' not found"
            )))?;

        if row.state == "COMPLETE" {
            notifier.remove(session_id).await;
        }

        Ok(model_to_response(&row))
    }

    pub async fn complete(
        db: &DatabaseConnection,
        notifier: &SessionNotifier,
        session_id: &str,
        completion: SessionCompletion,
    ) -> Result<session::Model, ServiceError> {
        let row = session::Entity::find_by_id(session_id)
            .one(db)
            .await
            .map_err(ServiceError::Db)?
            .ok_or(ServiceError::NotFound(format!(
                "session '{session_id}' not found"
            )))?;

        if row.state == "COMPLETE" {
            return Err(ServiceError::Forbidden("session already completed".into()));
        }

        let server_random = BASE64.encode(uuid::Uuid::new_v4().as_bytes());

        let update = session::ActiveModel {
            id: Set(session_id.to_string()),
            state: Set("COMPLETE".to_string()),
            end_result: Set(Some(completion.end_result.as_str().to_string())),
            document_number: Set(completion.document_number),
            flow_type: Set(completion
                .flow_type
                .as_ref()
                .map(|f| f.as_str().to_string())),
            interaction_type_used: Set(completion
                .interaction_type_used
                .as_ref()
                .map(|i| i.as_str().to_string())),
            device_ip_address: Set(completion.device_ip_address),
            signature_json: Set(completion.signature_value),
            server_random: Set(Some(server_random)),
            user_challenge: Set(completion.user_challenge),
            cert_value: Set(completion.cert_value),
            cert_level: Set(completion.cert_level),
            updated_at: Set(Utc::now().into()),
            ..Default::default()
        };

        session::Entity::update(update)
            .exec(db)
            .await
            .map_err(ServiceError::Db)?;

        notifier.notify(session_id).await;

        session::Entity::find_by_id(session_id)
            .one(db)
            .await
            .map_err(ServiceError::Db)?
            .ok_or(ServiceError::NotFound("session".to_string()))
    }
}

fn model_to_response(m: &session::Model) -> SessionStatusResponse {
    let state = match m.state.as_str() {
        "COMPLETE" => SessionState::Complete,
        _ => SessionState::Running,
    };

    let result = if state == SessionState::Complete {
        Some(SessionResult {
            end_result: SessionEndResult::from_str(
                m.end_result.as_deref().unwrap_or("SERVER_ERROR"),
            ),
            document_number: m.document_number.clone(),
            details: None,
        })
    } else {
        None
    };

    let cert = m.cert_value.as_ref().map(|value| CertificateInfo {
        value: value.clone(),
        certificate_level: m.cert_level.clone().unwrap_or_default(),
    });

    SessionStatusResponse {
        state,
        result,
        signature_protocol: m
            .signature_protocol
            .as_deref()
            .and_then(SessionSignatureProtocol::from_str),
        signature: build_signature(m),
        cert,
        interaction_type_used: m
            .interaction_type_used
            .as_deref()
            .and_then(InteractionType::from_str),
        device_ip_address: m.device_ip_address.clone(),
        ignored_properties: m
            .ignored_properties_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok()),
    }
}

fn build_signature(m: &session::Model) -> Option<SessionSignature> {
    if m.state != "COMPLETE" || m.end_result.as_deref() != Some("OK") {
        return None;
    }

    let flow_type = m.flow_type.as_deref().and_then(FlowType::from_str)?;
    let sig_alg = m
        .signature_algorithm
        .as_deref()
        .and_then(SignatureAlgorithm::from_str)?;
    let hash_alg = m
        .hash_algorithm
        .as_deref()
        .and_then(HashAlgorithm::from_str);

    let sig_alg_params = hash_alg
        .as_ref()
        .map(|ha| SignatureAlgorithmParametersInResponse {
            hash_algorithm: Some(ha.clone()),
            mask_gen_algorithm: Some(MaskGenAlgorithm {
                algorithm: MaskGenAlgorithmId::IdMgf1,
                parameters: Some(MaskGenAlgorithmParameters {
                    hash_algorithm: ha.clone(),
                }),
            }),
            salt_length: Some(ha.salt_length()),
            trailer_field: Some("0xbc".to_string()),
        });

    match m.kind.as_str() {
        "authentication" => Some(SessionSignature::AcspV2(AcspV2Signature {
            value: m.signature_json.clone().unwrap_or_default(),
            server_random: m.server_random.clone().unwrap_or_default(),
            user_challenge: m.user_challenge.clone().unwrap_or_default(),
            flow_type,
            signature_algorithm: sig_alg,
            signature_algorithm_parameters: sig_alg_params,
        })),
        "signing" => Some(SessionSignature::RawDigest(RawDigestSignature {
            value: m.signature_json.clone().unwrap_or_default(),
            flow_type,
            signature_algorithm: sig_alg,
            signature_algorithm_parameters: sig_alg_params,
        })),
        "certificate_choice" => Some(SessionSignature::CertificateChoice(
            CertificateChoiceSignature { flow_type },
        )),
        _ => None,
    }
}

pub struct CreateSessionParams {
    pub relying_party_id: String,
    pub account_id: Option<String>,
    pub kind: SessionKind,
    pub session_token: Option<String>,
    pub session_secret: Option<String>,
    pub device_link_base: Option<String>,
    pub signature_protocol: Option<String>,
    pub signature_algorithm: Option<SignatureAlgorithm>,
    pub hash_algorithm: Option<HashAlgorithm>,
    pub certificate_level: Option<String>,
    pub challenge_or_digest: Option<String>,
    pub interactions: Option<String>,
    pub nonce: Option<String>,
    pub initial_callback_url: Option<String>,
    pub linked_session_id: Option<String>,
    pub vc_type: Option<String>,
    pub vc_value: Option<String>,
    pub share_md_client_ip_address: bool,
}

pub struct SessionCompletion {
    pub end_result: SessionEndResult,
    pub document_number: Option<String>,
    pub flow_type: Option<FlowType>,
    pub interaction_type_used: Option<InteractionType>,
    pub device_ip_address: Option<String>,
    pub signature_value: Option<String>,
    pub user_challenge: Option<String>,
    pub cert_value: Option<String>,
    pub cert_level: Option<String>,
}

#[derive(Debug, Clone)]
pub enum SessionKind {
    Authentication,
    Signing,
    CertificateChoice,
}

impl SessionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Authentication => "authentication",
            Self::Signing => "signing",
            Self::CertificateChoice => "certificate_choice",
        }
    }
}
