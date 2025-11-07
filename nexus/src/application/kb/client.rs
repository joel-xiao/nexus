use tracing::{info, debug};

pub struct KnowledgeBase {
    documents: Vec<String>,
    #[allow(dead_code)]
    embeddings: Vec<Vec<f32>>,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            embeddings: Vec::new(),
        }
    }

    pub async fn add_document(&mut self, content: String) {
        info!("Adding document to knowledge base");
        self.documents.push(content);
        // TODO: 实现向量嵌入
    }

    pub async fn query(&self, query: &str) -> Vec<String> {
        info!("Querying knowledge base: {}", query);
        
        // 简单的关键词匹配
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        
        for doc in &self.documents {
            if doc.to_lowercase().contains(&query_lower) {
                results.push(doc.clone());
            }
        }
        
        debug!("Found {} relevant documents", results.len());
        
        // 如果没有找到，返回示例
        if results.is_empty() {
            vec![
                "这是一个示例文档，展示了知识库的基本功能".to_string(),
                "您可以通过 add_document 方法添加更多文档".to_string(),
            ]
        } else {
            results.into_iter().take(3).collect()
        }
    }

    pub fn size(&self) -> usize {
        self.documents.len()
    }
}

impl Default for KnowledgeBase {
    fn default() -> Self {
        Self::new()
    }
}

