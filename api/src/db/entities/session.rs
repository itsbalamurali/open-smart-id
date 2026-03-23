use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub relying_party_id: String,
    pub account_id: Option<String>,

    // ── Session type & state ──
    /// "authentication", "signing", or "certificate_choice"
    pub kind: String,
    /// "RUNNING" or "COMPLETE"
    pub state: String,

    // ── Device link fields (populated for device-link flows) ──
    pub session_token: Option<String>,
    pub session_secret: Option<String>,
    pub device_link_base: Option<String>,

    // ── Request parameters ──
    /// "ACSP_V2" or "RAW_DIGEST_SIGNATURE"
    pub signature_protocol: Option<String>,
    /// "rsassa-pss", "sha256WithRSAEncryption", etc.
    pub signature_algorithm: Option<String>,
    /// Hash algorithm from signatureAlgorithmParameters
    pub hash_algorithm: Option<String>,
    /// "QUALIFIED", "ADVANCED", or "QSCD"
    pub certificate_level: Option<String>,
    /// Base64-encoded rpChallenge (auth) or digest (signing)
    #[sea_orm(column_type = "Text")]
    pub challenge_or_digest: Option<String>,
    /// Base64-encoded interactions JSON
    #[sea_orm(column_type = "Text")]
    pub interactions: Option<String>,
    /// Nonce for idempotent behavior
    pub nonce: Option<String>,
    /// Callback URL for device-link flows
    pub initial_callback_url: Option<String>,
    /// Linked session ID for notification/linked signing
    pub linked_session_id: Option<String>,
    /// "QR", "App2App", "Web2App", or "Notification"
    pub flow_type: Option<String>,

    // ── Verification code (notification signing) ──
    /// "numeric4"
    pub vc_type: Option<String>,
    /// 4-digit verification code value
    pub vc_value: Option<String>,

    // ── Request properties ──
    pub share_md_client_ip_address: bool,

    // ── Result fields (populated on completion) ──
    /// "OK", "USER_REFUSED", "TIMEOUT", etc.
    pub end_result: Option<String>,
    pub document_number: Option<String>,
    /// JSON-serialized signature object
    #[sea_orm(column_type = "Text")]
    pub signature_json: Option<String>,
    /// Server-generated random (ACSP_V2)
    pub server_random: Option<String>,
    /// User challenge from mobile device (ACSP_V2)
    pub user_challenge: Option<String>,
    /// Base64-encoded DER certificate
    #[sea_orm(column_type = "Text")]
    pub cert_value: Option<String>,
    pub cert_level: Option<String>,
    /// "displayTextAndPIN", "confirmationMessage", etc.
    pub interaction_type_used: Option<String>,
    pub device_ip_address: Option<String>,
    /// JSON array of ignored request properties
    #[sea_orm(column_type = "Text")]
    pub ignored_properties_json: Option<String>,

    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::relying_party::Entity",
        from = "Column::RelyingPartyId",
        to = "super::relying_party::Column::Id"
    )]
    RelyingParty,
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id"
    )]
    Account,
}

impl Related<super::relying_party::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RelyingParty.def()
    }
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
