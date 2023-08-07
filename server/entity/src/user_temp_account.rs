//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(schema_name = "ccs_schema", table_name = "user_temp_account")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_temp_account_id: String,
    pub email_address: String,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub hashed_password: Vec<u8>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
