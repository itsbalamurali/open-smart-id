use crate::AppState;
use crate::db::entities::session;
use crate::models::*;
use crate::services::account::AccountService;
use crate::services::device::DeviceService;
use crate::services::session::DeviceLinkResponse;

/// Generate a device-link session token and secret.
pub fn device_link_credentials() -> (String, String) {
    let token = uuid::Uuid::new_v4().to_string().replace('-', "");
    let secret = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        uuid::Uuid::new_v4().as_bytes(),
    );
    (token, secret)
}

/// Build a DeviceLinkResponse from a session model.
pub fn device_link_response(session: &session::Model) -> DeviceLinkResponse {
    DeviceLinkResponse {
        session_id: session.id.clone(),
        session_token: session.session_token.clone().unwrap_or_default(),
        session_secret: session.session_secret.clone().unwrap_or_default(),
        device_link_base: session.device_link_base.clone().unwrap_or_default(),
    }
}

/// Extract share_md_client_ip_address from optional RequestProperties.
pub fn share_md_client_ip(props: Option<&RequestProperties>) -> bool {
    props
        .and_then(|p| p.share_md_client_ip_address)
        .unwrap_or(false)
}

/// Send FCM push notification in background (fire-and-forget).
pub fn send_fcm_push(
    state: &AppState,
    account_id: &str,
    session_id: &str,
    kind: &str,
    rp_name: &str,
) {
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

/// Resolve certificate for session completion (used by AppApi and InternalApi).
pub async fn resolve_cert_for_completion(
    state: &AppState,
    sess: &session::Model,
) -> Result<(Option<String>, Option<String>, Option<String>), ApiErrorResponse> {
    let account = match &sess.account_id {
        Some(id) => Some(AccountService::find_by_id(&state.db, id).await?),
        None => Some(AccountService::create_anonymous(&state.db).await?),
    };

    let Some(acct) = account else {
        return Ok((None, None, None));
    };

    let purpose = match sess.kind.as_str() {
        "authentication" => Some("authentication"),
        "signing" => Some("signing"),
        _ => None,
    };

    if let Some(purpose) = purpose {
        let cert = state
            .certificate
            .get_or_issue_cert(&state.db, &acct.id, &acct.document_number, purpose)
            .await?;
        Ok((
            Some(acct.document_number),
            Some(cert.cert_value),
            Some(cert.cert_level),
        ))
    } else {
        Ok((Some(acct.document_number), None, None))
    }
}
