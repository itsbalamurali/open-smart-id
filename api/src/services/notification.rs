use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::db::entities::device;

/// FCM push notification service using the HTTP v1 API.
///
/// Requires env vars:
/// - `FCM_PROJECT_ID` — Google Cloud project ID
/// - `GOOGLE_SERVICE_ACCOUNT_KEY` — path to service account JSON key file
///
/// Returns `None` from `from_env()` if not configured, allowing the server
/// to run without FCM.
#[derive(Clone)]
pub struct NotificationService {
    inner: Arc<Inner>,
}

struct Inner {
    client: Client,
    project_id: String,
    service_account_email: String,
    private_key_pem: String,
    token_cache: Mutex<Option<CachedToken>>,
}

struct CachedToken {
    access_token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
struct ServiceAccountKey {
    client_email: String,
    private_key: String,
}

#[derive(Serialize)]
struct JwtClaims {
    iss: String,
    scope: String,
    aud: String,
    iat: i64,
    exp: i64,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
}

#[derive(Serialize)]
struct FcmMessage {
    message: FcmMessageBody,
}

#[derive(Serialize)]
struct FcmMessageBody {
    token: String,
    notification: FcmNotification,
    data: std::collections::HashMap<String, String>,
}

#[derive(Serialize)]
struct FcmNotification {
    title: String,
    body: String,
}

impl NotificationService {
    /// Create from environment variables. Returns `None` if FCM is not configured.
    pub fn from_env() -> Option<Self> {
        let project_id = std::env::var("FCM_PROJECT_ID").ok()?;
        let key_path = std::env::var("GOOGLE_SERVICE_ACCOUNT_KEY").ok()?;

        let key_json = std::fs::read_to_string(&key_path)
            .inspect_err(|e| tracing::warn!("failed to read service account key: {e}"))
            .ok()?;
        let key: ServiceAccountKey = serde_json::from_str(&key_json)
            .inspect_err(|e| tracing::warn!("failed to parse service account key: {e}"))
            .ok()?;

        tracing::info!("FCM notifications enabled for project {project_id}");

        Some(Self {
            inner: Arc::new(Inner {
                client: Client::new(),
                project_id,
                service_account_email: key.client_email,
                private_key_pem: key.private_key,
                token_cache: Mutex::new(None),
            }),
        })
    }

    async fn get_access_token(&self) -> Result<String, NotificationError> {
        {
            let cache = self.inner.token_cache.lock().await;
            if let Some(ref cached) = *cache {
                if cached.expires_at > chrono::Utc::now() + chrono::Duration::seconds(60) {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        let now = chrono::Utc::now();
        let claims = JwtClaims {
            iss: self.inner.service_account_email.clone(),
            scope: "https://www.googleapis.com/auth/firebase.messaging".to_string(),
            aud: "https://oauth2.googleapis.com/token".to_string(),
            iat: now.timestamp(),
            exp: (now + chrono::Duration::seconds(3600)).timestamp(),
        };

        let encoding_key =
            jsonwebtoken::EncodingKey::from_rsa_pem(self.inner.private_key_pem.as_bytes())
                .map_err(|e| NotificationError::Auth(e.to_string()))?;

        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
        let jwt = jsonwebtoken::encode(&header, &claims, &encoding_key)
            .map_err(|e| NotificationError::Auth(e.to_string()))?;

        let http_resp = self
            .inner
            .client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await
            .map_err(|e: reqwest::Error| NotificationError::Http(e.to_string()))?;
        let resp: TokenResponse = http_resp
            .json()
            .await
            .map_err(|e: reqwest::Error| NotificationError::Http(e.to_string()))?;

        let mut cache = self.inner.token_cache.lock().await;
        *cache = Some(CachedToken {
            access_token: resp.access_token.clone(),
            expires_at: now + chrono::Duration::seconds(resp.expires_in),
        });

        Ok(resp.access_token)
    }

    async fn send_to_device(
        &self,
        fcm_token: &str,
        title: &str,
        body: &str,
        data: std::collections::HashMap<String, String>,
    ) -> Result<(), NotificationError> {
        let access_token = self.get_access_token().await?;
        let url = format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            self.inner.project_id
        );

        let message = FcmMessage {
            message: FcmMessageBody {
                token: fcm_token.to_string(),
                notification: FcmNotification {
                    title: title.to_string(),
                    body: body.to_string(),
                },
                data,
            },
        };

        let resp = self
            .inner
            .client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&message)
            .send()
            .await
            .map_err(|e: reqwest::Error| NotificationError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            tracing::warn!("FCM send failed: status={status}, body={text}");
            return Err(NotificationError::FcmError(format!("{status}: {text}")));
        }

        Ok(())
    }

    /// Send push notification to all active devices for an account.
    pub async fn notify_session_created(
        &self,
        devices: &[device::Model],
        session_id: &str,
        session_kind: &str,
        rp_name: &str,
    ) {
        let body = match session_kind {
            "authentication" => format!("{rp_name} requests authentication"),
            "signing" => format!("{rp_name} requests signature"),
            _ => format!("{rp_name} requests action"),
        };

        for device in devices {
            let mut data = std::collections::HashMap::new();
            data.insert("sessionId".to_string(), session_id.to_string());
            data.insert("sessionKind".to_string(), session_kind.to_string());

            if let Err(e) = self
                .send_to_device(&device.fcm_token, "SmartID", &body, data)
                .await
            {
                tracing::warn!(device_id = %device.id, error = %e, "failed to send FCM push");
            }
        }
    }
}

#[derive(Debug)]
pub enum NotificationError {
    Auth(String),
    Http(String),
    FcmError(String),
}

impl std::fmt::Display for NotificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auth(msg) => write!(f, "auth error: {msg}"),
            Self::Http(msg) => write!(f, "http error: {msg}"),
            Self::FcmError(msg) => write!(f, "FCM error: {msg}"),
        }
    }
}
