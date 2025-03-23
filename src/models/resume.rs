use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow, sqlx::Type)]
pub struct Resume {
    pub id: Uuid,
    pub user_id: Uuid,
    pub file_path: String,
    #[serde(rename = "analysisResult")]
    pub analysis_result: Option<Value>,
    #[serde(rename = "createdAt")]
    pub uploaded_at: Option<DateTime<Utc>>,
}