//! SeaORM Entity. Generated by sea-orm-codegen 0.9.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(schema_name = "ccs_schema", table_name = "pwd_change_req")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub pwd_change_req_id: String,
    pub email_address: String,
    pub requested_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef")
    }
}

impl ActiveModelBehavior for ActiveModel {}
