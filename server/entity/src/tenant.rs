//! SeaORM Entity. Generated by sea-orm-codegen 0.6.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "tenant")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_account_id: i32,
    #[sea_orm(unique)]
    pub tenant_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user_account::Entity",
        from = "Column::UserAccountId",
        to = "super::user_account::Column::UserAccountId",
        on_update = "Restrict",
        on_delete = "Cascade"
    )]
    UserAccount,
}

impl Related<super::user_account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserAccount.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
