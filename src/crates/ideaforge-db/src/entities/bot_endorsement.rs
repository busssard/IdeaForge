use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "bot_endorsements")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub idea_id: Uuid,
    pub bot_id: Uuid,
    pub reason: String,
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
        from = "Column::BotId",
        to = "super::user::Column::Id"
    )]
    Bot,
}

impl Related<super::idea::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Idea.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bot.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
