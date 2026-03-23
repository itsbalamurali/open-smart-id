use poem_openapi::{Enum, Object};
use serde::{Deserialize, Serialize};

// Re-export types from services
#[allow(unused_imports)]
pub use crate::services::certificate::{
    AuthCertificateLevel, CertificateInfo, CertificateResponse, CertificateState,
    SignCertificateLevel,
};
#[allow(unused_imports)]
pub use crate::services::relying_party::{ApiErrorResponse, ErrorDetail, ProblemDetails};
#[allow(unused_imports)]
pub use crate::services::session::{
    AcspV2Signature, CertificateChoiceSignature, DeviceLinkResponse, FlowType, InteractionType,
    NotificationAuthenticationResponse, NotificationCertificateChoiceResponse,
    NotificationSigningLinkedResponse, NotificationSigningResponse, RawDigestSignature,
    SessionEndResult, SessionResult, SessionResultDetails, SessionSignature,
    SessionSignatureProtocol, SessionState, SessionStatusResponse,
};

// ── Enums ──
// Each enum has as_str() for DB storage and from_str() for reading back.

/// Cryptographic hash algorithm used for signing operations.
#[derive(Debug, Clone, Serialize, Deserialize, Enum, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// SHA-256 hash algorithm.
    #[oai(rename = "SHA-256")]
    Sha256,
    /// SHA-384 hash algorithm.
    #[oai(rename = "SHA-384")]
    Sha384,
    /// SHA-512 hash algorithm.
    #[oai(rename = "SHA-512")]
    Sha512,
    /// SHA3-256 hash algorithm.
    #[oai(rename = "SHA3-256")]
    Sha3256,
    /// SHA3-384 hash algorithm.
    #[oai(rename = "SHA3-384")]
    Sha3384,
    /// SHA3-512 hash algorithm.
    #[oai(rename = "SHA3-512")]
    Sha3512,
}

impl HashAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sha256 => "SHA-256",
            Self::Sha384 => "SHA-384",
            Self::Sha512 => "SHA-512",
            Self::Sha3256 => "SHA3-256",
            Self::Sha3384 => "SHA3-384",
            Self::Sha3512 => "SHA3-512",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "SHA-256" => Some(Self::Sha256),
            "SHA-384" => Some(Self::Sha384),
            "SHA-512" => Some(Self::Sha512),
            "SHA3-256" => Some(Self::Sha3256),
            "SHA3-384" => Some(Self::Sha3384),
            "SHA3-512" => Some(Self::Sha3512),
            _ => None,
        }
    }

    pub fn salt_length(&self) -> i64 {
        match self {
            Self::Sha256 | Self::Sha3256 => 32,
            Self::Sha384 | Self::Sha3384 => 48,
            Self::Sha512 | Self::Sha3512 => 64,
        }
    }
}

/// Signature algorithm used for cryptographic operations.
#[derive(Debug, Clone, Serialize, Deserialize, Enum, PartialEq, Eq)]
pub enum SignatureAlgorithm {
    /// RSA-PSS signature algorithm.
    #[oai(rename = "rsassa-pss")]
    RsassaPss,
    /// SHA-256 with RSA encryption.
    #[oai(rename = "sha256WithRSAEncryption")]
    Sha256WithRsaEncryption,
    /// SHA-384 with RSA encryption.
    #[oai(rename = "sha384WithRSAEncryption")]
    Sha384WithRsaEncryption,
    /// SHA-512 with RSA encryption.
    #[oai(rename = "sha512WithRSAEncryption")]
    Sha512WithRsaEncryption,
}

impl SignatureAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RsassaPss => "rsassa-pss",
            Self::Sha256WithRsaEncryption => "sha256WithRSAEncryption",
            Self::Sha384WithRsaEncryption => "sha384WithRSAEncryption",
            Self::Sha512WithRsaEncryption => "sha512WithRSAEncryption",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "rsassa-pss" => Some(Self::RsassaPss),
            "sha256WithRSAEncryption" => Some(Self::Sha256WithRsaEncryption),
            "sha384WithRSAEncryption" => Some(Self::Sha384WithRsaEncryption),
            "sha512WithRSAEncryption" => Some(Self::Sha512WithRsaEncryption),
            _ => None,
        }
    }
}

/// Mask generation algorithm identifier for RSA-PSS.
#[derive(Debug, Clone, Serialize, Deserialize, Enum, PartialEq, Eq)]
pub enum MaskGenAlgorithmId {
    /// MGF1 mask generation function.
    #[oai(rename = "id-mgf1")]
    IdMgf1,
}

