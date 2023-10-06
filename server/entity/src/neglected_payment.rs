//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(schema_name = "ccs_schema", table_name = "neglected_payment")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub consultation_id: i64,
    pub user_account_id: i64,
    pub consultant_id: i64,
    pub meeting_at: DateTimeWithTimeZone,
    pub fee_per_hour_in_yen: i32,
    #[sea_orm(column_type = "Text")]
    pub sender_name: String,
    pub neglect_confirmed_by: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
