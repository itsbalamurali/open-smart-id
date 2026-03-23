use chrono::Utc;
use sea_orm::*;

use crate::db::entities::account;
use crate::services::relying_party::ServiceError;

pub struct AccountService;

impl AccountService {
    /// Parse ETSI semantic ID (e.g. "PNOEE-48010010101") and find or create account.
    pub async fn find_or_create_by_semantic_id(
        db: &DatabaseConnection,
        semantic_id: &str,
    ) -> Result<account::Model, ServiceError> {
        if let Some(acct) = account::Entity::find()
            .filter(account::Column::SemanticId.eq(semantic_id))
            .one(db)
            .await
            .map_err(ServiceError::Db)?
        {
            return Ok(acct);
        }

        let (identity_type, country_code, national_id) = parse_etsi_semantic_id(semantic_id)?;
        let document_number = generate_document_number(semantic_id);
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();

        let model = account::ActiveModel {
            id: Set(id.clone()),
            semantic_id: Set(Some(semantic_id.to_string())),
            document_number: Set(document_number),
            identity_type: Set(Some(identity_type)),
            country_code: Set(Some(country_code)),
            national_identity_number: Set(Some(national_id)),
            status: Set("active".to_string()),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };

        account::Entity::insert(model)
            .exec(db)
            .await
            .map_err(ServiceError::Db)?;

        account::Entity::find_by_id(&id)
            .one(db)
            .await
            .map_err(ServiceError::Db)?
            .ok_or(ServiceError::NotFound("account".to_string()))
    }

    /// Find account by document number or return 404.
    pub async fn find_by_document_number(
        db: &DatabaseConnection,
        document_number: &str,
    ) -> Result<account::Model, ServiceError> {
        account::Entity::find()
            .filter(account::Column::DocumentNumber.eq(document_number))
            .one(db)
            .await
            .map_err(ServiceError::Db)?
            .ok_or(ServiceError::NotFound(format!(
                "account with document number '{document_number}' not found"
            )))
    }

    /// Create an anonymous account (no semantic ID).
    pub async fn create_anonymous(db: &DatabaseConnection) -> Result<account::Model, ServiceError> {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        let doc_num = format!("ANON-{}", &id[..18]);

        let model = account::ActiveModel {
            id: Set(id.clone()),
            semantic_id: Set(None),
            document_number: Set(doc_num),
            identity_type: Set(None),
            country_code: Set(None),
            national_identity_number: Set(None),
            status: Set("active".to_string()),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };

        account::Entity::insert(model)
            .exec(db)
            .await
            .map_err(ServiceError::Db)?;

        account::Entity::find_by_id(&id)
            .one(db)
            .await
            .map_err(ServiceError::Db)?
            .ok_or(ServiceError::NotFound("account".to_string()))
    }
}

/// Parse "PNOEE-48010010101" → ("PNO", "EE", "48010010101")
fn parse_etsi_semantic_id(id: &str) -> Result<(String, String, String), ServiceError> {
    if id.len() < 6 {
        return Err(ServiceError::NotFound(format!(
            "invalid ETSI semantic ID: '{id}'"
        )));
    }

    let identity_type = id[..3].to_string();
    let country_code = id[3..5].to_string();

    if id.as_bytes().get(5) != Some(&b'-') {
        return Err(ServiceError::NotFound(format!(
            "invalid ETSI semantic ID format: '{id}'"
        )));
    }

    let national_id = id[6..].to_string();

    if !matches!(identity_type.as_str(), "PAS" | "IDC" | "PNO") {
        return Err(ServiceError::NotFound(format!(
            "unsupported identity type: '{identity_type}'"
        )));
    }

    Ok((identity_type, country_code, national_id))
}

/// Generate a document number from a semantic ID.
/// Format: {SEMANTIC_ID}-{SHORT_HASH}
fn generate_document_number(semantic_id: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    semantic_id.hash(&mut hasher);
    uuid::Uuid::new_v4().to_string().hash(&mut hasher);
    let hash = hasher.finish();

    let suffix: String = format!("{:X}", hash)
        .chars()
        .collect::<Vec<_>>()
        .chunks(4)
        .take(2)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("-");

    format!("{semantic_id}-{suffix}")
}