/// Signature protocol for authentication sessions.
#[derive(Debug, Clone, Serialize, Deserialize, Enum, PartialEq, Eq)]
pub enum AuthSignatureProtocol {
    /// ACSP version 2 authentication protocol.
    #[oai(rename = "ACSP_V2")]
    AcspV2,
}

impl AuthSignatureProtocol {
    pub fn as_str(&self) -> &'static str {
        "ACSP_V2"
    }
}

/// Signature protocol for signing sessions.
#[derive(Debug, Clone, Serialize, Deserialize, Enum, PartialEq, Eq)]
pub enum SignSignatureProtocol {
    /// Raw digest signature protocol.
    #[oai(rename = "RAW_DIGEST_SIGNATURE")]
    RawDigestSignature,
}

impl SignSignatureProtocol {
    pub fn as_str(&self) -> &'static str {
        "RAW_DIGEST_SIGNATURE"
    }
}

/// Type of verification code displayed to the user.
#[derive(Debug, Clone, Serialize, Deserialize, Enum, PartialEq, Eq)]
pub enum VerificationCodeType {
    /// 4-digit numeric verification code.
    #[oai(rename = "numeric4")]
    Numeric4,
}

// ── Shared sub-objects ──

/// Signature algorithm parameters sent in a request.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct SignatureAlgorithmParametersInRequest {
    /// Hash algorithm to use with the signature.
    #[oai(rename = "hashAlgorithm")]
    pub hash_algorithm: HashAlgorithm,
}

/// Parameters for the mask generation algorithm.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct MaskGenAlgorithmParameters {
    /// Hash algorithm used by the mask generation function.
    #[oai(rename = "hashAlgorithm")]
    pub hash_algorithm: HashAlgorithm,
}

/// Mask generation algorithm specification for RSA-PSS.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct MaskGenAlgorithm {
    /// Mask generation function identifier.
    pub algorithm: MaskGenAlgorithmId,
    /// Optional parameters for the mask generation function.
    #[oai(skip_serializing_if_is_none)]
    pub parameters: Option<MaskGenAlgorithmParameters>,
}

/// Signature algorithm parameters returned in a response.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct SignatureAlgorithmParametersInResponse {
    /// Hash algorithm used with the signature.
    #[oai(rename = "hashAlgorithm", skip_serializing_if_is_none)]
    pub hash_algorithm: Option<HashAlgorithm>,
    /// Mask generation algorithm (RSA-PSS).
    #[oai(rename = "maskGenAlgorithm", skip_serializing_if_is_none)]
    pub mask_gen_algorithm: Option<MaskGenAlgorithm>,
    /// Salt length in bytes (RSA-PSS).
    #[oai(rename = "saltLength", skip_serializing_if_is_none)]
    pub salt_length: Option<i64>,
    /// Trailer field value (RSA-PSS).
    #[oai(rename = "trailerField", skip_serializing_if_is_none)]
    pub trailer_field: Option<String>,
}

/// Optional request properties controlling session behavior.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct RequestProperties {
    /// Whether to share the mobile device's IP address with the RP.
    #[oai(
        rename = "shareMdClientIpAddress",
        default,
        skip_serializing_if_is_none
    )]
    pub share_md_client_ip_address: Option<bool>,
}

/// Verification code shown to the user for session confirmation.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct VC {
    /// Verification code type.
    #[oai(rename = "type")]
    pub vc_type: VerificationCodeType,
    /// Verification code value.
    pub value: String,
}

// ── Signature protocol parameters (request) ──

/// Parameters for the ACSP_V2 authentication signature protocol.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct AuthSignatureProtocolParameters {
    /// Base64-encoded challenge generated by the relying party.
    #[oai(rename = "rpChallenge")]
    pub rp_challenge: String,
    /// Signature algorithm to use.
    #[oai(rename = "signatureAlgorithm")]
    pub signature_algorithm: SignatureAlgorithm,
    /// Optional signature algorithm parameters.
    #[oai(rename = "signatureAlgorithmParameters", skip_serializing_if_is_none)]
    pub signature_algorithm_parameters: Option<SignatureAlgorithmParametersInRequest>,
}

/// Parameters for the RAW_DIGEST_SIGNATURE signing protocol.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct SignSignatureProtocolParameters {
    /// Base64-encoded hash digest to be signed.
    pub digest: String,
    /// Signature algorithm to use.
    #[oai(rename = "signatureAlgorithm")]
    pub signature_algorithm: SignatureAlgorithm,
    /// Optional signature algorithm parameters.
    #[oai(rename = "signatureAlgorithmParameters", skip_serializing_if_is_none)]
    pub signature_algorithm_parameters: Option<SignatureAlgorithmParametersInRequest>,
}

