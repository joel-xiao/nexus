use axum::{Json, Extension};
use serde::Serialize;
use std::sync::Arc;
use crate::state::AppState;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    /// 服务状态
    #[schema(example = "healthy")]
    pub status: String,
    /// 服务版本
    #[schema(example = "0.1.0")]
    pub version: String,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct ReadinessResponse {
    /// 是否就绪
    #[schema(example = true)]
    pub ready: bool,
    /// 各项检查结果
    pub checks: HealthChecks,
}

#[derive(Serialize, ToSchema)]
pub struct HealthChecks {
    /// Redis 状态
    #[schema(example = "ok")]
    pub redis: String,
    /// 适配器状态
    #[schema(example = "ok (1 adapters)")]
    pub adapters: String,
    /// 任务队列状态
    #[schema(example = "ok")]
    pub task_queue: String,
}

// ===== 业务逻辑处理函数 =====

pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
    })
}

pub async fn readiness_handler(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<ReadinessResponse> {
    let redis_status = if std::env::var("REDIS_URL").is_ok() {
        "ok".to_string()
    } else {
        "not_configured".to_string()
    };

    let adapter_count = state.adapter_registry.read().await.list().await.len();
    let adapter_status = if adapter_count > 0 {
        format!("ok ({} adapters)", adapter_count)
    } else {
        "warning (no adapters)".to_string()
    };

    let task_queue_status = "ok".to_string();
    let all_ok = redis_status == "ok" && adapter_count > 0 && task_queue_status == "ok";

    Json(ReadinessResponse {
        ready: all_ok,
        checks: HealthChecks {
            redis: redis_status,
            adapters: adapter_status,
            task_queue: task_queue_status,
        },
    })
}
