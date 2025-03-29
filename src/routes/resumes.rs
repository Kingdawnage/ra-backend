use std::sync::Arc;

use axum::{extract::{Multipart, Path, Query}, response::IntoResponse, routing::{get, post}, Extension, Json, Router};
use bytes::Bytes;
use tokio::fs;
use uuid::Uuid;
use validator::Validate;

use crate::{services::{database::UserActions, middleware::JWTAuthMiddleware, nlp::call_nlp_service}, utils::{dtos::{FilterResumeDto, RequestQueryDto, Response, ResumeData, ResumeListResponseDto, ResumeResponseDto}, error::HttpError}, AppState};

pub fn resume_routes() -> Router {
    Router::new()
        .route("/resume", post(upload_resume))
        .route("/resume/{resume_id}", get(get_resume).delete(delete_resume))
        .route("/resumes", get(get_resumes))
}

pub async fn upload_resume(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, HttpError> {
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
    Path(resume_id): Path<Uuid>,
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {
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
    Path(resume_id): Path<Uuid>,
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {
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
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {
    query_params
        .validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let page = query_params.page.unwrap_or(1);
    let limit = query_params.limit.unwrap_or(10);

    let user_id = &user.user.id;

    let resumes = app_state
        .db_client
        .get_resumes(*user_id, page as u32, limit)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let response = ResumeListResponseDto {
        status: "success".to_string(),
        resumes: FilterResumeDto::filter_resumes(&resumes),
        results: resumes.len() as i64,
    };
    Ok(Json(response))
} 