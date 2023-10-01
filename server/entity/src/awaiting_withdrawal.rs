//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(schema_name = "ccs_schema", table_name = "awaiting_withdrawal")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub consultation_id: i64,
    pub fee_per_hour_in_yen: i32,
    pub payment_confirmed_by: String,
    pub created_at: DateTimeWithTimeZone,
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
