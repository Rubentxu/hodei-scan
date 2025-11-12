/// JWT Authentication module
use crate::modules::error::{Result, ServerError};
use crate::modules::types::{AuthToken, User, UserId};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_id: UserId,
    username: String,
    exp: usize,
    iat: usize,
}

/// Authentication service
pub struct AuthService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiration_hours: u64,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(secret: String, expiration_hours: u64) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());

        Self {
            encoding_key,
            decoding_key,
            expiration_hours,
        }
    }

    /// Generate a JWT token for a user
    pub fn generate_token(&self, user: &User) -> Result<AuthToken> {
        let now = Utc::now();
        let expiration = now + Duration::hours(self.expiration_hours as i64);

        let claims = Claims {
            user_id: user.id,
            username: user.username.clone(),
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let token = jsonwebtoken::encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| ServerError::Jwt(e.to_string()))?;

        Ok(AuthToken {
            token,
            user_id: user.id,
            expires_at: expiration,
        })
    }

    /// Validate a JWT token and return user info
    pub fn validate_token(&self, token: &str) -> Result<UserId> {
        let decoded =
            jsonwebtoken::decode::<Claims>(token, &self.decoding_key, &Validation::default())
                .map_err(|e| ServerError::Jwt(e.to_string()))?;

        Ok(decoded.claims.user_id)
    }

    /// Extract token from Authorization header
    pub fn extract_token_from_header(&self, auth_header: &str) -> Result<String> {
        let parts: Vec<&str> = auth_header.split_whitespace().collect();
        if parts.len() != 2 || parts[0] != "Bearer" {
            return Err(ServerError::Authentication(
                "Invalid authorization header format".to_string(),
            ));
        }
        Ok(parts[1].to_string())
    }
}

impl Clone for AuthService {
    fn clone(&self) -> Self {
        // We can't easily extract the secret from the keys, so we'll need to store it
        // or accept that cloning is expensive. For now, let's mark this as not implementable.
        unimplemented!("AuthService cannot be cloned due to key storage limitations")
    }
}
