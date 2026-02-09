// MVP domain modules
pub mod idea;
pub mod user;
pub mod team;
pub mod contribution;
pub mod category;
pub mod notification;

// Re-export MVP types at module level
pub use idea::*;
pub use user::*;
pub use team::*;
pub use contribution::*;
pub use category::*;
pub use notification::*;

// =============================================================================
// DEFERRED modules (Phase 2+)
// =============================================================================
// The following modules are part of the long-term architecture but are not
// needed for the MVP. They are kept as empty stubs to preserve the crate
// structure and make future restoration straightforward.
//
// - agent.rs     -> Phase 2+ (AI agent accounts, bot transparency)
// - pledge.rs    -> Phase 2-3 (Cardano blockchain pledges)
// - todo.rs      -> Replaced by team.rs BoardTask (richer task board model)
