use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::enums::NotificationKind;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "notifications")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub kind: NotificationKind,
    pub title: String,
    pub message: String,
    pub link_url: Option<String>,
    pub read_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    /// User who triggered this notification (the sender for messages, the
    /// stoker for stokes, etc.). Used to coalesce per-actor rows. DB
    /// default is NULL so existing rows load cleanly.
    pub related_user_id: Option<Uuid>,
    /// How many events this row collapses. 1 means a single event; >1 means
    /// this row has been bumped by subsequent same-kind same-actor events.
    /// DB default is 1.
    pub count: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
