//! SeaORM Entity. Generated by sea-orm-codegen 0.9.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(schema_name = "ccs_schema", table_name = "user_rating")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_rating_id: i64,
    pub user_account_id: i64,
    pub consultant_id: i64,
    #[sea_orm(column_type = "Text")]
    pub charge_id: String,
    pub consultation_date_time: DateTimeWithTimeZone,
    pub rating: Option<i16>,
    pub rated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef")
    }
}

impl ActiveModelBehavior for ActiveModel {}
