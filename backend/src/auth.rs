use crate::{config::JwtConfig, error::ApiError, state::AppState};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use dormtk_core::Id;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Id,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthUser {
    pub user_id: Id,
    pub role: String,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = bearer_token(parts)?;
        decode_token(token, &state.config.jwt)
    }
}

pub fn encode_token(
    user_id: impl Into<Id>,
    role: impl Into<String>,
    config: &JwtConfig,
) -> Result<String, ApiError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| ApiError::internal(format!("system time error: {error}")))?;
    let claims = Claims {
        sub: user_id.into(),
        role: role.into(),
        exp: (now.as_secs() + config.expiration_seconds) as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|error| ApiError::internal(format!("token encode error: {error}")))
}

pub fn decode_token(token: &str, config: &JwtConfig) -> Result<AuthUser, ApiError> {
    let token = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| ApiError::unauthorized("invalid bearer token"))?;

    Ok(AuthUser {
        user_id: token.claims.sub,
        role: token.claims.role,
    })
}

pub fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|error| ApiError::internal(format!("password hash error: {error}")))?;

    Ok(hash.to_string())
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, ApiError> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|error| ApiError::internal(format!("password hash parse error: {error}")))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

fn bearer_token(parts: &Parts) -> Result<&str, ApiError> {
    let header = parts
        .headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("missing authorization header"))?;

    header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::unauthorized("invalid authorization scheme"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> JwtConfig {
        JwtConfig {
            secret: "test-secret".to_owned(),
            expiration_seconds: 60,
        }
    }

    #[test]
    fn token_roundtrip_returns_auth_user() {
        let config = test_config();
        let token = encode_token("user-1", "admin", &config).expect("token should encode");
        let user = decode_token(&token, &config).expect("token should decode");

        assert_eq!(
            user,
            AuthUser {
                user_id: "user-1".to_owned(),
                role: "admin".to_owned()
            }
        );
    }

    #[test]
    fn invalid_token_is_rejected() {
        let config = test_config();

        assert!(decode_token("not-a-token", &config).is_err());
    }

    #[test]
    fn password_hash_verifies_matching_password() {
        let hash = hash_password("secret").expect("password should hash");

        assert!(verify_password("secret", &hash).expect("password should verify"));
        assert!(!verify_password("other", &hash).expect("password should verify"));
    }
}
