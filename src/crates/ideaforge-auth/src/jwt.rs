use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT claims embedded in access tokens.
///
/// MVP: human-only, no is_bot field.
/// Phase 2+: restore is_bot for AI agent support.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: Uuid,
    /// User email
    pub email: String,
    /// User role (single role for MVP: entrepreneur / maker / curious)
    pub role: String,
    /// Issued at (unix timestamp)
    pub iat: i64,
    /// Expiration (unix timestamp)
    pub exp: i64,
}

/// Configuration for JWT token generation and validation.
pub struct JwtConfig {
    pub secret: String,
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
}

impl JwtConfig {
    /// Create an access token for a user.
    pub fn create_access_token(
        &self,
        user_id: Uuid,
        email: &str,
        role: &str,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id,
            email: email.to_string(),
            role: role.to_string(),
            iat: now.timestamp(),
            exp: (now + self.access_token_ttl).timestamp(),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
    }

    /// Validate and decode an access token.
    pub fn validate_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }
}
