use ideaforge_core::Permission;

/// Check if a set of user permissions includes the required permission.
pub fn has_permission(user_permissions: &[Permission], required: &str) -> bool {
    user_permissions.iter().any(|p| p.matches(required))
}

/// MVP permission constants (simplified for 3 roles).
pub mod perms {
    // Ideas
    pub const IDEAS_CREATE: &str = "ideas.create";
    pub const IDEAS_UPDATE_OWN: &str = "ideas.update_own";
    pub const IDEAS_DELETE_OWN: &str = "ideas.delete_own";
    pub const IDEAS_SET_MATURITY: &str = "ideas.set_maturity";

    // Stokes (human approvals)
    pub const STOKES_CREATE: &str = "stokes.create";

    // Contributions
    pub const CONTRIBUTIONS_CREATE: &str = "contributions.create";

    // Task boards
    pub const BOARD_CREATE: &str = "board.create";
    pub const BOARD_MANAGE_OWN: &str = "board.manage_own";
    pub const BOARD_TASKS_CLAIM: &str = "board.tasks.claim";

    // Team
    pub const TEAM_APPLY: &str = "team.apply";
    pub const TEAM_MANAGE_OWN: &str = "team.manage_own";

    // Admin
    pub const ADMIN_ALL: &str = "admin.*";
}
