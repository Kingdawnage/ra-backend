use std::sync::Arc;

use axum::{extract::{Multipart, Path, Query}, middleware::from_fn, response::IntoResponse, routing::{get, post, put}, Extension, Json, Router};
use bytes::Bytes;
use tokio::fs;
use uuid::Uuid;
use validator::Validate;

use crate::{models::users::UserRole, services::{database::UserActions, middleware::{role_check, JWTAuthMiddleware}, nlp::call_nlp_service}, utils::{dtos::{FilterResumeDto, FilterUserDto, RequestQueryDto, Response, ResumeResponseDto, RoleUpdateDto, UserData, UserListResponseDto, ResumeListResponseDto, UserPassUpdateDto, ResumeData, UserResponseDto}, error::{ErrorMessage, HttpError}, password}, AppState};

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
        .route("/{user_id}/resume/{resume_id}", get(get_resume).delete(delete_resume))
        .route("/{user_id}/resumes", get(get_resumes))
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

pub async fn upload_resume(
    Path(user_id): Path<Uuid>,
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, HttpError> {
    if user.user.id != user_id {
        return Err(HttpError::unauthorized(ErrorMessage::InvalidToken.to_string()));
    }
    let upload_dir = "./uploads/temp";
    fs::create_dir_all(upload_dir).await.map_err(|e| HttpError::server_error(e.to_string()))?;

    let user_id = &user.user.id;
    let mut analysis_result = None;

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

            match call_nlp_service(&app_state.http_client, &file_path, &file_name).await{
                Ok(result) => {
                    analysis_result = Some(result);
                    println!("Recieved analysis result: {:?}", analysis_result);
                }
                Err(e) => {
                    println!("Error calling NLP service: {:?}", e);
                }
            }

            let _result = app_state
                .db_client
                .save_resume(*user_id, &file_path, analysis_result.clone())
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?;
        }

        Ok(Json(Response {
            message: "Resume uploaded successfully".to_string(),
            status: "success",
        }))
}

pub async fn delete_resume(
    Path((user_id, resume_id)): Path<(Uuid, Uuid)>,
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {
    if user.user.id != user_id {
        return Err(HttpError::unauthorized(ErrorMessage::PermissionDenied.to_string()));
    }

    let user_id = &user.user.id;
    let resume = app_state
        .db_client
        .get_resume(Some(*user_id), Some(resume_id))
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let _result = app_state
        .db_client
        .delete_resume(Some(*user_id), Some(resume_id))
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let file_path = resume.as_ref().unwrap().file_path.clone();

    if let Err(e) = fs::remove_file(&file_path).await {
        println!("Warning: Could not delete file {}: {}", &file_path, e);
    }

    Ok(Json(Response {
        message: "Resume deleted successfully".to_string(),
        status: "success",
    }))
}

pub async fn get_resume(
    Path((user_id, resume_id)): Path<(Uuid, Uuid)>,
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {
    if user.user.id != user_id {
        return Err(HttpError::unauthorized(ErrorMessage::PermissionDenied.to_string()));
    }

    let user_id = &user.user.id;

    let resume = app_state
        .db_client
        .get_resume(Some(*user_id), Some(resume_id))
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let filtered_resume = FilterResumeDto::filter_resume(resume.as_ref().unwrap());

    let response = ResumeResponseDto {
        status: "success".to_string(),
        data: ResumeData {resume: filtered_resume}
    };
    Ok(Json(response))
}

pub async fn get_resumes(
    Query(query_params): Query<RequestQueryDto>,
    Path(user_id): Path<Uuid>,
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {
    if user.user.id != user_id {
        return Err(HttpError::unauthorized(ErrorMessage::PermissionDenied.to_string()));
    }

    query_params
        .validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let page = query_params.page.unwrap_or(1);
    let limit = query_params.limit.unwrap_or(10);

    let user_id = &user.user.id;

    let resumes = app_state
        .db_client
        .get_resumes(user_id.clone(), page as u32, limit)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let response = ResumeListResponseDto {
        status: "success".to_string(),
        resumes: FilterResumeDto::filter_resumes(&resumes),
        results: resumes.len() as i64,
    };
    Ok(Json(response))
}
