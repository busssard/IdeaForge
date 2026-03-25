use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "nda_signatures")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub nda_template_id: Uuid,
    pub idea_id: Uuid,
    pub signer_id: Uuid,
    pub signer_name: String,
    pub signer_email: String,
    pub ip_address: Option<String>,
    pub signed_at: DateTimeWithTimeZone,
    pub expires_at: Option<DateTimeWithTimeZone>,
    pub revoked_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::nda_template::Entity",
        from = "Column::NdaTemplateId",
        to = "super::nda_template::Column::Id"
    )]
    NdaTemplate,
    #[sea_orm(
        belongs_to = "super::idea::Entity",
        from = "Column::IdeaId",
        to = "super::idea::Column::Id"
    )]
    Idea,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::SignerId",
        to = "super::user::Column::Id"
    )]
    Signer,
}

impl Related<super::nda_template::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NdaTemplate.def()
    }
}

impl Related<super::idea::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Idea.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Signer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
