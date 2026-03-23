use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chrono::{Duration, Utc};
use poem_openapi::{Enum, Object};
use rcgen::{
    BasicConstraints, CertificateParams, CertifiedIssuer, DnType, ExtendedKeyUsagePurpose, IsCa,
    KeyPair, KeyUsagePurpose,
};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::entities::certificate;
use crate::services::relying_party::ServiceError;

/// State of a certificate lookup result.
#[derive(Debug, Clone, Serialize, Deserialize, Enum, PartialEq, Eq)]
pub enum CertificateState {
    /// A valid certificate was found.
    #[oai(rename = "OK")]
    Ok,
    /// The user's document is unusable and no certificate is available.
    #[oai(rename = "DOCUMENT_UNUSABLE")]
    DocumentUnusable,
}

/// Requested certificate level for authentication.
#[derive(Debug, Clone, Serialize, Deserialize, Enum, PartialEq, Eq)]
pub enum AuthCertificateLevel {
    /// Advanced electronic signature level.
    #[oai(rename = "ADVANCED")]
    Advanced,
    /// Qualified electronic signature level.
    #[oai(rename = "QUALIFIED")]
    Qualified,
}

impl AuthCertificateLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Advanced => "ADVANCED",
            Self::Qualified => "QUALIFIED",
        }
    }
}

/// Requested certificate level for signing.
#[derive(Debug, Clone, Serialize, Deserialize, Enum, PartialEq, Eq)]
pub enum SignCertificateLevel {
    /// Advanced electronic signature level.
    #[oai(rename = "ADVANCED")]
    Advanced,
    /// Qualified electronic signature level.
    #[oai(rename = "QUALIFIED")]
    Qualified,
    /// Qualified signature creation device level.
    #[oai(rename = "QSCD")]
    Qscd,
}

impl SignCertificateLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Advanced => "ADVANCED",
            Self::Qualified => "QUALIFIED",
            Self::Qscd => "QSCD",
        }
    }
}

/// Certificate information returned in API responses.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct CertificateInfo {
    /// Base64-encoded DER certificate.
    pub value: String,
    /// Certificate level (e.g. "QUALIFIED").
    #[oai(rename = "certificateLevel")]
    pub certificate_level: String,
}

/// Response for certificate retrieval requests.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct CertificateResponse {
    /// Certificate lookup state.
    pub state: CertificateState,
    /// Certificate data (present when state is OK).
    #[oai(skip_serializing_if_is_none)]
    pub cert: Option<CertificateInfo>,
}

// OID 2.5.4.5 = id-at-serialNumber
const DN_SERIAL_NUMBER: &[u64] = &[2, 5, 4, 5];

#[derive(Clone)]
pub struct CertificateService {
    ca: Arc<CertifiedIssuer<'static, KeyPair>>,
}

impl CertificateService {
    pub fn new() -> Result<Self, rcgen::Error> {
        let ca_key_pair = KeyPair::generate()?;

        let mut ca_params = CertificateParams::new(Vec::<String>::new())?;
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params
            .distinguished_name
            .push(DnType::OrganizationName, "SmartID CA");
        ca_params
            .distinguished_name
            .push(DnType::CommonName, "SmartID Root CA");
        ca_params.key_usages.push(KeyUsagePurpose::KeyCertSign);
        ca_params.key_usages.push(KeyUsagePurpose::CrlSign);

        let ca = CertifiedIssuer::self_signed(ca_params, ca_key_pair)?;
        Ok(Self { ca: Arc::new(ca) })
    }

    pub fn ca_pem(&self) -> String {
        self.ca.as_ref().pem()
    }

    /// Find active auth cert for account, or issue a new one and persist.
    pub async fn get_or_issue_auth_cert(
        &self,
        db: &DatabaseConnection,
        account_id: &str,
        serial_number: &str,
    ) -> Result<certificate::Model, ServiceError> {
        if let Some(cert) = Self::find_active(db, account_id, "authentication").await? {
            return Ok(cert);
        }
        self.issue_and_persist(db, account_id, serial_number, CertPurpose::Authentication)
            .await
    }

