use crate::common::helpers::{get_test_mode, TestMode};

pub async fn wait(ms: u64) {
    tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await;
}

pub async fn wait_for_ready() {
    let wait_time = match get_test_mode() {
        TestMode::Mock => 100,
        TestMode::Real => 1000,
    };
    wait(wait_time).await;
}

pub fn check_env_var(var_name: &str) -> bool {
    std::env::var(var_name).is_ok()
}

pub fn check_test_environment() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    match get_test_mode() {
        TestMode::Real => {
            if !check_env_var("NEXUS_TEST_ADAPTER_NAME") {
                errors.push("NEXUS_TEST_ADAPTER_NAME not set".to_string());
            }
            if !check_env_var("NEXUS_TEST_API_KEY") {
                errors.push("NEXUS_TEST_API_KEY not set".to_string());
            }
        }
        TestMode::Mock => {}
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
