use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::enums::ApplicationStatus;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "team_applications")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub idea_id: Uuid,
    pub user_id: Uuid,
    #[sea_orm(column_type = "Text")]
    pub message: String,
    pub status: ApplicationStatus,
    pub reviewed_by: Option<Uuid>,
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
        belongs_to = "super::user::Entity",
        from = "Column::ReviewedBy",
        to = "super::user::Column::Id"
    )]
    Reviewer,
}

impl Related<super::idea::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Idea.def()
    }
}

// Note: Can't impl Related<User> twice due to ambiguity.
// Use Relation::User.def() or Relation::Reviewer.def() directly when needed.

impl ActiveModelBehavior for ActiveModel {}
