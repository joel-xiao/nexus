use axum::{Json, Extension};
use crate::state::AppState;
use crate::domain::config::feature_flag::FeatureFlag;
use std::sync::Arc;
use super::common::{ok_response, ok_response_with_message, error_response};
use crate::routes::config::flags::{CreateFlagRequest, UpdateFlagRequest, FlagResponse};

// 辅助函数：解析标志状态
use crate::domain::config::feature_flag::FlagStatus;
fn parse_flag_status(status: &str, percentage: Option<u8>) -> FlagStatus {
    match status {
        "enabled" => FlagStatus::Enabled,
        "disabled" => FlagStatus::Disabled,
        "gradual" => FlagStatus::GradualRollout {
            percentage: percentage.unwrap_or(50),
        },
        _ => FlagStatus::Disabled,
    }
}

// ===== 业务逻辑处理函数 =====

pub async fn create_flag(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<CreateFlagRequest>,
) -> Json<FlagResponse> {
    let status = parse_flag_status(&payload.status, payload.percentage);
    let flag = FeatureFlag::new(payload.name.clone(), status);
    state.config_manager.feature_flags().register(flag).await;

    Json(FlagResponse {
        name: payload.name,
        enabled: true,
    })
}

pub async fn check_flag(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<FlagResponse> {
    let enabled = state.config_manager.feature_flags()
        .is_enabled(&name, None).await;

    Json(FlagResponse {
        name,
        enabled,
    })
}

pub async fn list_flags(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<Vec<FeatureFlag>> {
    let flags = state.config_manager.feature_flags().list().await;
    Json(flags)
}

pub async fn get_flag(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    match state.config_manager.feature_flags().get(&name).await {
        Some(flag) => ok_response(serde_json::json!({ "flag": flag })),
        None => error_response(&format!("Feature flag {} not found", name))
    }
}

pub async fn update_flag(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(payload): Json<UpdateFlagRequest>,
) -> Json<FlagResponse> {
    let status = parse_flag_status(&payload.status, payload.percentage);

    if let Some(mut flag) = state.config_manager.feature_flags().get(&name).await {
        flag.status = status;
        if let Some(desc) = payload.description {
            flag.description = desc;
        }
        state.config_manager.feature_flags().update(&name, flag).await;
        Json(FlagResponse {
            name,
            enabled: true,
        })
    } else {
        Json(FlagResponse {
            name,
            enabled: false,
        })
    }
}

pub async fn delete_flag(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    state.config_manager.feature_flags().delete(&name).await;
    ok_response_with_message(&format!("Feature flag {} deleted", name), serde_json::json!({}))
}
