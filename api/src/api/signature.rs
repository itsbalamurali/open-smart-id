use poem::web::Data;
use poem_openapi::{OpenApi, param::Path, payload::Json};

use crate::AppState;
use crate::models::*;
use crate::services::account::AccountService;
use crate::services::device::DeviceService;
use crate::services::relying_party::RelyingPartyService;
use crate::services::session::{CreateSessionParams, SessionKind, SessionService};

pub struct SignatureApi;

#[OpenApi(prefix_path = "/v3/signature", tag = "super::ApiTags::Signature")]
impl SignatureApi {
    /// Device link based signing session with ETSI Natural Person Semantics Identifier
    #[oai(
        path = "/device-link/etsi/:id_etsi",
        method = "post",
        operation_id = "signDeviceLinkSemanticId"
    )]
    async fn sign_device_link_semantic_id(
        &self,
        state: Data<&AppState>,
        #[oai(name = "id_etsi")] id_etsi: Path<String>,
        body: Json<DeviceLinkSigningRequest>,
    ) -> Result<Json<DeviceLinkResponse>, ApiErrorResponse> {
        let rp = RelyingPartyService::validate(
            &state.db,
            &body.relying_party_uuid,
            &body.relying_party_name,
        )
        .await?;
        let account = AccountService::find_or_create_by_semantic_id(&state.db, &id_etsi.0).await?;
        let session = SessionService::create(
            &state.db,
            device_link_sign_params(&rp.id, Some(account.id), &body.0),
        )
        .await?;
        Ok(Json(device_link_response(&session)))
    }

    /// Device link based signing session with document number
    #[oai(
        path = "/device-link/document/:document_number",
        method = "post",
        operation_id = "signDeviceLinkDocumentNumber"
    )]
    async fn sign_device_link_document_number(
        &self,
        state: Data<&AppState>,
        document_number: Path<String>,
        body: Json<DeviceLinkSigningRequest>,
    ) -> Result<Json<DeviceLinkResponse>, ApiErrorResponse> {
        let rp = RelyingPartyService::validate(
            &state.db,
            &body.relying_party_uuid,
            &body.relying_party_name,
        )
        .await?;
        let account =
            AccountService::find_by_document_number(&state.db, &document_number.0).await?;
        let session = SessionService::create(
            &state.db,
            device_link_sign_params(&rp.id, Some(account.id), &body.0),
        )
        .await?;
        Ok(Json(device_link_response(&session)))
    }

    /// Notification based signing session with ETSI Natural Person Semantics Identifier
    #[oai(
        path = "/notification/etsi/:id_etsi",
        method = "post",
        operation_id = "signNotificationSemanticId"
    )]
    async fn sign_notification_semantic_id(
        &self,
        state: Data<&AppState>,
        #[oai(name = "id_etsi")] id_etsi: Path<String>,
        body: Json<NotificationSigningRequest>,
    ) -> Result<Json<NotificationSigningResponse>, ApiErrorResponse> {
        let rp = RelyingPartyService::validate(
            &state.db,
            &body.relying_party_uuid,
            &body.relying_party_name,
        )
        .await?;
        let account = AccountService::find_or_create_by_semantic_id(&state.db, &id_etsi.0).await?;
        let session = SessionService::create(
            &state.db,
            notification_sign_params(&rp.id, Some(account.id.clone()), &body.0),
        )
        .await?;
        send_fcm_push(
            &state,
            &account.id,
            &session.id,
            "signing",
            &body.relying_party_name,
        );
        Ok(Json(notification_sign_response(&session)))
    }

    /// Notification based signing session with document number
    #[oai(
        path = "/notification/document/:document_number",
        method = "post",
        operation_id = "signNotificationDocumentNumber"
    )]
    async fn sign_notification_document_number(
        &self,
        state: Data<&AppState>,
        document_number: Path<String>,
        body: Json<NotificationSigningRequest>,
    ) -> Result<Json<NotificationSigningResponse>, ApiErrorResponse> {
        let rp = RelyingPartyService::validate(
            &state.db,
            &body.relying_party_uuid,
            &body.relying_party_name,
        )
        .await?;
        let account =
            AccountService::find_by_document_number(&state.db, &document_number.0).await?;
        let session = SessionService::create(
            &state.db,
            notification_sign_params(&rp.id, Some(account.id.clone()), &body.0),
        )
        .await?;
        send_fcm_push(
            &state,
            &account.id,
            &session.id,
            "signing",
            &body.relying_party_name,
        );
        Ok(Json(notification_sign_response(&session)))
    }

    /// Notification based signing session linked to a previous session
    #[oai(
        path = "/notification/linked/:document_number",
        method = "post",
        operation_id = "signNotificationLinked"
    )]
    async fn sign_notification_linked(
        &self,
        state: Data<&AppState>,
        document_number: Path<String>,
        body: Json<NotificationSigningLinkedRequest>,
    ) -> Result<Json<NotificationSigningLinkedResponse>, ApiErrorResponse> {
        let rp = RelyingPartyService::validate(
            &state.db,
            &body.relying_party_uuid,
            &body.relying_party_name,
        )
        .await?;
        let account =
            AccountService::find_by_document_number(&state.db, &document_number.0).await?;
        let pp = &body.signature_protocol_parameters;
        let session = SessionService::create(
            &state.db,
            CreateSessionParams {
                relying_party_id: rp.id,
                account_id: Some(account.id.clone()),
                kind: SessionKind::Signing,
                session_token: None,
                session_secret: None,
                device_link_base: None,
                signature_protocol: Some(body.signature_protocol.as_str().to_string()),
                signature_algorithm: Some(pp.signature_algorithm.clone()),
                hash_algorithm: pp
                    .signature_algorithm_parameters
                    .as_ref()
                    .map(|p| p.hash_algorithm.clone()),
                certificate_level: body
                    .certificate_level
                    .as_ref()
                    .map(|l| l.as_str().to_string()),
                challenge_or_digest: Some(pp.digest.clone()),
                interactions: Some(body.interactions.clone()),
                nonce: body.nonce.clone(),
                initial_callback_url: None,
                linked_session_id: Some(body.linked_session_id.clone()),
                vc_type: None,
                vc_value: None,
                share_md_client_ip_address: body
                    .request_properties
                    .as_ref()
                    .and_then(|p| p.share_md_client_ip_address)
                    .unwrap_or(false),
            },
        )
        .await?;
        send_fcm_push(
            &state,
            &account.id,
            &session.id,
            "signing",
            &body.relying_party_name,
        );
        Ok(Json(NotificationSigningLinkedResponse {
            session_id: session.id,
        }))
    }

    /// Get the signing certificate of the requested document number
    #[oai(
        path = "/certificate/:document_number",
        method = "post",
        operation_id = "signCertificateDocumentNumber"
    )]
    async fn sign_certificate_document_number(
        &self,
        state: Data<&AppState>,
        document_number: Path<String>,
        body: Json<SigningCertificateRequest>,
    ) -> Result<Json<CertificateResponse>, ApiErrorResponse> {
        let _rp = RelyingPartyService::validate(
            &state.db,
            &body.relying_party_uuid,
            &body.relying_party_name,
        )
        .await?;
        let account =
            AccountService::find_by_document_number(&state.db, &document_number.0).await?;

        match state
            .certificate
            .get_or_issue_signing_cert(&state.db, &account.id, &account.document_number)
            .await
        {
            Ok(cert) => Ok(Json(CertificateResponse {
                state: CertificateState::Ok,
                cert: Some(CertificateInfo {
                    value: cert.cert_value,
                    certificate_level: cert.cert_level,
                }),
            })),
            Err(_) => Ok(Json(CertificateResponse {
                state: CertificateState::DocumentUnusable,
                cert: None,
            })),
        }
    }

    /// Anonymous device link based certificate choice session
    #[oai(
        path = "/certificate-choice/device-link/anonymous",
        method = "post",
        operation_id = "signCertChoiceDeviceLinkAnonymous"
    )]
    async fn sign_cert_choice_device_link_anonymous(
        &self,
        state: Data<&AppState>,
        body: Json<DeviceLinkCertificateChoiceRequest>,
    ) -> Result<Json<DeviceLinkResponse>, ApiErrorResponse> {
        let rp = RelyingPartyService::validate(
            &state.db,
            &body.relying_party_uuid,
            &body.relying_party_name,
        )
        .await?;
        let token = uuid::Uuid::new_v4().to_string().replace('-', "");
        let secret = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            uuid::Uuid::new_v4().as_bytes(),
        );
        let session = SessionService::create(
            &state.db,
            CreateSessionParams {
                relying_party_id: rp.id,
                account_id: None,
                kind: SessionKind::CertificateChoice,
                session_token: Some(token),
                session_secret: Some(secret),
                device_link_base: Some("https://sid.demo.sk.ee/device-link".to_string()),
                signature_protocol: None,
                signature_algorithm: None,
                hash_algorithm: None,
                certificate_level: body
                    .certificate_level
                    .as_ref()
                    .map(|l| l.as_str().to_string()),
                challenge_or_digest: None,
                interactions: None,
                nonce: body.nonce.clone(),
                initial_callback_url: body.initial_callback_url.clone(),
                linked_session_id: None,
                vc_type: None,
                vc_value: None,
                share_md_client_ip_address: body
                    .request_properties
                    .as_ref()
                    .and_then(|p| p.share_md_client_ip_address)
                    .unwrap_or(false),
            },
        )
        .await?;
        Ok(Json(device_link_response(&session)))
    }

    /// Notification based certificate choice session with ETSI semantic identifier
    #[oai(
        path = "/certificate-choice/notification/etsi/:id_etsi",
        method = "post",
        operation_id = "signCertChoiceNotificationSemanticId"
    )]
    async fn sign_cert_choice_notification_semantic_id(
        &self,
        state: Data<&AppState>,
        #[oai(name = "id_etsi")] id_etsi: Path<String>,
        body: Json<NotificationCertificateChoiceRequest>,
    ) -> Result<Json<NotificationCertificateChoiceResponse>, ApiErrorResponse> {
        let rp = RelyingPartyService::validate(
            &state.db,
            &body.relying_party_uuid,
            &body.relying_party_name,
        )
        .await?;
        let account = AccountService::find_or_create_by_semantic_id(&state.db, &id_etsi.0).await?;
        let session = SessionService::create(
            &state.db,
            CreateSessionParams {
                relying_party_id: rp.id,
                account_id: Some(account.id),
                kind: SessionKind::CertificateChoice,
                session_token: None,
                session_secret: None,
                device_link_base: None,
                signature_protocol: None,
                signature_algorithm: None,
                hash_algorithm: None,
                certificate_level: body
                    .certificate_level
                    .as_ref()
                    .map(|l| l.as_str().to_string()),
                challenge_or_digest: None,
                interactions: None,
                nonce: body.nonce.clone(),
                initial_callback_url: None,
                linked_session_id: None,
                vc_type: None,
                vc_value: None,
                share_md_client_ip_address: body
                    .request_properties
                    .as_ref()
                    .and_then(|p| p.share_md_client_ip_address)
                    .unwrap_or(false),
            },
        )
        .await?;
        Ok(Json(NotificationCertificateChoiceResponse {
            session_id: session.id,
        }))
    }
}

