//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(schema_name = "ccs_schema", table_name = "consultant_rating")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub consultant_rating_id: i64,
    #[sea_orm(unique)]
    pub consultation_id: i64,
    pub rating: Option<i16>,
    pub rated_at: Option<DateTimeWithTimeZone>,
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
