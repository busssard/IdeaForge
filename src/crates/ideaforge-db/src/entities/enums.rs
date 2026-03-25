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
    #[sea_orm(string_value = "admin")]
    Admin,
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
            Self::Admin => write!(f, "admin"),
        }
    }
}

impl UserRole {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "entrepreneur" => Some(Self::Entrepreneur),
            "maker" => Some(Self::Maker),
            "curious" => Some(Self::Curious),
            "admin" => Some(Self::Admin),
            _ => None,
        }
    }

    /// Whether this role can create ideas.
    pub fn can_create_ideas(&self) -> bool {
        matches!(self, Self::Entrepreneur | Self::Maker | Self::Admin)
    }

    /// Whether this role can submit formal suggestions (not just comments).
    pub fn can_suggest(&self) -> bool {
        matches!(self, Self::Entrepreneur | Self::Maker | Self::Admin)
    }

    /// Whether this role has admin/moderation powers.
    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Admin)
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
    #[sea_orm(string_value = "private")]
    Private,
    #[sea_orm(string_value = "nda_protected")]
    NdaProtected,
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
            Self::Private => write!(f, "private"),
            Self::NdaProtected => write!(f, "nda_protected"),
        }
    }
}

impl IdeaOpenness {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "open" => Some(Self::Open),
            "collaborative" => Some(Self::Collaborative),
            "commercial" => Some(Self::Commercial),
            "private" => Some(Self::Private),
            "nda_protected" => Some(Self::NdaProtected),
            _ => None,
        }
    }

    /// Whether this idea should appear in public browse/search results.
    pub fn is_publicly_listed(&self) -> bool {
        matches!(self, Self::Open | Self::Collaborative | Self::Commercial | Self::NdaProtected)
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

impl std::fmt::Display for ContributionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Comment => write!(f, "comment"),
            Self::Suggestion => write!(f, "suggestion"),
            Self::Design => write!(f, "design"),
            Self::Code => write!(f, "code"),
            Self::Research => write!(f, "research"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// PostgreSQL enum: team_member_role
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "team_member_role")]
pub enum TeamMemberRole {
    #[sea_orm(string_value = "lead")]
    Lead,
    #[sea_orm(string_value = "builder")]
    Builder,
    #[sea_orm(string_value = "advisor")]
    Advisor,
}

impl Default for TeamMemberRole {
    fn default() -> Self {
        Self::Builder
    }
}

impl std::fmt::Display for TeamMemberRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lead => write!(f, "lead"),
            Self::Builder => write!(f, "builder"),
            Self::Advisor => write!(f, "advisor"),
        }
    }
}

impl TeamMemberRole {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "lead" => Some(Self::Lead),
            "builder" => Some(Self::Builder),
            "advisor" => Some(Self::Advisor),
            _ => None,
        }
    }
}

/// PostgreSQL enum: application_status
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "application_status")]
pub enum ApplicationStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "accepted")]
    Accepted,
    #[sea_orm(string_value = "rejected")]
    Rejected,
    #[sea_orm(string_value = "withdrawn")]
    Withdrawn,
}

impl Default for ApplicationStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for ApplicationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Accepted => write!(f, "accepted"),
            Self::Rejected => write!(f, "rejected"),
            Self::Withdrawn => write!(f, "withdrawn"),
        }
    }
}

impl ApplicationStatus {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "accepted" => Some(Self::Accepted),
            "rejected" => Some(Self::Rejected),
            "withdrawn" => Some(Self::Withdrawn),
            _ => None,
        }
    }
}

/// PostgreSQL enum: invite_permission
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "invite_permission")]
pub enum InvitePermission {
    #[sea_orm(string_value = "view")]
    View,
    #[sea_orm(string_value = "comment")]
    Comment,
}

impl Default for InvitePermission {
    fn default() -> Self {
        Self::View
    }
}

/// PostgreSQL enum: flag_target_type
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "flag_target_type")]
pub enum FlagTargetType {
    #[sea_orm(string_value = "idea")]
    Idea,
    #[sea_orm(string_value = "comment")]
    Comment,
    #[sea_orm(string_value = "user")]
    User,
}

impl std::fmt::Display for FlagTargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idea => write!(f, "idea"),
            Self::Comment => write!(f, "comment"),
            Self::User => write!(f, "user"),
        }
    }
}

