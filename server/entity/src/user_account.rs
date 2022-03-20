//! SeaORM Entity. Generated by sea-orm-codegen 0.6.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_account", schema_name = "ccs_schema")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_account_id: i64,
    #[sea_orm(unique)]
    pub email_address: String,
    pub hashed_password: Vec<u8>,
    pub last_login_time: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub disabled_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::identity_info::Entity")]
    IdentityInfo,
    #[sea_orm(has_many = "super::career_info::Entity")]
    CareerInfo,
    #[sea_orm(has_many = "super::consulting_fee::Entity")]
    ConsultingFee,
    #[sea_orm(has_many = "super::tenant::Entity")]
    Tenant,
}

impl Related<super::identity_info::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::IdentityInfo.def()
    }
}

impl Related<super::career_info::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CareerInfo.def()
    }
}

impl Related<super::consulting_fee::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ConsultingFee.def()
    }
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
