use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::models::{resume::Resume, users::{User, UserRole}};

#[derive(Debug, Clone)]
pub struct DBClient {
    pool: Pool<Postgres>
}

impl DBClient {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
pub trait UserActions {
    async fn get_user(
        &self, 
        user_id: Option<Uuid>,
        name: Option<&str>,
        email: Option<&str>,
        token: Option<&str>,
    ) -> Result<Option<User>, sqlx::Error>;

    async fn get_users(&self, page: u32, limit: usize) -> Result<Vec<User>, sqlx::Error>;

    async fn save_user<T: Into<String> + Send>(
        &self,
        name: T,
        email: T,
        password: T,
        verification_token: T,
        token_expiration: DateTime<Utc>,
    ) -> Result<User, sqlx::Error>;

    async fn get_user_count(&self) -> Result<i64, sqlx::Error>;

    async fn update_user_name<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        name: T,
    ) -> Result<User, sqlx::Error>;

    async fn update_user_role(&self, user_id: Uuid, role: UserRole) -> Result<User, sqlx::Error>;

    async fn update_user_password(
        &self,
        user_id: Uuid,
        password: String,
    ) -> Result<User, sqlx::Error>;

    async fn verifed_token(&self, token: &str) -> Result<(), sqlx::Error>;

    async fn add_verifed_token(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error>;

    async fn save_resume<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        file_path: T,
        analysis_result: Option<serde_json::Value>,
    ) -> Result<Resume, sqlx::Error>;

    async fn get_resume(
        &self,
        user_id: Option<Uuid>,
        resume_id: Option<Uuid>,
    ) -> Result<Option<Resume>, sqlx::Error>;

    async fn delete_resume(
        &self,
        user_id: Option<Uuid>,
        resume_id: Option<Uuid>,
    ) -> Result<(), sqlx::Error>;

    async fn get_resumes(
        &self,
        user_id: Uuid,
        page: u32,
        limit: usize,
    ) -> Result<Vec<Resume>, sqlx::Error>;

}

