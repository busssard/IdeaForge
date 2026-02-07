//! IdeaForge Core - Domain types, traits, and errors
//!
//! This crate contains the shared domain model used by all other IdeaForge crates.
//! It has no infrastructure dependencies (no database, no HTTP, no external services).

pub mod domain;
pub mod error;

pub use domain::*;
pub use error::AppError;
