use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "roles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]
    pub name: String,
    pub description: Option<String>,
    /// JSON array of permission enum values, e.g. ["admin_users:read","roles:write"]
    #[sea_orm(column_type = "Text")]
    pub permissions: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::admin_user_role::Entity")]
    AdminUserRoles,
}

impl Related<super::admin_user_role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AdminUserRoles.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
