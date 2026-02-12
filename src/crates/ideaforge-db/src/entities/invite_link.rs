use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::enums::InvitePermission;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "invite_links")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub idea_id: Uuid,
    #[sea_orm(unique)]
    pub token: String,
    pub permission: InvitePermission,
    pub created_by: Uuid,
    pub expires_at: Option<DateTimeWithTimeZone>,
    pub revoked_at: Option<DateTimeWithTimeZone>,
    pub access_count: i32,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::idea::Entity",
        from = "Column::IdeaId",
        to = "super::idea::Column::Id"
    )]
    Idea,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::CreatedBy",
        to = "super::user::Column::Id"
    )]
    Creator,
}

impl Related<super::idea::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Idea.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Creator.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
