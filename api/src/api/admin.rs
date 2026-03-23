use chrono::Utc;
use poem::web::Data;
use poem_openapi::{Object, OpenApi, param::Path, param::Query, payload::Json};
use sea_orm::*;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::db::entities::{account, certificate, device, relying_party, session};
use crate::models::*;

pub struct AdminApi;

// ── Relying Party DTOs ──

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct CreateRelyingPartyRequest {
    pub uuid: String,
    pub name: String,
    #[oai(rename = "logoUrl", skip_serializing_if_is_none)]
    pub logo_url: Option<String>,
    #[oai(rename = "websiteUrl", skip_serializing_if_is_none)]
    pub website_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct UpdateRelyingPartyRequest {
    #[oai(skip_serializing_if_is_none)]
    pub name: Option<String>,
    #[oai(rename = "logoUrl", skip_serializing_if_is_none)]
    pub logo_url: Option<String>,
    #[oai(rename = "websiteUrl", skip_serializing_if_is_none)]
    pub website_url: Option<String>,
    #[oai(rename = "isActive", skip_serializing_if_is_none)]
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct RelyingPartyResponse {
    pub id: String,
    pub uuid: String,
    pub name: String,
    #[oai(rename = "logoUrl", skip_serializing_if_is_none)]
    pub logo_url: Option<String>,
    #[oai(rename = "websiteUrl", skip_serializing_if_is_none)]
    pub website_url: Option<String>,
    #[oai(rename = "isActive")]
    pub is_active: bool,
    #[oai(rename = "createdAt")]
    pub created_at: String,
    #[oai(rename = "updatedAt")]
    pub updated_at: String,
}

impl From<relying_party::Model> for RelyingPartyResponse {
    fn from(m: relying_party::Model) -> Self {
        Self {
            id: m.id,
            uuid: m.uuid,
            name: m.name,
            logo_url: m.logo_url,
            website_url: m.website_url,
            is_active: m.is_active,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}

// ── Account DTOs ──

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct AccountResponse {
    pub id: String,
    #[oai(rename = "semanticId", skip_serializing_if_is_none)]
    pub semantic_id: Option<String>,
    #[oai(rename = "documentNumber")]
    pub document_number: String,
    #[oai(rename = "identityType", skip_serializing_if_is_none)]
    pub identity_type: Option<String>,
    #[oai(rename = "countryCode", skip_serializing_if_is_none)]
    pub country_code: Option<String>,
    #[oai(rename = "nationalIdentityNumber", skip_serializing_if_is_none)]
    pub national_identity_number: Option<String>,
    pub status: String,
    #[oai(rename = "createdAt")]
    pub created_at: String,
    #[oai(rename = "updatedAt")]
    pub updated_at: String,
}

impl From<account::Model> for AccountResponse {
    fn from(m: account::Model) -> Self {
        Self {
            id: m.id,
            semantic_id: m.semantic_id,
            document_number: m.document_number,
            identity_type: m.identity_type,
            country_code: m.country_code,
            national_identity_number: m.national_identity_number,
            status: m.status,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct UpdateAccountRequest {
    #[oai(skip_serializing_if_is_none)]
    pub status: Option<String>,
}

// ── Session DTOs ──

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct SessionResponse {
    pub id: String,
    #[oai(rename = "relyingPartyId")]
    pub relying_party_id: String,
    #[oai(rename = "accountId", skip_serializing_if_is_none)]
    pub account_id: Option<String>,
    pub kind: String,
    pub state: String,
    #[oai(rename = "endResult", skip_serializing_if_is_none)]
    pub end_result: Option<String>,
    #[oai(rename = "documentNumber", skip_serializing_if_is_none)]
    pub document_number: Option<String>,
    #[oai(rename = "flowType", skip_serializing_if_is_none)]
    pub flow_type: Option<String>,
    #[oai(rename = "createdAt")]
    pub created_at: String,
    #[oai(rename = "updatedAt")]
    pub updated_at: String,
}

impl From<session::Model> for SessionResponse {
    fn from(m: session::Model) -> Self {
        Self {
            id: m.id,
            relying_party_id: m.relying_party_id,
            account_id: m.account_id,
            kind: m.kind,
            state: m.state,
            end_result: m.end_result,
            document_number: m.document_number,
            flow_type: m.flow_type,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}

// ── Device DTOs ──

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct DeviceResponse {
    pub id: String,
    #[oai(rename = "accountId")]
    pub account_id: String,
    #[oai(rename = "deviceName", skip_serializing_if_is_none)]
    pub device_name: Option<String>,
    pub platform: String,
    #[oai(rename = "isActive")]
    pub is_active: bool,
    #[oai(rename = "createdAt")]
    pub created_at: String,
    #[oai(rename = "updatedAt")]
    pub updated_at: String,
}

impl From<device::Model> for DeviceResponse {
    fn from(m: device::Model) -> Self {
        Self {
            id: m.id,
            account_id: m.account_id,
            device_name: m.device_name,
            platform: m.platform,
            is_active: m.is_active,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}

// ── Certificate DTOs ──

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct CertificateResponse {
    pub id: String,
    #[oai(rename = "accountId")]
    pub account_id: String,
    #[oai(rename = "certType")]
    pub cert_type: String,
    #[oai(rename = "certLevel")]
    pub cert_level: String,
    #[oai(rename = "isActive")]
    pub is_active: bool,
    #[oai(rename = "createdAt")]
    pub created_at: String,
    #[oai(rename = "expiresAt")]
    pub expires_at: String,
}

impl From<certificate::Model> for CertificateResponse {
    fn from(m: certificate::Model) -> Self {
        Self {
            id: m.id,
            account_id: m.account_id,
            cert_type: m.cert_type,
            cert_level: m.cert_level,
            is_active: m.is_active,
            created_at: m.created_at.to_rfc3339(),
            expires_at: m.expires_at.to_rfc3339(),
        }
    }
}

// ── Paginated list wrapper ──

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct PaginatedResponse<T: poem_openapi::types::Type + Send + Sync + poem_openapi::types::ParseFromJSON + poem_openapi::types::ToJSON> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    #[oai(rename = "perPage")]
    pub per_page: u64,
}

// ── Admin API implementation ──

#[OpenApi(prefix_path = "/admin", tag = "super::ApiTags::Admin")]
impl AdminApi {
    // ── Relying Parties ──

    /// List all relying parties
    #[oai(
        path = "/relying-parties",
        method = "get",
        operation_id = "adminListRelyingParties"
    )]
    async fn list_relying_parties(
        &self,
        state: Data<&AppState>,
        page: Query<Option<u64>>,
        per_page: Query<Option<u64>>,
    ) -> Result<Json<PaginatedResponse<RelyingPartyResponse>>, ApiErrorResponse> {
        let page = page.0.unwrap_or(1).clamp(1, u64::MAX);
        let per_page = per_page.0.unwrap_or(20).clamp(1, 100);
        let paginator = relying_party::Entity::find()
            .order_by_desc(relying_party::Column::CreatedAt)
            .paginate(&state.db, per_page);
        let total = paginator.num_items().await.map_err(to_internal)?;
        let items: Vec<RelyingPartyResponse> = paginator
            .fetch_page(page - 1)
            .await
            .map_err(to_internal)?
            .into_iter()
            .map(Into::into)
            .collect();
        Ok(Json(PaginatedResponse {
            items,
            total,
            page,
            per_page,
        }))
    }

    /// Get a relying party by ID
    #[oai(
        path = "/relying-parties/:id",
        method = "get",
        operation_id = "adminGetRelyingParty"
    )]
    async fn get_relying_party(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> Result<Json<RelyingPartyResponse>, ApiErrorResponse> {
        let rp = relying_party::Entity::find_by_id(&id.0)
            .one(&state.db)
            .await
            .map_err(to_internal)?
            .ok_or_else(|| not_found("Relying party"))?;
        Ok(Json(rp.into()))
    }

    /// Create a new relying party
    #[oai(
        path = "/relying-parties",
        method = "post",
        operation_id = "adminCreateRelyingParty"
    )]
    async fn create_relying_party(
        &self,
        state: Data<&AppState>,
        body: Json<CreateRelyingPartyRequest>,
    ) -> Result<Json<RelyingPartyResponse>, ApiErrorResponse> {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        let model = relying_party::ActiveModel {
            id: Set(id.clone()),
            uuid: Set(body.uuid.clone()),
            name: Set(body.name.clone()),
            logo_url: Set(body.logo_url.clone()),
            website_url: Set(body.website_url.clone()),
            is_active: Set(true),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };
        relying_party::Entity::insert(model)
            .exec(&state.db)
            .await
            .map_err(to_internal)?;
        let rp = relying_party::Entity::find_by_id(&id)
            .one(&state.db)
            .await
            .map_err(to_internal)?
            .ok_or_else(|| not_found("Relying party"))?;
        Ok(Json(rp.into()))
    }

    /// Update a relying party
    #[oai(
        path = "/relying-parties/:id",
        method = "patch",
        operation_id = "adminUpdateRelyingParty"
    )]
    async fn update_relying_party(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
        body: Json<UpdateRelyingPartyRequest>,
    ) -> Result<Json<RelyingPartyResponse>, ApiErrorResponse> {
        let rp = relying_party::Entity::find_by_id(&id.0)
            .one(&state.db)
            .await
            .map_err(to_internal)?
            .ok_or_else(|| not_found("Relying party"))?;
        let mut active: relying_party::ActiveModel = rp.into();
        if let Some(name) = &body.name {
            active.name = Set(name.clone());
        }
        if body.logo_url.is_some() {
            active.logo_url = Set(body.logo_url.clone());
        }
        if body.website_url.is_some() {
            active.website_url = Set(body.website_url.clone());
        }
        if let Some(is_active) = body.is_active {
            active.is_active = Set(is_active);
        }
        active.updated_at = Set(Utc::now().into());
        let updated = active.update(&state.db).await.map_err(to_internal)?;
        Ok(Json(updated.into()))
    }

    /// Delete a relying party
    #[oai(
        path = "/relying-parties/:id",
        method = "delete",
        operation_id = "adminDeleteRelyingParty"
    )]
    async fn delete_relying_party(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> Result<Json<serde_json::Value>, ApiErrorResponse> {
        let result = relying_party::Entity::delete_by_id(&id.0)
            .exec(&state.db)
            .await
            .map_err(to_internal)?;
        if result.rows_affected == 0 {
            return Err(not_found("Relying party"));
        }
        Ok(Json(serde_json::json!({ "deleted": true })))
    }

    // ── Accounts ──

    /// List all accounts
    #[oai(
        path = "/accounts",
        method = "get",
        operation_id = "adminListAccounts"
    )]
    async fn list_accounts(
        &self,
        state: Data<&AppState>,
        page: Query<Option<u64>>,
        per_page: Query<Option<u64>>,
    ) -> Result<Json<PaginatedResponse<AccountResponse>>, ApiErrorResponse> {
        let page = page.0.unwrap_or(1).clamp(1, u64::MAX);
        let per_page = per_page.0.unwrap_or(20).clamp(1, 100);
        let paginator = account::Entity::find()
            .order_by_desc(account::Column::CreatedAt)
            .paginate(&state.db, per_page);
        let total = paginator.num_items().await.map_err(to_internal)?;
        let items: Vec<AccountResponse> = paginator
            .fetch_page(page - 1)
            .await
            .map_err(to_internal)?
            .into_iter()
            .map(Into::into)
            .collect();
        Ok(Json(PaginatedResponse {
            items,
            total,
            page,
            per_page,
        }))
    }

    /// Get an account by ID
    #[oai(
        path = "/accounts/:id",
        method = "get",
        operation_id = "adminGetAccount"
    )]
    async fn get_account(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> Result<Json<AccountResponse>, ApiErrorResponse> {
        let acct = account::Entity::find_by_id(&id.0)
            .one(&state.db)
            .await
            .map_err(to_internal)?
            .ok_or_else(|| not_found("Account"))?;
        Ok(Json(acct.into()))
    }

    /// Update an account (e.g. change status)
    #[oai(
        path = "/accounts/:id",
        method = "patch",
        operation_id = "adminUpdateAccount"
    )]
    async fn update_account(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
        body: Json<UpdateAccountRequest>,
    ) -> Result<Json<AccountResponse>, ApiErrorResponse> {
        let acct = account::Entity::find_by_id(&id.0)
            .one(&state.db)
            .await
            .map_err(to_internal)?
            .ok_or_else(|| not_found("Account"))?;
        let mut active: account::ActiveModel = acct.into();
        if let Some(status) = &body.status {
            active.status = Set(status.clone());
        }
        active.updated_at = Set(Utc::now().into());
        let updated = active.update(&state.db).await.map_err(to_internal)?;
        Ok(Json(updated.into()))
    }

    /// Delete an account
    #[oai(
        path = "/accounts/:id",
        method = "delete",
        operation_id = "adminDeleteAccount"
    )]
    async fn delete_account(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> Result<Json<serde_json::Value>, ApiErrorResponse> {
        let result = account::Entity::delete_by_id(&id.0)
            .exec(&state.db)
            .await
            .map_err(to_internal)?;
        if result.rows_affected == 0 {
            return Err(not_found("Account"));
        }
        Ok(Json(serde_json::json!({ "deleted": true })))
    }

    // ── Sessions ──

    /// List all sessions
    #[oai(
        path = "/sessions",
        method = "get",
        operation_id = "adminListSessions"
    )]
    async fn list_sessions(
        &self,
        state: Data<&AppState>,
        page: Query<Option<u64>>,
        per_page: Query<Option<u64>>,
        /// Filter by state (e.g. RUNNING, COMPLETE)
        #[oai(name = "state")]
        filter_state: Query<Option<String>>,
        /// Filter by relying party ID
        #[oai(name = "relyingPartyId")]
        rp_id: Query<Option<String>>,
    ) -> Result<Json<PaginatedResponse<SessionResponse>>, ApiErrorResponse> {
        let page = page.0.unwrap_or(1).clamp(1, u64::MAX);
        let per_page = per_page.0.unwrap_or(20).clamp(1, 100);
        let mut query = session::Entity::find().order_by_desc(session::Column::CreatedAt);
        if let Some(s) = &filter_state.0 {
            query = query.filter(session::Column::State.eq(s.as_str()));
        }
        if let Some(rp) = &rp_id.0 {
            query = query.filter(session::Column::RelyingPartyId.eq(rp.as_str()));
        }
        let paginator = query.paginate(&state.db, per_page);
        let total = paginator.num_items().await.map_err(to_internal)?;
        let items: Vec<SessionResponse> = paginator
            .fetch_page(page - 1)
            .await
            .map_err(to_internal)?
            .into_iter()
            .map(Into::into)
            .collect();
        Ok(Json(PaginatedResponse {
            items,
            total,
            page,
            per_page,
        }))
    }

    /// Get a session by ID
    #[oai(
        path = "/sessions/:id",
        method = "get",
        operation_id = "adminGetSession"
    )]
    async fn get_session(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> Result<Json<SessionResponse>, ApiErrorResponse> {
        let sess = session::Entity::find_by_id(&id.0)
            .one(&state.db)
            .await
            .map_err(to_internal)?
            .ok_or_else(|| not_found("Session"))?;
        Ok(Json(sess.into()))
    }

    /// Delete a session
    #[oai(
        path = "/sessions/:id",
        method = "delete",
        operation_id = "adminDeleteSession"
    )]
    async fn delete_session(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> Result<Json<serde_json::Value>, ApiErrorResponse> {
        let result = session::Entity::delete_by_id(&id.0)
            .exec(&state.db)
            .await
            .map_err(to_internal)?;
        if result.rows_affected == 0 {
            return Err(not_found("Session"));
        }
        Ok(Json(serde_json::json!({ "deleted": true })))
    }

    // ── Devices ──

    /// List all devices
    #[oai(
        path = "/devices",
        method = "get",
        operation_id = "adminListDevices"
    )]
    async fn list_devices(
        &self,
        state: Data<&AppState>,
        page: Query<Option<u64>>,
        per_page: Query<Option<u64>>,
        /// Filter by account ID
        #[oai(name = "accountId")]
        account_id: Query<Option<String>>,
    ) -> Result<Json<PaginatedResponse<DeviceResponse>>, ApiErrorResponse> {
        let page = page.0.unwrap_or(1).clamp(1, u64::MAX);
        let per_page = per_page.0.unwrap_or(20).clamp(1, 100);
        let mut query = device::Entity::find().order_by_desc(device::Column::CreatedAt);
        if let Some(aid) = &account_id.0 {
            query = query.filter(device::Column::AccountId.eq(aid.as_str()));
        }
        let paginator = query.paginate(&state.db, per_page);
        let total = paginator.num_items().await.map_err(to_internal)?;
        let items: Vec<DeviceResponse> = paginator
            .fetch_page(page - 1)
            .await
            .map_err(to_internal)?
            .into_iter()
            .map(Into::into)
            .collect();
        Ok(Json(PaginatedResponse {
            items,
            total,
            page,
            per_page,
        }))
    }

    /// Get a device by ID
    #[oai(
        path = "/devices/:id",
        method = "get",
        operation_id = "adminGetDevice"
    )]
    async fn get_device(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> Result<Json<DeviceResponse>, ApiErrorResponse> {
        let dev = device::Entity::find_by_id(&id.0)
            .one(&state.db)
            .await
            .map_err(to_internal)?
            .ok_or_else(|| not_found("Device"))?;
        Ok(Json(dev.into()))
    }

    /// Delete a device
    #[oai(
        path = "/devices/:id",
        method = "delete",
        operation_id = "adminDeleteDevice"
    )]
    async fn delete_device(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> Result<Json<serde_json::Value>, ApiErrorResponse> {
        let result = device::Entity::delete_by_id(&id.0)
            .exec(&state.db)
            .await
            .map_err(to_internal)?;
        if result.rows_affected == 0 {
            return Err(not_found("Device"));
        }
        Ok(Json(serde_json::json!({ "deleted": true })))
    }

    // ── Certificates ──

    /// List all certificates
    #[oai(
        path = "/certificates",
        method = "get",
        operation_id = "adminListCertificates"
    )]
    async fn list_certificates(
        &self,
        state: Data<&AppState>,
        page: Query<Option<u64>>,
        per_page: Query<Option<u64>>,
        /// Filter by account ID
        #[oai(name = "accountId")]
        account_id: Query<Option<String>>,
    ) -> Result<Json<PaginatedResponse<CertificateResponse>>, ApiErrorResponse> {
        let page = page.0.unwrap_or(1).clamp(1, u64::MAX);
        let per_page = per_page.0.unwrap_or(20).clamp(1, 100);
        let mut query = certificate::Entity::find().order_by_desc(certificate::Column::CreatedAt);
        if let Some(aid) = &account_id.0 {
            query = query.filter(certificate::Column::AccountId.eq(aid.as_str()));
        }
        let paginator = query.paginate(&state.db, per_page);
        let total = paginator.num_items().await.map_err(to_internal)?;
        let items: Vec<CertificateResponse> = paginator
            .fetch_page(page - 1)
            .await
            .map_err(to_internal)?
            .into_iter()
            .map(Into::into)
            .collect();
        Ok(Json(PaginatedResponse {
            items,
            total,
            page,
            per_page,
        }))
    }

    /// Get a certificate by ID
    #[oai(
        path = "/certificates/:id",
        method = "get",
        operation_id = "adminGetCertificate"
    )]
    async fn get_certificate(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> Result<Json<CertificateResponse>, ApiErrorResponse> {
        let cert = certificate::Entity::find_by_id(&id.0)
            .one(&state.db)
            .await
            .map_err(to_internal)?
            .ok_or_else(|| not_found("Certificate"))?;
        Ok(Json(cert.into()))
    }

    /// Revoke a certificate (set is_active = false)
    #[oai(
        path = "/certificates/:id/revoke",
        method = "post",
        operation_id = "adminRevokeCertificate"
    )]
    async fn revoke_certificate(
        &self,
        state: Data<&AppState>,
        id: Path<String>,
    ) -> Result<Json<CertificateResponse>, ApiErrorResponse> {
        let cert = certificate::Entity::find_by_id(&id.0)
            .one(&state.db)
            .await
            .map_err(to_internal)?
            .ok_or_else(|| not_found("Certificate"))?;
        let mut active: certificate::ActiveModel = cert.into();
        active.is_active = Set(false);
        let updated = active.update(&state.db).await.map_err(to_internal)?;
        Ok(Json(updated.into()))
    }
}

// ── Helpers ──

fn to_internal(e: DbErr) -> ApiErrorResponse {
    ApiErrorResponse::InternalServerError(Json(ProblemDetails::new(
        500,
        "Internal Server Error",
        &e.to_string(),
    )))
}

fn not_found(entity: &str) -> ApiErrorResponse {
    ApiErrorResponse::NotFound(Json(ProblemDetails::new(
        404,
        "Not Found",
        &format!("{entity} not found"),
    )))
}
