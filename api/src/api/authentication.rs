use poem::web::Data;
use poem_openapi::{OpenApi, param::Path, payload::Json};

use crate::AppState;
use crate::models::*;
use crate::services::account::AccountService;
use crate::services::relying_party::RelyingPartyService;
use crate::services::session::{CreateSessionParams, SessionKind, SessionService};

use super::helpers::{device_link_credentials, device_link_response, send_fcm_push, share_md_client_ip};

pub struct AuthenticationApi;

#[OpenApi(
    prefix_path = "/v3/authentication",
    tag = "super::ApiTags::Authentication"
)]
impl AuthenticationApi {
    /// Device link based authentication session with ETSI Natural Person Semantics Identifier
    #[oai(
        path = "/device-link/etsi/:id_etsi",
        method = "post",
        operation_id = "authDeviceLinkSemanticId"
    )]
    async fn auth_device_link_semantic_id(
        &self,
        state: Data<&AppState>,
        #[oai(name = "id_etsi")] id_etsi: Path<String>,
        body: Json<DeviceLinkAuthenticationRequest>,
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
            device_link_auth_params(&rp.id, Some(account.id), &body.0),
        )
        .await?;
        Ok(Json(device_link_response(&session)))
    }

    /// Device link based authentication session with document number
    #[oai(
        path = "/device-link/document/:document_number",
        method = "post",
        operation_id = "authDeviceLinkDocumentNumber"
    )]
    async fn auth_device_link_document_number(
        &self,
        state: Data<&AppState>,
        document_number: Path<String>,
        body: Json<DeviceLinkAuthenticationRequest>,
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
            device_link_auth_params(&rp.id, Some(account.id), &body.0),
        )
        .await?;
        Ok(Json(device_link_response(&session)))
    }

    /// Anonymous device link based authentication session
    #[oai(
        path = "/device-link/anonymous",
        method = "post",
        operation_id = "authDeviceLinkAnonymous"
    )]
    async fn auth_device_link_anonymous(
        &self,
        state: Data<&AppState>,
        body: Json<DeviceLinkAuthenticationRequest>,
    ) -> Result<Json<DeviceLinkResponse>, ApiErrorResponse> {
        let rp = RelyingPartyService::validate(
            &state.db,
            &body.relying_party_uuid,
            &body.relying_party_name,
        )
        .await?;
        let session =
            SessionService::create(&state.db, device_link_auth_params(&rp.id, None, &body.0))
                .await?;
        Ok(Json(device_link_response(&session)))
    }

    /// Notification based authentication session with ETSI Natural Person Semantics Identifier
    #[oai(
        path = "/notification/etsi/:id_etsi",
        method = "post",
        operation_id = "authNotificationSemanticId"
    )]
    async fn auth_notification_semantic_id(
        &self,
        state: Data<&AppState>,
        #[oai(name = "id_etsi")] id_etsi: Path<String>,
        body: Json<NotificationAuthenticationRequest>,
    ) -> Result<Json<NotificationAuthenticationResponse>, ApiErrorResponse> {
        let rp = RelyingPartyService::validate(
            &state.db,
            &body.relying_party_uuid,
            &body.relying_party_name,
        )
        .await?;
        let account = AccountService::find_or_create_by_semantic_id(&state.db, &id_etsi.0).await?;
        let session = SessionService::create(
            &state.db,
            notification_auth_params(&rp.id, Some(account.id.clone()), &body.0),
        )
        .await?;
        send_fcm_push(
            &state,
            &account.id,
            &session.id,
            "authentication",
            &body.relying_party_name,
        );
        Ok(Json(NotificationAuthenticationResponse {
            session_id: session.id,
        }))
    }

    /// Notification based authentication session with document number
    #[oai(
        path = "/notification/document/:document_number",
        method = "post",
        operation_id = "authNotificationDocumentNumber"
    )]
    async fn auth_notification_document_number(
        &self,
        state: Data<&AppState>,
        document_number: Path<String>,
        body: Json<NotificationAuthenticationRequest>,
    ) -> Result<Json<NotificationAuthenticationResponse>, ApiErrorResponse> {
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
            notification_auth_params(&rp.id, Some(account.id.clone()), &body.0),
        )
        .await?;
        send_fcm_push(
            &state,
            &account.id,
            &session.id,
            "authentication",
            &body.relying_party_name,
        );
        Ok(Json(NotificationAuthenticationResponse {
            session_id: session.id,
        }))
    }
}

fn device_link_auth_params(
    rp_id: &str,
    account_id: Option<String>,
    req: &DeviceLinkAuthenticationRequest,
) -> CreateSessionParams {
    let (token, secret) = device_link_credentials();
    let pp = &req.signature_protocol_parameters;

    CreateSessionParams {
        relying_party_id: rp_id.to_string(),
        account_id,
        kind: SessionKind::Authentication,
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
        challenge_or_digest: Some(pp.rp_challenge.clone()),
        interactions: Some(req.interactions.clone()),
        nonce: None,
        initial_callback_url: Some(req.initial_callback_url.clone()),
        linked_session_id: None,
        vc_type: None,
        vc_value: None,
        share_md_client_ip_address: share_md_client_ip(req.request_properties.as_ref()),
    }
}

fn notification_auth_params(
    rp_id: &str,
    account_id: Option<String>,
    req: &NotificationAuthenticationRequest,
) -> CreateSessionParams {
    let pp = &req.signature_protocol_parameters;

    CreateSessionParams {
        relying_party_id: rp_id.to_string(),
        account_id,
        kind: SessionKind::Authentication,
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
        challenge_or_digest: Some(pp.rp_challenge.clone()),
        interactions: Some(req.interactions.clone()),
        nonce: None,
        initial_callback_url: None,
        linked_session_id: None,
        vc_type: Some("numeric4".to_string()),
        vc_value: None,
        share_md_client_ip_address: share_md_client_ip(req.request_properties.as_ref()),
    }
}