fn device_link_sign_params(
    rp_id: &str,
    account_id: Option<String>,
    req: &DeviceLinkSigningRequest,
) -> CreateSessionParams {
    let token = uuid::Uuid::new_v4().to_string().replace('-', "");
    let secret = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        uuid::Uuid::new_v4().as_bytes(),
    );
    let pp = &req.signature_protocol_parameters;

    CreateSessionParams {
        relying_party_id: rp_id.to_string(),
        account_id,
        kind: SessionKind::Signing,
        session_token: Some(token),
        session_secret: Some(secret),
        device_link_base: Some("https://sid.demo.sk.ee/device-link".to_string()),
        signature_protocol: Some(req.signature_protocol.as_str().to_string()),
        signature_algorithm: Some(pp.signature_algorithm.clone()),
        hash_algorithm: pp
            .signature_algorithm_parameters
            .as_ref()
            .map(|p| p.hash_algorithm.clone()),
        certificate_level: req
            .certificate_level
            .as_ref()
            .map(|l| l.as_str().to_string()),
        challenge_or_digest: Some(pp.digest.clone()),
        interactions: Some(req.interactions.clone()),
        nonce: req.nonce.clone(),
        initial_callback_url: req.initial_callback_url.clone(),
        linked_session_id: None,
        vc_type: None,
        vc_value: None,
        share_md_client_ip_address: req
            .request_properties
            .as_ref()
            .and_then(|p| p.share_md_client_ip_address)
            .unwrap_or(false),
    }
}

