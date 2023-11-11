//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.4

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(schema_name = "ccs_schema", table_name = "bank_account")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_account_id: i64,
    #[sea_orm(column_type = "Text")]
    pub bank_code: String,
    #[sea_orm(column_type = "Text")]
    pub branch_code: String,
    #[sea_orm(column_type = "Text")]
    pub account_type: String,
    #[sea_orm(column_type = "Text")]
    pub account_number: String,
    #[sea_orm(column_type = "Text")]
    pub account_holder_name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::awaiting_withdrawal::Entity",
        from = "Column::UserAccountId",
        to = "super::awaiting_withdrawal::Column::ConsultantId" // コンサルタントに報酬として振込を行いたいので、関連付けるカラムはConsultantIdであっている
    )]
    AwaitingWithdrawal,
}

impl Related<super::awaiting_withdrawal::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AwaitingWithdrawal.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
