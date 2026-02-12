use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::enums::{FlagStatus, FlagTargetType};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "flags")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub flagger_id: Uuid,
    pub target_type: FlagTargetType,
    pub target_id: Uuid,
    pub reason: String,
    pub status: FlagStatus,
    pub reviewed_by: Option<Uuid>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::FlaggerId",
        to = "super::user::Column::Id"
    )]
    Flagger,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Flagger.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