// ── Request bodies ──

/// Request body for device-link based authentication.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct DeviceLinkAuthenticationRequest {
    /// UUID of the relying party.
    #[oai(rename = "relyingPartyUUID")]
    pub relying_party_uuid: String,
    /// Display name of the relying party.
    #[oai(rename = "relyingPartyName")]
    pub relying_party_name: String,
    /// URL called when the device link is scanned.
    #[oai(rename = "initialCallbackUrl")]
    pub initial_callback_url: String,
    /// Requested certificate level.
    #[oai(rename = "certificateLevel", skip_serializing_if_is_none)]
    pub certificate_level: Option<AuthCertificateLevel>,
    /// Signature protocol to use.
    #[oai(rename = "signatureProtocol")]
    pub signature_protocol: AuthSignatureProtocol,
    /// Signature protocol parameters.
    #[oai(rename = "signatureProtocolParameters")]
    pub signature_protocol_parameters: AuthSignatureProtocolParameters,
    /// JSON-encoded interaction definitions.
    pub interactions: String,
    /// Optional request properties.
    #[oai(rename = "requestProperties", skip_serializing_if_is_none)]
    pub request_properties: Option<RequestProperties>,
    /// Optional client capability list.
    #[oai(skip_serializing_if_is_none)]
    pub capabilities: Option<Vec<String>>,
}

/// Request body for notification-based authentication.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct NotificationAuthenticationRequest {
    /// UUID of the relying party.
    #[oai(rename = "relyingPartyUUID")]
    pub relying_party_uuid: String,
    /// Display name of the relying party.
    #[oai(rename = "relyingPartyName")]
    pub relying_party_name: String,
    /// Requested certificate level.
    #[oai(rename = "certificateLevel", skip_serializing_if_is_none)]
    pub certificate_level: Option<AuthCertificateLevel>,
    /// Signature protocol to use.
    #[oai(rename = "signatureProtocol")]
    pub signature_protocol: AuthSignatureProtocol,
    /// Signature protocol parameters.
    #[oai(rename = "signatureProtocolParameters")]
    pub signature_protocol_parameters: AuthSignatureProtocolParameters,
    /// JSON-encoded interaction definitions.
    pub interactions: String,
    /// Optional request properties.
    #[oai(rename = "requestProperties", skip_serializing_if_is_none)]
    pub request_properties: Option<RequestProperties>,
    /// Optional client capability list.
    #[oai(skip_serializing_if_is_none)]
    pub capabilities: Option<Vec<String>>,
    /// Verification code type for user confirmation.
    #[oai(rename = "vcType")]
    pub vc_type: VerificationCodeType,
}

/// Request body for device-link based signing.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct DeviceLinkSigningRequest {
    /// UUID of the relying party.
    #[oai(rename = "relyingPartyUUID")]
    pub relying_party_uuid: String,
    /// Display name of the relying party.
    #[oai(rename = "relyingPartyName")]
    pub relying_party_name: String,
    /// URL called when the device link is scanned.
    #[oai(rename = "initialCallbackUrl", skip_serializing_if_is_none)]
    pub initial_callback_url: Option<String>,
    /// Requested certificate level.
    #[oai(rename = "certificateLevel", skip_serializing_if_is_none)]
    pub certificate_level: Option<SignCertificateLevel>,
    /// Signature protocol to use.
    #[oai(rename = "signatureProtocol")]
    pub signature_protocol: SignSignatureProtocol,
    /// Signature protocol parameters.
    #[oai(rename = "signatureProtocolParameters")]
    pub signature_protocol_parameters: SignSignatureProtocolParameters,
    /// Optional nonce for replay protection.
    #[oai(skip_serializing_if_is_none)]
    pub nonce: Option<String>,
    /// JSON-encoded interaction definitions.
    pub interactions: String,
    /// Optional request properties.
    #[oai(rename = "requestProperties", skip_serializing_if_is_none)]
    pub request_properties: Option<RequestProperties>,
    /// Optional client capability list.
    #[oai(skip_serializing_if_is_none)]
    pub capabilities: Option<Vec<String>>,
}

