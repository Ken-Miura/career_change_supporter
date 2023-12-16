//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.10

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(schema_name = "ccs_schema", table_name = "consultation_req")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub consultation_req_id: i64,
    pub user_account_id: i64,
    pub consultant_id: i64,
    pub first_candidate_date_time: DateTimeWithTimeZone,
    pub second_candidate_date_time: DateTimeWithTimeZone,
    pub third_candidate_date_time: DateTimeWithTimeZone,
    pub latest_candidate_date_time: DateTimeWithTimeZone,
    pub fee_per_hour_in_yen: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
