//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.4

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(schema_name = "ccs_schema", table_name = "user_rating")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_rating_id: i64,
    pub user_account_id: i64,
    pub consultant_id: i64,
    pub meeting_at: DateTimeWithTimeZone,
    #[sea_orm(column_type = "Text", unique)]
    pub charge_id: String,
    pub rating: Option<i16>,
    pub rated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
