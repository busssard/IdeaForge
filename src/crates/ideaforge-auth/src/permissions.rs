use ideaforge_core::Permission;

/// Check if a set of user permissions includes the required permission.
pub fn has_permission(user_permissions: &[Permission], required: &str) -> bool {
    user_permissions.iter().any(|p| p.matches(required))
}

/// Common permission constants.
pub mod perms {
    pub const IDEAS_CREATE: &str = "ideas.create";
    pub const IDEAS_UPDATE_OWN: &str = "ideas.update_own";
    pub const IDEAS_UPDATE_ANY: &str = "ideas.update_any";
    pub const IDEAS_DELETE_OWN: &str = "ideas.delete_own";
    pub const IDEAS_APPROVE: &str = "ideas.approve";
    pub const IDEAS_SET_MATURITY: &str = "ideas.set_maturity";
    pub const IDEAS_VIEW_SECRET: &str = "ideas.view_secret";
    pub const PLEDGES_CREATE: &str = "pledges.create";
    pub const PLEDGES_REFUND: &str = "pledges.refund";
    pub const CONTRIBUTIONS_CREATE: &str = "contributions.create";
    pub const CONTRIBUTIONS_MODERATE: &str = "contributions.moderate";
    pub const TODOS_CREATE: &str = "todos.create";
    pub const TODOS_ASSIGN: &str = "todos.assign";
    pub const USERS_MANAGE: &str = "users.manage";
    pub const USERS_VERIFY_EXPERT: &str = "users.verify_expert";
    pub const AI_AGENTS_REGISTER: &str = "ai_agents.register";
    pub const AI_AGENTS_MANAGE: &str = "ai_agents.manage";
    pub const ADMIN_ALL: &str = "admin.*";
}
