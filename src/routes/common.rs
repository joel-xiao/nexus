use axum::Json;
use serde::Serialize;

// ===== 通用响应辅助函数 =====

/// 创建成功响应
pub fn ok_response<T: Serialize>(data: T) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "data": data
    }))
}

/// 创建成功响应（带消息）
pub fn ok_response_with_message<T: Serialize>(message: &str, data: T) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "message": message,
        "data": data
    }))
}

/// 创建错误响应
pub fn error_response(message: &str) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "error",
        "message": message
    }))
}
