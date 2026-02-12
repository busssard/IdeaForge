use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::enums::{IdeaMaturity, IdeaOpenness};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "ideas")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub summary: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub maturity: IdeaMaturity,
    pub openness: IdeaOpenness,
    pub category_id: Option<Uuid>,
    pub stoke_count: i32,
    #[sea_orm(column_type = "JsonBinary")]
    pub looking_for_skills: serde_json::Value,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub archived_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::AuthorId",
        to = "super::user::Column::Id"
    )]
    Author,
    #[sea_orm(
        belongs_to = "super::category::Entity",
        from = "Column::CategoryId",
        to = "super::category::Column::Id"
    )]
    Category,
    #[sea_orm(has_many = "super::stoke::Entity")]
    Stokes,
    #[sea_orm(has_many = "super::contribution::Entity")]
    Contributions,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Author.def()
    }
}

impl Related<super::category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Category.def()
    }
}

impl Related<super::stoke::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Stokes.def()
    }
}

impl Related<super::contribution::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contributions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
