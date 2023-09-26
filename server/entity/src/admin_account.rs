//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(schema_name = "ccs_schema", table_name = "admin_account")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub admin_account_id: i64,
    #[sea_orm(unique)]
    pub email_address: String,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub hashed_password: Vec<u8>,
    pub last_login_time: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub mfa_enabled_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
