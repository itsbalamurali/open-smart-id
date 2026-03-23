use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// ETSI Natural Person Semantics Identifier (e.g. PNOEE-48010010101)
    #[sea_orm(unique)]
    pub semantic_id: Option<String>,
    /// Smart-ID document number (e.g. PNOEE-30001010004-K2GN-NQ)
    #[sea_orm(unique)]
    pub document_number: String,
    /// Identity type: PAS, IDC, PNO
    pub identity_type: Option<String>,
    /// ISO 3166-1 alpha-2 country code
    pub country_code: Option<String>,
    /// National identity number
    pub national_identity_number: Option<String>,
    pub status: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::certificate::Entity")]
    Certificates,
    #[sea_orm(has_many = "super::session::Entity")]
    Sessions,
    #[sea_orm(has_many = "super::device::Entity")]
    Devices,
}

impl Related<super::certificate::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Certificates.def()
    }
}

impl Related<super::session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sessions.def()
    }
}

impl Related<super::device::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Devices.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
