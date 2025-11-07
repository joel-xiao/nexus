use nexus::application::kb::KnowledgeBase;

#[tokio::test]
async fn test_knowledge_base() {
    let kb = KnowledgeBase::new();

    let results = kb.query("test query").await;
    assert!(results.is_empty());

    assert_eq!(kb.size(), 0);
}
