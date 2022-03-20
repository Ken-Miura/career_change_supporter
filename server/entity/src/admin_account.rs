//! SeaORM Entity. Generated by sea-orm-codegen 0.6.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "admin_account", schema_name = "ccs_schema")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub admin_account_id: i64,
    #[sea_orm(unique)]
    pub email_address: String,
    pub hashed_password: Vec<u8>,
    pub last_login_time: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef")
    }
}

impl ActiveModelBehavior for ActiveModel {}
