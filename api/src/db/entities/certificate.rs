use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "certificates")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub account_id: String,
    /// "authentication" or "signing"
    pub cert_type: String,
    /// DER-encoded certificate, Base64-encoded
    #[sea_orm(column_type = "Text")]
    pub cert_value: String,
    /// QUALIFIED, ADVANCED, or QSCD
    pub cert_level: String,
    /// PEM-encoded private key (server-side only)
    #[sea_orm(column_type = "Text")]
    pub key_pair_pem: String,
    pub is_active: bool,
    pub created_at: DateTimeUtc,
    pub expires_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id"
    )]
    Account,
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
