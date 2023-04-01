//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(schema_name = "ccs_schema", table_name = "temp_mfa_secret")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub temp_mfa_secret_id: i64,
    pub user_account_id: i64,
    #[sea_orm(column_type = "Text")]
    pub base32_encoded_secret: String,
    pub expired_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
