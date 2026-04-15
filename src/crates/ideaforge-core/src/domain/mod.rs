// MVP domain modules
pub mod category;
pub mod contribution;
pub mod idea;
pub mod notification;
pub mod team;
pub mod user;

// Re-export MVP types at module level
pub use category::*;
pub use contribution::*;
pub use idea::*;
pub use notification::*;
pub use team::*;
pub use user::*;

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
