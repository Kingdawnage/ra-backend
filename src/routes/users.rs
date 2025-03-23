use std::sync::Arc;

use axum::{extract::{Path, Query, Multipart}, middleware::from_fn, response::IntoResponse, routing::{get, post, put}, Extension, Json, Router};
use bytes::Bytes;
use tokio::fs;
use uuid::Uuid;
use validator::Validate;

use crate::{models::users::UserRole, services::{database::UserActions, middleware::{role_check, JWTAuthMiddleware}}, utils::{dtos::{FilterUserDto, RequestQueryDto, Response, ResumeUploadDto, RoleUpdateDto, UserData, UserListResponseDto, UserPassUpdateDto, UserResponseDto}, error::{ErrorMessage, HttpError}, password}, AppState};

pub fn user_routes() -> Router {
    Router::new()
        .route(
            "/me",
            get(get_me).layer(from_fn(|state, req, next| {
                role_check(state, req, next, vec![UserRole::Admin, UserRole::User])
            })),
        )
        .route(
            "/users",
            get(get_users).layer(from_fn(|state, req, next| {
                // Remove UserRole::User from the list to allow only Admin to access this route
                role_check(state, req, next, vec![UserRole::Admin, UserRole::User])
            })),
        )
        .route("/user/name", put(update_user_name))
        .route("/user/role", put(update_user_role))
        .route("/user/password", put(update_user_password))
        .route("/{user_id}/resume", post(upload_resume))
}

pub async fn get_me(
    Extension(_app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>
) -> Result<impl IntoResponse, HttpError> {
    let filtered_user = FilterUserDto::filter_user(&user.user);

    let response_data = UserResponseDto {
        status: "success".to_string(),
        data: UserData { user: filtered_user },
    };

    Ok(Json(response_data))
}

pub async fn upload_resume(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, HttpError> {
    let upload_dir = "./uploads/temp";
    fs::create_dir_all(upload_dir).await.map_err(|e| HttpError::server_error(e.to_string()))?;

    let user_id = &user.user.id;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| HttpError::bad_request(e.to_string()))?
        {
            let field_name = field.name().map(|s| s.to_string());
            let file_name = field
                .file_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| Uuid::new_v4().to_string());

            let data: Bytes = field
                .bytes()
                .await
                .map_err(|e| HttpError::bad_request(e.to_string()))?;

            let file_path = format!("{}/{}", upload_dir, file_name);
            fs::write(&file_path, &data).await.map_err(|e| HttpError::server_error(e.to_string()))?;

            println!(
                "User {} uploaded file from field {:?} with filename {}, saved at {}",
                user_id,
                field_name,
                file_name,
                file_path
            );

            let _result = app_state
                .db_client
                .save_resume(*user_id, &file_path, None)
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?;
        }

        Ok(Json(Response {
            message: "Resume uploaded successfully".to_string(),
            status: "success",
        }))
}

pub async fn post_resume(
    Path(user_id): Path<Uuid>,
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    Json(body): Json<ResumeUploadDto>,
) -> Result<impl IntoResponse, HttpError> {
    if user.user.id != user_id {
        return Err(HttpError::unauthorized(ErrorMessage::InvalidToken.to_string()));
    }

    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let user = &user.user;
    let user_id = Uuid::parse_str(&user.id.to_string()).unwrap();

    let _result = app_state
        .db_client
        .save_resume(user_id, &body.file_path, body.analysis_result)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    return Ok(Json(Response {
        message: "Resume uploaded successfully".to_string(),
        status: "success",
    }));
}

pub async fn get_users(
    Query(query_params): Query<RequestQueryDto>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<impl IntoResponse, HttpError> {
    query_params
        .validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let page = query_params.page.unwrap_or(1);
    let limit = query_params.limit.unwrap_or(10);

    let users = app_state
        .db_client
        .get_users(page as u32, limit)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user_count = app_state
        .db_client
        .get_user_count()
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let response = UserListResponseDto {
        status: "success".to_string(),
        users: FilterUserDto::filter_users(&users),
        results: user_count,
    };

    Ok(Json(response))
}

pub async fn update_user_name(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    Json(body): Json<FilterUserDto>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let user = &user.user;

    let user_id = Uuid::parse_str(&user.id.to_string()).unwrap();

    let result = app_state
        .db_client
        .update_user_name(user_id.clone(), &body.name)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let filtered_user = FilterUserDto::filter_user(&result);

    let response = UserResponseDto {
        data: UserData {
            user: filtered_user,
        },
        status: "success".to_string(),
    };

    Ok(Json(response))
}

pub async fn update_user_role(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    Json(body): Json<RoleUpdateDto>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let user = &user.user;

    let user_id = Uuid::parse_str(&user.id.to_string()).unwrap();

    let result = app_state
        .db_client
        .update_user_role(user_id.clone(), body.role)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let filtered_user = FilterUserDto::filter_user(&result);

    let response = UserResponseDto {
        data: UserData {
            user: filtered_user,
        },
        status: "success".to_string(),
    };

    Ok(Json(response))
}

pub async fn update_user_password(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    Json(body): Json<UserPassUpdateDto>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let user = &user.user;

    let user_id = Uuid::parse_str(&user.id.to_string()).unwrap();

    let result = app_state
        .db_client
        .get_user(Some(user_id.clone()), None, None, None)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user = result.ok_or(HttpError::unauthorized(
        ErrorMessage::InvalidToken.to_string(),
    ))?;

    let password_matched = password::compare(&body.old_password, &user.password)
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    if !password_matched {
        return Err(HttpError::bad_request(
            "Old password is incorrect!".to_string(),
        ));
    }

    let hash_password =
        password::hash(&body.new_password).map_err(|e| HttpError::server_error(e.to_string()))?;

    app_state
        .db_client
        .update_user_password(user_id.clone(), hash_password)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let response = Response {
        message: "Password updated successfully".to_string(),
        status: "success",
    };

    Ok(Json(response))
}