fn notification_sign_params(
    rp_id: &str,
    account_id: Option<String>,
    req: &NotificationSigningRequest,
) -> CreateSessionParams {
    let pp = &req.signature_protocol_parameters;
    let vc_value = format!("{:04}", rand_vc());

    CreateSessionParams {
        relying_party_id: rp_id.to_string(),
        account_id,
        kind: SessionKind::Signing,
        session_token: None,
        session_secret: None,
        device_link_base: None,
        signature_protocol: Some(req.signature_protocol.as_str().to_string()),
        signature_algorithm: Some(pp.signature_algorithm.clone()),
        hash_algorithm: pp
            .signature_algorithm_parameters
            .as_ref()
            .map(|p| p.hash_algorithm.clone()),
        certificate_level: req
            .certificate_level
            .as_ref()
            .map(|l| l.as_str().to_string()),
        challenge_or_digest: Some(pp.digest.clone()),
        interactions: Some(req.interactions.clone()),
        nonce: req.nonce.clone(),
        initial_callback_url: None,
        linked_session_id: None,
        vc_type: Some("numeric4".to_string()),
        vc_value: Some(vc_value),
        share_md_client_ip_address: req
            .request_properties
            .as_ref()
            .and_then(|p| p.share_md_client_ip_address)
            .unwrap_or(false),
    }
}

