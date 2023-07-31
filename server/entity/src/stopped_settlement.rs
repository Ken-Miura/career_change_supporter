//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(schema_name = "ccs_schema", table_name = "stopped_settlement")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub stopped_settlement_id: i64,
    #[sea_orm(unique)]
    pub consultation_id: i64,
    #[sea_orm(column_type = "Text", unique)]
    pub charge_id: String,
    pub fee_per_hour_in_yen: i32,
    #[sea_orm(column_type = "Text")]
    pub platform_fee_rate_in_percentage: String,
    pub credit_facilities_expired_at: DateTimeWithTimeZone,
    pub stopped_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::consultation::Entity",
        from = "Column::ConsultationId",
        to = "super::consultation::Column::ConsultationId"
    )]
    Consultation,
}

impl Related<super::consultation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Consultation.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
