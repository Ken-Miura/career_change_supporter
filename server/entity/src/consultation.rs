//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(schema_name = "ccs_schema", table_name = "consultation")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub consultation_id: i64,
    pub user_account_id: i64,
    pub consultant_id: i64,
    pub meeting_at: DateTimeWithTimeZone,
    #[sea_orm(column_type = "Text", unique)]
    pub charge_id: String,
    pub user_account_peer_id: Option<String>,
    pub user_account_peer_opened_at: Option<DateTimeWithTimeZone>,
    pub consultant_peer_id: Option<String>,
    pub consultant_peer_opend_at: Option<DateTimeWithTimeZone>,
    pub consultation_started_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
