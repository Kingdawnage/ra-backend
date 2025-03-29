use std::fs;

use reqwest::multipart;
use serde_json::Value;

use crate::utils::error::HttpError;

pub async fn call_nlp_service(
    http_client: &reqwest::Client,
    file_path: &str,
    file_name: &str,
) -> Result<Value, HttpError>{
    println!("Reading file from path: {}", file_path);
    let file_bytes = fs::read(file_path)
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let form = multipart::Form::new()
    .part(
        "file",
        multipart::Part::bytes(file_bytes)
        .file_name(file_name.to_string()),
    );

    println!("Sending request to NLP service at http://host.docker.internal:8000/analyze_resume/");
    let response = http_client
        .post("http://host.docker.internal:8000/analyze_resume/")
        // Uncomment when running locally
        // .post("http://host.docker.internal:8000/analyze_resume/")
        .multipart(form)
        .send()
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;
        
    let json: Value = response.json().await
        .map_err(|e| HttpError::server_error(e.to_string()))?;
    
    println!("Successfully parsed response");
    Ok(json)
}