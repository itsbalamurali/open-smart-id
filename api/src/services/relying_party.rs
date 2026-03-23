use chrono::Utc;
use poem_openapi::{ApiResponse, Object, payload::Json};
use sea_orm::*;
use serde::{Deserialize, Serialize};

use crate::db::entities::relying_party;

pub struct RelyingPartyService;

impl RelyingPartyService {
    pub async fn find_or_create(
        db: &DatabaseConnection,
        rp_uuid: &str,
        rp_name: &str,
    ) -> Result<relying_party::Model, DbErr> {
        if let Some(rp) = relying_party::Entity::find()
            .filter(relying_party::Column::Uuid.eq(rp_uuid))
            .one(db)
            .await?
        {
            return Ok(rp);
        }

        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();

        let model = relying_party::ActiveModel {
            id: Set(id.clone()),
            uuid: Set(rp_uuid.to_string()),
            name: Set(rp_name.to_string()),
            logo_url: Set(None),
            website_url: Set(None),
            is_active: Set(true),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };

        relying_party::Entity::insert(model).exec(db).await?;

        relying_party::Entity::find_by_id(&id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("relying_party".to_string()))
    }

    pub async fn update(
        db: &DatabaseConnection,
        rp_uuid: &str,
        logo_url: Option<String>,
        website_url: Option<String>,
    ) -> Result<relying_party::Model, ServiceError> {
        let rp = relying_party::Entity::find()
            .filter(relying_party::Column::Uuid.eq(rp_uuid))
            .one(db)
            .await
            .map_err(ServiceError::Db)?
            .ok_or_else(|| ServiceError::NotFound("Relying party not found".into()))?;

        let mut active: relying_party::ActiveModel = rp.into();
        if logo_url.is_some() {
            active.logo_url = Set(logo_url);
        }
        if website_url.is_some() {
            active.website_url = Set(website_url);
        }
        active.updated_at = Set(Utc::now().into());

        active.update(db).await.map_err(ServiceError::Db)
    }

    pub async fn validate(
        db: &DatabaseConnection,
        rp_uuid: &str,
        rp_name: &str,
    ) -> Result<relying_party::Model, ServiceError> {
        let rp = Self::find_or_create(db, rp_uuid, rp_name)
            .await
            .map_err(ServiceError::Db)?;

        if !rp.is_active {
            return Err(ServiceError::Forbidden("Relying party is inactive".into()));
        }

        Ok(rp)
    }
}

#[derive(Debug)]
pub enum ServiceError {
    Db(DbErr),
    Forbidden(String),
    NotFound(String),
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Db(e) => write!(f, "database error: {e}"),
            Self::Forbidden(msg) => write!(f, "forbidden: {msg}"),
            Self::NotFound(msg) => write!(f, "not found: {msg}"),
        }
    }
}

impl std::error::Error for ServiceError {}

/// Individual error detail within a problem response.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct ErrorDetail {
    /// Machine-readable error code.
    #[oai(skip_serializing_if_is_none)]
    pub code: Option<String>,
    /// Human-readable error description.
    #[oai(skip_serializing_if_is_none)]
    pub detail: Option<String>,
    /// Name of the request parameter that caused the error.
    #[oai(rename = "paramName", skip_serializing_if_is_none)]
    pub param_name: Option<String>,
    /// JSON Pointer to the problematic field.
    #[oai(skip_serializing_if_is_none)]
    pub pointer: Option<String>,
}

/// RFC 9457 Problem Details error response body.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct ProblemDetails {
    /// Problem type URI.
    #[oai(rename = "type")]
    pub problem_type: String,
    /// HTTP status code.
    pub status: i32,
    /// Short human-readable summary.
    pub title: String,
    /// Detailed human-readable explanation.
    pub detail: String,
    /// URI identifying the specific occurrence.
    #[oai(skip_serializing_if_is_none)]
    pub instance: Option<String>,
    /// List of individual error details.
    #[oai(skip_serializing_if_is_none)]
    pub errors: Option<Vec<ErrorDetail>>,
}

impl ProblemDetails {
    pub fn new(status: i32, title: &str, detail: &str) -> Self {
        Self {
            problem_type: "about:blank".to_string(),
            status,
            title: title.to_string(),
            detail: detail.to_string(),
            instance: None,
            errors: None,
        }
    }
}

#[derive(ApiResponse)]
pub enum ApiErrorResponse {
    /// Bad request
    #[oai(status = 400)]
    BadRequest(Json<ProblemDetails>),
    /// Unauthorized
    #[oai(status = 401)]
    Unauthorized(Json<ProblemDetails>),
    /// Forbidden
    #[oai(status = 403)]
    Forbidden(Json<ProblemDetails>),
    /// Not found
    #[oai(status = 404)]
    NotFound(Json<ProblemDetails>),
    /// Client too old
    #[oai(status = 480)]
    ClientTooOld(Json<ProblemDetails>),
    /// Internal server error
    #[oai(status = 500)]
    InternalServerError(Json<ProblemDetails>),
    /// System under maintenance
    #[oai(status = 580)]
    SystemUnderMaintenance(Json<ProblemDetails>),
}

impl From<ServiceError> for ApiErrorResponse {
    fn from(e: ServiceError) -> Self {
        match e {
            ServiceError::Db(db_err) => Self::InternalServerError(Json(ProblemDetails::new(
                500,
                "Internal Server Error",
                &db_err.to_string(),
            ))),
            ServiceError::Forbidden(msg) => {
                Self::Forbidden(Json(ProblemDetails::new(403, "Forbidden", &msg)))
            }
            ServiceError::NotFound(msg) => {
                Self::NotFound(Json(ProblemDetails::new(404, "Not Found", &msg)))
            }
        }
    }
}