#[async_trait]
impl UserActions for DBClient {
    async fn save_resume<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        file_path: T,
        analysis_result: Option<serde_json::Value>,
    ) -> Result<Resume, sqlx::Error> {
        let resume = sqlx::query_as!(
            Resume,
            r#"
            INSERT INTO resumes (user_id, file_path, analysis_result)
            VALUES ($1, $2, $3::jsonb)
            RETURNING id, user_id, file_path, analysis_result, uploaded_at
            "#,
            user_id,
            file_path.into(),
            analysis_result
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(resume)
    }

    async fn get_resume(
        &self,
        user_id: Option<Uuid>,
        resume_id: Option<Uuid>,
    ) -> Result<Option<Resume>, sqlx::Error> {
        if let (Some(user_id), Some(resume_id)) = (user_id, resume_id) {
            let resume = sqlx::query_as!(
                Resume,
                r#"
                SELECT id, user_id, file_path, analysis_result, uploaded_at
                FROM resumes
                WHERE id = $1 AND user_id = $2
                "#,
                resume_id,
                user_id
            )
            .fetch_optional(&self.pool)
            .await?;
            
            Ok(resume)
        } else {
            Ok(None)
        }
    }

    async fn delete_resume(
        &self,
        user_id: Option<Uuid>,
        resume_id: Option<Uuid>
    ) -> Result<(), sqlx::Error> {
        if let (Some(user_id), Some(resume_id)) = (user_id, resume_id) {
            let _ = sqlx::query!(
                r#"
                DELETE FROM resumes
                WHERE id = $1 AND user_id = $2
                "#,
                resume_id,
                user_id
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    async fn get_resumes(
        &self,
        user_id: Uuid,
        page: u32,
        limit: usize,
    ) -> Result<Vec<Resume>, sqlx::Error> {
        let offset = (page - 1) * limit as u32;

        let resumes = sqlx::query_as!(
            Resume,
            r#"
            SELECT id, user_id, file_path, analysis_result, uploaded_at
            FROM resumes
            WHERE user_id = $1
            ORDER BY uploaded_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(resumes)
    }

    async fn get_user(
        &self,
        user_id: Option<Uuid>,
        name: Option<&str>,
        email: Option<&str>,
        token: Option<&str>,
    ) -> Result<Option<User>, sqlx::Error> {
        let mut user: Option<User> = None;

        if let Some(user_id) = user_id {
            user = sqlx::query_as!(
                User,
                r#"SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at as "token_expiration?", role as "role: UserRole" FROM users WHERE id = $1"#,
                user_id
            ).fetch_optional(&self.pool).await?;
        } else if let Some(name) = name {
            user = sqlx::query_as!(
                User,
                r#"SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at as "token_expiration?", role as "role: UserRole" FROM users WHERE name = $1"#,
                name
            ).fetch_optional(&self.pool).await?;
        } else if let Some(email) = email {
            user = sqlx::query_as!(
                User,
                r#"SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at as "token_expiration?", role as "role: UserRole" FROM users WHERE email = $1"#,
                email
            ).fetch_optional(&self.pool).await?;
        } else if let Some(token) = token {
            user = sqlx::query_as!(
                User,
                r#"
                SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at as "token_expiration?", role as "role: UserRole" 
                FROM users 
                WHERE verification_token = $1"#,
                token
            )
            .fetch_optional(&self.pool)
            .await?;
        }

        Ok(user)
    }

    async fn get_users(&self, page: u32, limit: usize) -> Result<Vec<User>, sqlx::Error> {
        let offset = (page - 1) * limit as u32;

        let users = sqlx::query_as!(
            User,
            r#"SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at as "token_expiration?", role as "role: UserRole" FROM users 
            ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
            limit as i64,
            offset as i64,
        ).fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    async fn save_user<T: Into<String> + Send>(
        &self,
        name: T,
        email: T,
        password: T,
        verification_token: T,
        token_expires_at: DateTime<Utc>,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (name, email, password,verification_token, token_expires_at) 
            VALUES ($1, $2, $3, $4, $5) 
            RETURNING id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at as "token_expiration?", role as "role: UserRole"
            "#,
            name.into(),
            email.into(),
            password.into(),
            verification_token.into(),
            token_expires_at
        ).fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    async fn get_user_count(&self) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar!(r#"SELECT COUNT(*) FROM users"#)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.unwrap_or(0))
    }

    async fn update_user_name<T: Into<String> + Send>(
        &self,
        user_id: Uuid,
        new_name: T,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET name = $1, updated_at = Now()
            WHERE id = $2
            RETURNING id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at as "token_expiration?", role as "role: UserRole"
            "#,
            new_name.into(),
            user_id
        ).fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn update_user_role(
        &self,
        user_id: Uuid,
        new_role: UserRole,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET role = $1, updated_at = Now()
            WHERE id = $2
            RETURNING id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at as "token_expiration?", role as "role: UserRole"
            "#,
            new_role as UserRole,
            user_id
        ).fetch_one(&self.pool)
       .await?;

        Ok(user)
    }

    async fn update_user_password(
        &self,
        user_id: Uuid,
        new_password: String,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET password = $1, updated_at = Now()
            WHERE id = $2
            RETURNING id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at as "token_expiration?", role as "role: UserRole"
            "#,
            new_password,
            user_id
        ).fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn verifed_token(&self, token: &str) -> Result<(), sqlx::Error> {
        let _ = sqlx::query!(
            r#"
            UPDATE users
            SET verified = true, 
                updated_at = Now(),
                verification_token = NULL,
                token_expires_at = NULL
            WHERE verification_token = $1
            "#,
            token
        )
        .execute(&self.pool)
        .await;

        Ok(())
    }

    async fn add_verifed_token(
        &self,
        user_id: Uuid,
        token: &str,
        token_expires_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        let _ = sqlx::query!(
            r#"
            UPDATE users
            SET verification_token = $1, token_expires_at = $2, updated_at = Now()
            WHERE id = $3
            "#,
            token,
            token_expires_at,
            user_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}