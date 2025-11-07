use axum::{
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
    BoxError,
};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, warn};

/// 全局错误处理中间件
pub struct ErrorHandler {
    #[allow(dead_code)]
    metrics: Option<Arc<crate::monitor::prometheus::PrometheusMetrics>>,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self { metrics: None }
    }

    pub fn with_metrics(metrics: Arc<crate::monitor::prometheus::PrometheusMetrics>) -> Self {
        Self {
            metrics: Some(metrics),
        }
    }

    /// 处理请求中的错误
    pub async fn handle_error(
        err: BoxError,
        req: Request,
        metrics: Option<Arc<crate::monitor::prometheus::PrometheusMetrics>>,
    ) -> Response {
        let error_msg = err.to_string();
        let path = req.uri().path().to_string();

        error!(
            error = %error_msg,
            path = %path,
            "Request error occurred"
        );

        if metrics.is_some() {}

        if error_msg.contains("timeout") || error_msg.contains("deadline") {
            (
                StatusCode::REQUEST_TIMEOUT,
                axum::Json(json!({
                    "error": "Request timeout",
                    "message": "The request took too long to process",
                    "status": 408
                })),
            )
                .into_response()
        } else if error_msg.contains("not found") || error_msg.contains("404") {
            (
                StatusCode::NOT_FOUND,
                axum::Json(json!({
                    "error": "Not found",
                    "message": "The requested resource was not found",
                    "status": 404
                })),
            )
                .into_response()
        } else {
            let mut error_json = json!({
                "error": "Internal server error",
                "message": "An unexpected error occurred",
                "status": 500,
            });

            #[cfg(debug_assertions)]
            {
                error_json["details"] = json!(error_msg);
            }

            (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(error_json)).into_response()
        }
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// 统一错误类型
#[derive(Debug)]
pub enum AppError {
    Internal(String),
    NotFound(String),
    BadRequest(String),
    Timeout(String),
    Unauthorized(String),
}

impl AppError {
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_msg) = match self {
            AppError::Internal(msg) => {
                error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
            AppError::NotFound(msg) => {
                warn!("Not found: {}", msg);
                (StatusCode::NOT_FOUND, msg)
            }
            AppError::BadRequest(msg) => {
                warn!("Bad request: {}", msg);
                (StatusCode::BAD_REQUEST, msg)
            }
            AppError::Timeout(msg) => {
                warn!("Timeout: {}", msg);
                (StatusCode::REQUEST_TIMEOUT, msg)
            }
            AppError::Unauthorized(msg) => {
                warn!("Unauthorized: {}", msg);
                (StatusCode::UNAUTHORIZED, msg)
            }
        };

        (
            status,
            axum::Json(json!({
                "error": status.as_str(),
                "message": error_msg,
                "status": status.as_u16()
            })),
        )
            .into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        let msg = err.to_string();
        if msg.contains("not found") {
            Self::NotFound(msg)
        } else if msg.contains("timeout") || msg.contains("deadline") {
            Self::Timeout(msg)
        } else {
            Self::Internal(msg)
        }
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for AppError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::Internal(err.to_string())
    }
}
