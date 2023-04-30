//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(schema_name = "ccs_schema", table_name = "consultation")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub consultation_id: i64,
    pub user_account_id: i64,
    pub consultant_id: i64,
    pub meeting_at: DateTimeWithTimeZone,
    #[sea_orm(unique)]
    pub room_name: String,
    pub user_account_entered_at: Option<DateTimeWithTimeZone>,
    pub consultant_entered_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::user_rating::Entity")]
    UserRating,
    #[sea_orm(has_one = "super::consultant_rating::Entity")]
    ConsultantRating,
    #[sea_orm(has_one = "super::settlement::Entity")]
    Settlement,
    #[sea_orm(has_one = "super::stopped_settlement::Entity")]
    StoppedSettlement,
    #[sea_orm(has_one = "super::receipt::Entity")]
    Receipt,
    #[sea_orm(has_one = "super::refund::Entity")]
    Refund,
}

impl Related<super::user_rating::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserRating.def()
    }
}

impl Related<super::consultant_rating::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ConsultantRating.def()
    }
}

impl Related<super::settlement::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Settlement.def()
    }
}

impl Related<super::stopped_settlement::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StoppedSettlement.def()
    }
}

impl Related<super::receipt::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Receipt.def()
    }
}

impl Related<super::refund::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Refund.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
