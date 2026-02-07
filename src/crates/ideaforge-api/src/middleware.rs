//! Tower middleware configuration for the Axum server.
//!
//! Includes:
//! - CORS configuration
//! - Request tracing (via tower-http)
//! - Rate limiting (Redis-backed sliding window)
//! - Compression (gzip)

// TODO: Implement rate limiting middleware using tower and Redis
// TODO: Configure CORS for frontend origin
// TODO: Set up OpenTelemetry tracing
