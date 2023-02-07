//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(
    schema_name = "ccs_schema",
    table_name = "rejected_update_identity_req"
)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub rjd_upd_identity_id: i64,
    pub user_account_id: i64,
    pub last_name: String,
    pub first_name: String,
    pub last_name_furigana: String,
    pub first_name_furigana: String,
    pub date_of_birth: Date,
    pub prefecture: String,
    pub city: String,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub telephone_number: String,
    pub reason: String,
    pub rejected_at: DateTimeWithTimeZone,
    pub rejected_by: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
