//! 知识库单元测试

use nexus::application::kb::KnowledgeBase;

/// 测试知识库
#[tokio::test]
async fn test_knowledge_base() {
    let kb = KnowledgeBase::new();
    
    // 测试查询（空知识库会返回示例文档）
    let results = kb.query("test query").await;
    // 空知识库会返回示例文档，所以结果不为空
    assert!(!results.is_empty());
    
    // 测试大小
    assert_eq!(kb.size(), 0);
}