/// Request body for notification-based signing.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct NotificationSigningRequest {
    /// UUID of the relying party.
    #[oai(rename = "relyingPartyUUID")]
    pub relying_party_uuid: String,
    /// Display name of the relying party.
    #[oai(rename = "relyingPartyName")]
    pub relying_party_name: String,
    /// Requested certificate level.
    #[oai(rename = "certificateLevel", skip_serializing_if_is_none)]
    pub certificate_level: Option<SignCertificateLevel>,
    /// Signature protocol to use.
    #[oai(rename = "signatureProtocol")]
    pub signature_protocol: SignSignatureProtocol,
    /// Signature protocol parameters.
    #[oai(rename = "signatureProtocolParameters")]
    pub signature_protocol_parameters: SignSignatureProtocolParameters,
    /// Optional nonce for replay protection.
    #[oai(skip_serializing_if_is_none)]
    pub nonce: Option<String>,
    /// JSON-encoded interaction definitions.
    pub interactions: String,
    /// Optional request properties.
    #[oai(rename = "requestProperties", skip_serializing_if_is_none)]
    pub request_properties: Option<RequestProperties>,
    /// Optional client capability list.
    #[oai(skip_serializing_if_is_none)]
    pub capabilities: Option<Vec<String>>,
}

/// Request body for notification-based signing linked to an existing session.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct NotificationSigningLinkedRequest {
    /// UUID of the relying party.
    #[oai(rename = "relyingPartyUUID")]
    pub relying_party_uuid: String,
    /// Display name of the relying party.
    #[oai(rename = "relyingPartyName")]
    pub relying_party_name: String,
    /// Requested certificate level.
    #[oai(rename = "certificateLevel", skip_serializing_if_is_none)]
    pub certificate_level: Option<SignCertificateLevel>,
    /// Signature protocol to use.
    #[oai(rename = "signatureProtocol")]
    pub signature_protocol: SignSignatureProtocol,
    /// Signature protocol parameters.
    #[oai(rename = "signatureProtocolParameters")]
    pub signature_protocol_parameters: SignSignatureProtocolParameters,
    /// ID of the previous session to link to.
    #[oai(rename = "linkedSessionID")]
    pub linked_session_id: String,
    /// Optional nonce for replay protection.
    #[oai(skip_serializing_if_is_none)]
    pub nonce: Option<String>,
    /// JSON-encoded interaction definitions.
    pub interactions: String,
    /// Optional request properties.
    #[oai(rename = "requestProperties", skip_serializing_if_is_none)]
    pub request_properties: Option<RequestProperties>,
    /// Optional client capability list.
    #[oai(skip_serializing_if_is_none)]
    pub capabilities: Option<Vec<String>>,
}

/// Request body for device-link based certificate choice.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct DeviceLinkCertificateChoiceRequest {
    /// UUID of the relying party.
    #[oai(rename = "relyingPartyUUID")]
    pub relying_party_uuid: String,
    /// Display name of the relying party.
    #[oai(rename = "relyingPartyName")]
    pub relying_party_name: String,
    /// URL called when the device link is scanned.
    #[oai(rename = "initialCallbackUrl", skip_serializing_if_is_none)]
    pub initial_callback_url: Option<String>,
    /// Requested certificate level.
    #[oai(rename = "certificateLevel", skip_serializing_if_is_none)]
    pub certificate_level: Option<SignCertificateLevel>,
    /// Optional nonce for replay protection.
    #[oai(skip_serializing_if_is_none)]
    pub nonce: Option<String>,
    /// Optional request properties.
    #[oai(rename = "requestProperties", skip_serializing_if_is_none)]
    pub request_properties: Option<RequestProperties>,
    /// Optional client capability list.
    #[oai(skip_serializing_if_is_none)]
    pub capabilities: Option<Vec<String>>,
}

/// Request body for notification-based certificate choice.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct NotificationCertificateChoiceRequest {
    /// UUID of the relying party.
    #[oai(rename = "relyingPartyUUID")]
    pub relying_party_uuid: String,
    /// Display name of the relying party.
    #[oai(rename = "relyingPartyName")]
    pub relying_party_name: String,
    /// Requested certificate level.
    #[oai(rename = "certificateLevel", skip_serializing_if_is_none)]
    pub certificate_level: Option<SignCertificateLevel>,
    /// Optional nonce for replay protection.
    #[oai(skip_serializing_if_is_none)]
    pub nonce: Option<String>,
    /// Optional request properties.
    #[oai(rename = "requestProperties", skip_serializing_if_is_none)]
    pub request_properties: Option<RequestProperties>,
    /// Optional client capability list.
    #[oai(skip_serializing_if_is_none)]
    pub capabilities: Option<Vec<String>>,
}

/// Request body for retrieving a signing certificate.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct SigningCertificateRequest {
    /// UUID of the relying party.
    #[oai(rename = "relyingPartyUUID")]
    pub relying_party_uuid: String,
    /// Display name of the relying party.
    #[oai(rename = "relyingPartyName")]
    pub relying_party_name: String,
    /// Requested certificate level.
    #[oai(rename = "certificateLevel", skip_serializing_if_is_none)]
    pub certificate_level: Option<SignCertificateLevel>,
}
