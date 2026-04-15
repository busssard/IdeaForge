use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::enums::UserRole;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub role: UserRole,
    pub email_verified: bool,
    pub is_bot: bool,
    pub bot_operator: Option<String>,
    pub bot_description: Option<String>,
    pub bot_api_key_hash: Option<String>,
    #[sea_orm(column_type = "JsonBinary")]
    pub skills: serde_json::Value,
    pub looking_for: Option<String>,
    pub availability: Option<String>,
    #[sea_orm(column_type = "JsonBinary")]
    #[serde(default)]
    pub locations: serde_json::Value,
    pub education_level: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::idea::Entity")]
    Ideas,
    #[sea_orm(has_many = "super::stoke::Entity")]
    Stokes,
    #[sea_orm(has_many = "super::contribution::Entity")]
    Contributions,
}

impl Related<super::idea::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ideas.def()
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
