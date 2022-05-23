//! SeaORM Entity. Generated by sea-orm-codegen 0.8.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "identity", schema_name = "ccs_schema")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
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
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef")
    }
}

impl ActiveModelBehavior for ActiveModel {}
