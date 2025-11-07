//! 后处理器单元测试

use nexus::application::postprocessor::{
    PostprocessorChain, ProcessingContext, RedactionMode, FormatMode, MergeStrategy
};
use nexus::monitor::AuditLog;
use nexus::monitor::EventBus;
use std::sync::Arc;

/// 测试后处理器链
#[tokio::test]
async fn test_postprocessor_chain() {
    let event_bus = Arc::new(EventBus::new());
    let audit_log = Arc::new(AuditLog::new(event_bus));
    
    let chain = PostprocessorChain::with_defaults(
        audit_log,
        RedactionMode::Mask,
        FormatMode::Plain,
        MergeStrategy::Concatenate,
    );
    
    let mut context = ProcessingContext::new(
        Some("user1".to_string()),
        "mock".to_string(),
        "test input".to_string(),
    );
    
    // 测试预处理
    let result = chain.pre_process(&mut context).await;
    assert!(result.is_ok());
    
    // 设置输出
    context = context.with_output("test output".to_string());
    
    // 测试后处理
    let result = chain.post_process(&mut context).await;
    assert!(result.is_ok());
}
