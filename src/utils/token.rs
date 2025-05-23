use axum::http::StatusCode;
use jsonwebtoken::{decode, encode, errors::{Error, ErrorKind}, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

use chrono::{Duration, Utc};

use crate::utils::error::{ErrorMessage, HttpError};

pub fn create_token(
    user_id: &str,
    secret: &[u8],
    expires_in_seconds: i64,
) -> Result<String, Error> {
    if user_id.is_empty() {
        return Err(ErrorKind::InvalidSubject.into());
    }

    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(expires_in_seconds)).timestamp() as usize;
    let claims = TokenClaims {
        sub: user_id.to_string(),
        iat,
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
}

pub fn decode_token<T: Into<String>>(token: T, secret: &[u8]) -> Result<String, HttpError> {
    let decode = decode::<TokenClaims>(
        &token.into(),
        &DecodingKey::from_secret(secret),
        &Validation::new(Algorithm::HS256),
    );

    match decode {
        Ok(token) => Ok(token.claims.sub),
        Err(_) => Err(HttpError::new(
            ErrorMessage::InvalidToken.to_string(),
            StatusCode::UNAUTHORIZED,
        )),
    }
}

