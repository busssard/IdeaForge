//! Custom Axum extractors for authentication and authorization.
//!
//! Usage in handlers:
//! ```rust,ignore
//! async fn create_idea(
//!     Auth(user): Auth,
//!     Json(payload): Json<CreateIdeaRequest>,
//! ) -> Result<Json<IdeaResponse>, ApiError> { ... }
//! ```

// TODO: Implement Auth extractor that:
// 1. Reads Bearer token from Authorization header (or X-Api-Key for bots)
// 2. Validates JWT / API key
// 3. Returns authenticated user context
//
// TODO: Implement Permission extractor that:
// 1. Takes Auth context
// 2. Checks user roles against required permission
// 3. Returns 403 if unauthorized