fn device_link_response(session: &crate::db::entities::session::Model) -> DeviceLinkResponse {
    DeviceLinkResponse {
        session_id: session.id.clone(),
        session_token: session.session_token.clone().unwrap_or_default(),
        session_secret: session.session_secret.clone().unwrap_or_default(),
        device_link_base: session.device_link_base.clone().unwrap_or_default(),
    }
}

fn notification_sign_response(
    session: &crate::db::entities::session::Model,
) -> NotificationSigningResponse {
    NotificationSigningResponse {
        session_id: session.id.clone(),
        vc: VC {
            vc_type: VerificationCodeType::Numeric4,
            value: session.vc_value.clone().unwrap_or_default(),
        },
    }
}

fn rand_vc() -> u16 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    (RandomState::new().build_hasher().finish() % 10000) as u16
}

fn send_fcm_push(state: &AppState, account_id: &str, session_id: &str, kind: &str, rp_name: &str) {
    if let Some(ref notif) = state.notification {
        let notif = notif.clone();
        let db = state.db.clone();
        let account_id = account_id.to_string();
        let session_id = session_id.to_string();
        let kind = kind.to_string();
        let rp_name = rp_name.to_string();
        tokio::spawn(async move {
            let devices = DeviceService::find_active_by_account(&db, &account_id)
                .await
                .unwrap_or_default();
            if !devices.is_empty() {
                notif
                    .notify_session_created(&devices, &session_id, &kind, &rp_name)
                    .await;
            }
        });
    }
}
