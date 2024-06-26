//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.10

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(schema_name = "ccs_schema", table_name = "identity")]
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

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::awaiting_payment::Entity",
        from = "Column::UserAccountId",
        to = "super::awaiting_payment::Column::UserAccountId"
    )]
    AwaitingPayment,
}

impl Related<super::awaiting_payment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AwaitingPayment.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
