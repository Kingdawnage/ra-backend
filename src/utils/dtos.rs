use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::models::users::{User, UserRole};

#[derive(Debug, Serialize, Deserialize, Clone, Default, Validate)]
pub struct RegisterUserDto {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,

    #[validate(
        email(message = "Email is not valid"),
        length(min = 1, message = "Email is required")
    )]
    pub email: String,

    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,

    #[validate(
        length(min = 1, message = "Confirm password is required"),
        must_match(other = "password", message = "Passwords do not match")
    )]
    pub confirm_password: String
}

#[derive(Serialize, Deserialize, Validate, Debug, Default, Clone)]
pub struct LoginUserDto {
    #[validate(
        length(min = 1, message = "Email is required"),
        email(message = "Email is invalid")
    )]
    pub email: String,

    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct RequestQueryDto {
    #[validate(range(min = 1))]
    pub page: Option<usize>,
    #[validate(range(min = 1, max = 50))]
    pub limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct FilterUserDto {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub verified: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

impl FilterUserDto {
    pub fn filter_user(user: &User) -> Self {
        FilterUserDto {
            id: user.id.to_string(),
            name: user.name.to_owned(),
            email: user.email.to_owned(),
            verified: user.verified,
            role: user.role.to_str().to_string(),
            created_at: user.created_at.unwrap(),
            updated_at: user.updated_at.unwrap(),
        }
    }

    pub fn filter_users(users: &[User]) -> Vec<Self> {
        users.iter().map(FilterUserDto::filter_user).collect()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserData {
    pub user: FilterUserDto,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserResponseDto {
    pub status: String,
    pub data: UserData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserListResponseDto {
    pub status: String,
    pub users: Vec<FilterUserDto>,
    pub results: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserLoginResponseDto {
    pub status: String,
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub status: &'static str,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, Validate)]
pub struct NameUpdateDto {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct RoleUpdateDto {
    #[validate(custom = "validate_user_role")]
    pub role: UserRole,
}

fn validate_user_role(role: &UserRole) -> Result<(), ValidationError> {
    match role {
        UserRole::Admin | UserRole::User => Ok(()),
        // _ => Err(ValidationError::new("invalid_role")),
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Validate)]
pub struct UserPassUpdateDto {
    #[validate(length(min = 6, message = "New password must be at least 6 characters"))]
    pub new_password: String,

    #[validate(
        length(min = 6, message = "New password confirm is required"),
        must_match(other = "new_password", message = "Passwords do not match")
    )]
    pub new_password_confirm: String,

    #[validate(length(min = 6, message = "Old password must be at least 6 characters"))]
    pub old_password: String,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct VerifyEmailQueryDto {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct ForgotPasswordRequestDto {
    #[validate(
        length(min = 1, message = "Email is required"),
        email(message = "Email is invalid")
    )]
    pub email: String,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct ResetPasswordRequestDto {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,

    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub new_password: String,

    #[validate(
        length(min = 6, message = "Password confirm is required"),
        must_match(other = "new_password", message = "Passwords do not match")
    )]
    pub new_password_confirm: String,
}