impl FlagTargetType {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "idea" => Some(Self::Idea),
            "comment" => Some(Self::Comment),
            "user" => Some(Self::User),
            _ => None,
        }
    }
}

/// PostgreSQL enum: flag_status
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "flag_status")]
pub enum FlagStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "reviewed")]
    Reviewed,
    #[sea_orm(string_value = "dismissed")]
    Dismissed,
}

impl Default for FlagStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for FlagStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Reviewed => write!(f, "reviewed"),
            Self::Dismissed => write!(f, "dismissed"),
        }
    }
}

impl FlagStatus {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "reviewed" => Some(Self::Reviewed),
            "dismissed" => Some(Self::Dismissed),
            _ => None,
        }
    }
}

/// PostgreSQL enum: notification_kind
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "notification_kind")]
pub enum NotificationKind {
    #[sea_orm(string_value = "stoke")]
    Stoke,
    #[sea_orm(string_value = "comment")]
    Comment,
    #[sea_orm(string_value = "suggestion")]
    Suggestion,
    #[sea_orm(string_value = "team_application")]
    TeamApplication,
    #[sea_orm(string_value = "team_accepted")]
    TeamAccepted,
    #[sea_orm(string_value = "team_rejected")]
    TeamRejected,
    #[sea_orm(string_value = "milestone")]
    Milestone,
    #[sea_orm(string_value = "bot_analysis")]
    BotAnalysis,
    #[sea_orm(string_value = "mention")]
    Mention,
    #[sea_orm(string_value = "nda_signed")]
    NdaSigned,
}

impl std::fmt::Display for NotificationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stoke => write!(f, "stoke"),
            Self::Comment => write!(f, "comment"),
            Self::Suggestion => write!(f, "suggestion"),
            Self::TeamApplication => write!(f, "team_application"),
            Self::TeamAccepted => write!(f, "team_accepted"),
            Self::TeamRejected => write!(f, "team_rejected"),
            Self::Milestone => write!(f, "milestone"),
            Self::BotAnalysis => write!(f, "bot_analysis"),
            Self::Mention => write!(f, "mention"),
            Self::NdaSigned => write!(f, "nda_signed"),
        }
    }
}

impl NotificationKind {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "stoke" => Some(Self::Stoke),
            "comment" => Some(Self::Comment),
            "suggestion" => Some(Self::Suggestion),
            "team_application" => Some(Self::TeamApplication),
            "team_accepted" => Some(Self::TeamAccepted),
            "team_rejected" => Some(Self::TeamRejected),
            "milestone" => Some(Self::Milestone),
            "bot_analysis" => Some(Self::BotAnalysis),
            "mention" => Some(Self::Mention),
            "nda_signed" => Some(Self::NdaSigned),
            _ => None,
        }
    }
}

/// PostgreSQL enum: task_status
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "task_status")]
pub enum TaskStatus {
    #[sea_orm(string_value = "open")]
    Open,
    #[sea_orm(string_value = "assigned")]
    Assigned,
    #[sea_orm(string_value = "in_review")]
    InReview,
    #[sea_orm(string_value = "done")]
    Done,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Open
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Assigned => write!(f, "assigned"),
            Self::InReview => write!(f, "in_review"),
            Self::Done => write!(f, "done"),
        }
    }
}

impl TaskStatus {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "open" => Some(Self::Open),
            "assigned" => Some(Self::Assigned),
            "in_review" => Some(Self::InReview),
            "done" => Some(Self::Done),
            _ => None,
        }
    }
}

/// PostgreSQL enum: task_priority
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "task_priority")]
pub enum TaskPriority {
    #[sea_orm(string_value = "low")]
    Low,
    #[sea_orm(string_value = "normal")]
    Normal,
    #[sea_orm(string_value = "high")]
    High,
    #[sea_orm(string_value = "urgent")]
    Urgent,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "high"),
            Self::Urgent => write!(f, "urgent"),
        }
    }
}

impl TaskPriority {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "low" => Some(Self::Low),
            "normal" => Some(Self::Normal),
            "high" => Some(Self::High),
            "urgent" => Some(Self::Urgent),
            _ => None,
        }
    }
}
