//! SeaORM Entity. Generated by sea-orm-codegen 0.9.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(schema_name = "ccs_schema", table_name = "consulting_fee")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_account_id: i64,
    pub fee_per_hour_in_yen: i32,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef")
    }
}

impl ActiveModelBehavior for ActiveModel {}