    /// Find active signing cert for account, or issue a new one and persist.
    pub async fn get_or_issue_signing_cert(
        &self,
        db: &DatabaseConnection,
        account_id: &str,
        serial_number: &str,
    ) -> Result<certificate::Model, ServiceError> {
        if let Some(cert) = Self::find_active(db, account_id, "signing").await? {
            return Ok(cert);
        }
        self.issue_and_persist(db, account_id, serial_number, CertPurpose::Signing)
            .await
    }

    async fn find_active(
        db: &DatabaseConnection,
        account_id: &str,
        cert_type: &str,
    ) -> Result<Option<certificate::Model>, ServiceError> {
        let now = Utc::now();
        certificate::Entity::find()
            .filter(certificate::Column::AccountId.eq(account_id))
            .filter(certificate::Column::CertType.eq(cert_type))
            .filter(certificate::Column::IsActive.eq(true))
            .filter(certificate::Column::ExpiresAt.gt(now))
            .one(db)
            .await
            .map_err(ServiceError::Db)
    }

    async fn issue_and_persist(
        &self,
        db: &DatabaseConnection,
        account_id: &str,
        serial_number: &str,
        purpose: CertPurpose,
    ) -> Result<certificate::Model, ServiceError> {
        let issued = self
            .issue_certificate(serial_number, &purpose)
            .map_err(|e| ServiceError::Forbidden(format!("certificate generation failed: {e}")))?;

        let now = Utc::now();
        let expires = now + Duration::days(365 * 3);
        let id = uuid::Uuid::new_v4().to_string();

        let cert_type = match purpose {
            CertPurpose::Authentication => "authentication",
            CertPurpose::Signing => "signing",
        };

        let model = certificate::ActiveModel {
            id: Set(id.clone()),
            account_id: Set(account_id.to_string()),
            cert_type: Set(cert_type.to_string()),
            cert_value: Set(issued.cert_der_base64),
            cert_level: Set(issued.certificate_level),
            key_pair_pem: Set(issued.key_pair_pem),
            is_active: Set(true),
            created_at: Set(now.into()),
            expires_at: Set(expires.into()),
        };

        certificate::Entity::insert(model)
            .exec(db)
            .await
            .map_err(ServiceError::Db)?;

        certificate::Entity::find_by_id(&id)
            .one(db)
            .await
            .map_err(ServiceError::Db)?
            .ok_or(ServiceError::NotFound("certificate".to_string()))
    }

    fn issue_certificate(
        &self,
        serial_number: &str,
        purpose: &CertPurpose,
    ) -> Result<IssuedCertificate, rcgen::Error> {
        let ee_key_pair = KeyPair::generate()?;

        let mut params = CertificateParams::new(Vec::<String>::new())?;
        params.is_ca = IsCa::NoCa;
        params
            .distinguished_name
            .push(DnType::CommonName, serial_number);
        params
            .distinguished_name
            .push(DnType::OrganizationName, "SmartID");
        params.distinguished_name.push(
            DnType::CustomDnType(DN_SERIAL_NUMBER.to_vec()),
            serial_number,
        );

        match purpose {
            CertPurpose::Authentication => {
                params.key_usages.push(KeyUsagePurpose::DigitalSignature);
                params
                    .extended_key_usages
                    .push(ExtendedKeyUsagePurpose::ClientAuth);
            }
            CertPurpose::Signing => {
                params.key_usages.push(KeyUsagePurpose::DigitalSignature);
                params.key_usages.push(KeyUsagePurpose::ContentCommitment);
            }
        }

        let cert = params.signed_by(&ee_key_pair, &*self.ca)?;

        Ok(IssuedCertificate {
            cert_der_base64: BASE64.encode(cert.der()),
            key_pair_pem: ee_key_pair.serialize_pem(),
            certificate_level: "QUALIFIED".to_string(),
        })
    }
}

struct IssuedCertificate {
    cert_der_base64: String,
    key_pair_pem: String,
    certificate_level: String,
}

enum CertPurpose {
    Authentication,
    Signing,
}
