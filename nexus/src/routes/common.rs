use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct StandardResponse<T: Serialize> {
    #[schema(example = "ok")]
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    #[schema(example = "error")]
    pub status: String,
    #[schema(example = "Error message")]
    pub message: String,
}

pub fn ok_response<T: Serialize>(data: T) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "data": data
    }))
}

pub fn ok_response_with_message<T: Serialize>(message: &str, data: T) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "message": message,
        "data": data
    }))
}

pub fn error_response(message: &str) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "error",
        "message": message
    }))
}
