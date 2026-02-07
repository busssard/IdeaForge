//! IdeaForge Search - Full-text search powered by Tantivy.
//!
//! Provides indexing and querying for ideas and users.
//! The search backend is abstracted behind a trait to allow
//! future migration from embedded Tantivy to Meilisearch.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Trait abstracting the search backend. Enables swapping Tantivy for Meilisearch.
pub trait SearchEngine: Send + Sync {
    /// Index an idea for full-text search.
    fn index_idea(&self, doc: IdeaDocument) -> Result<(), SearchError>;

    /// Remove an idea from the index.
    fn remove_idea(&self, idea_id: Uuid) -> Result<(), SearchError>;

    /// Search ideas by query string.
    fn search_ideas(&self, query: &str, limit: usize, offset: usize)
        -> Result<SearchResults, SearchError>;
}

/// Document representing an idea in the search index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeaDocument {
    pub id: Uuid,
    pub title: String,
    pub summary: String,
    pub description: String,
    pub category_names: Vec<String>,
    pub author_name: String,
    pub maturity: String,
}

/// Search results with pagination metadata.
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResults {
    pub hits: Vec<SearchHit>,
    pub total: usize,
}

/// A single search result.
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchHit {
    pub id: Uuid,
    pub title: String,
    pub summary: String,
    pub score: f32,
}

/// Errors from the search subsystem.
#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("Index error: {0}")]
    IndexError(String),

    #[error("Query error: {0}")]
    QueryError(String),
}
