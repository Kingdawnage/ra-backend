use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, SaltString}, Argon2, PasswordHash, PasswordVerifier};

use super::error::ErrorMessage;

const MAX_PASSWORD_LENGTH: usize = 64;

pub fn hash(password: impl Into<String>) -> Result<String, ErrorMessage> {
    let password = password.into();

    if password.is_empty() {
        return Err(ErrorMessage::EmptyPassword);
    }

    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(ErrorMessage::ExceededMaxPassowrdLength(MAX_PASSWORD_LENGTH));
    }

    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| ErrorMessage::HashingError)?
        .to_string();

    Ok(password_hash)
}

pub fn compare(password: &str, password_hash: &str) -> Result<bool, ErrorMessage> {
    let argon2 = Argon2::default();

    if password.is_empty() {
        return Err(ErrorMessage::EmptyPassword);
    }

    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(ErrorMessage::ExceededMaxPassowrdLength(MAX_PASSWORD_LENGTH));
    }

    let parsed_hash = PasswordHash::new(password_hash).map_err(|_| ErrorMessage::HashingError)?;

    let password_matched = argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_or(false, |_| true);

    Ok(password_matched)
}