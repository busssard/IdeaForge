use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// PostgreSQL enum: user_role
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "user_role")]
pub enum UserRole {
    #[sea_orm(string_value = "entrepreneur")]
    Entrepreneur,
    #[sea_orm(string_value = "maker")]
    Maker,
    #[sea_orm(string_value = "curious")]
    Curious,
}

impl Default for UserRole {
    fn default() -> Self {
        Self::Curious
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Entrepreneur => write!(f, "entrepreneur"),
            Self::Maker => write!(f, "maker"),
            Self::Curious => write!(f, "curious"),
        }
    }
}

impl UserRole {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "entrepreneur" => Some(Self::Entrepreneur),
            "maker" => Some(Self::Maker),
            "curious" => Some(Self::Curious),
            _ => None,
        }
    }
}

/// PostgreSQL enum: idea_maturity
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "idea_maturity")]
pub enum IdeaMaturity {
    #[sea_orm(string_value = "spark")]
    Spark,
    #[sea_orm(string_value = "building")]
    Building,
    #[sea_orm(string_value = "in_work")]
    InWork,
}

impl Default for IdeaMaturity {
    fn default() -> Self {
        Self::Spark
    }
}

impl std::fmt::Display for IdeaMaturity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spark => write!(f, "spark"),
            Self::Building => write!(f, "building"),
            Self::InWork => write!(f, "in_work"),
        }
    }
}

impl IdeaMaturity {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "spark" => Some(Self::Spark),
            "building" => Some(Self::Building),
            "in_work" => Some(Self::InWork),
            _ => None,
        }
    }
}

/// PostgreSQL enum: idea_openness
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "idea_openness")]
pub enum IdeaOpenness {
    #[sea_orm(string_value = "open")]
    Open,
    #[sea_orm(string_value = "collaborative")]
    Collaborative,
    #[sea_orm(string_value = "commercial")]
    Commercial,
}

impl Default for IdeaOpenness {
    fn default() -> Self {
        Self::Open
    }
}

impl std::fmt::Display for IdeaOpenness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Collaborative => write!(f, "collaborative"),
            Self::Commercial => write!(f, "commercial"),
        }
    }
}

impl IdeaOpenness {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "open" => Some(Self::Open),
            "collaborative" => Some(Self::Collaborative),
            "commercial" => Some(Self::Commercial),
            _ => None,
        }
    }
}

/// PostgreSQL enum: contribution_type
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "contribution_type")]
pub enum ContributionKind {
    #[sea_orm(string_value = "comment")]
    Comment,
    #[sea_orm(string_value = "suggestion")]
    Suggestion,
    #[sea_orm(string_value = "design")]
    Design,
    #[sea_orm(string_value = "code")]
    Code,
    #[sea_orm(string_value = "research")]
    Research,
    #[sea_orm(string_value = "other")]
    Other,
}

impl Default for ContributionKind {
    fn default() -> Self {
        Self::Comment
    }
}
