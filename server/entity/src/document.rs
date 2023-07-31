//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(schema_name = "ccs_schema", table_name = "document")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_account_id: i64,
    pub document_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
