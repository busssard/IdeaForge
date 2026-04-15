use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "mls_messages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub group_id: Uuid,
    pub sender_user_id: Uuid,
    pub ciphertext: Vec<u8>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::mls_group::Entity",
        from = "Column::GroupId",
        to = "super::mls_group::Column::Id"
    )]
    Group,
}

impl ActiveModelBehavior for ActiveModel {}
