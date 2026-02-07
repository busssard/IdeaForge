//! Repository layer providing domain-oriented database access.
//!
//! Each repository encapsulates queries for a specific domain entity,
//! converting between SeaORM entities and core domain types.

pub mod idea_repo;
pub mod user_repo;
pub mod pledge_repo;
