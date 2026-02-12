use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::enums::ContributionKind;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "contributions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub idea_id: Uuid,
    pub user_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub contribution_type: ContributionKind,
    pub title: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub body: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
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
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentId",
        to = "Column::Id"
    )]
    Parent,
}

impl Related<super::idea::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Idea.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
