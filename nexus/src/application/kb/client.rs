use tracing::{debug, info};

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
    }

    pub async fn query(&self, query: &str) -> Vec<String> {
        info!("Querying knowledge base: {}", query);

        if self.documents.is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let mut scored_results: Vec<(String, usize)> = Vec::new();

        for doc in &self.documents {
            let doc_lower = doc.to_lowercase();
            let mut score = 0;

            for word in &query_words {
                let count = doc_lower.matches(word).count();
                score += count;
            }

            if score > 0 {
                scored_results.push((doc.clone(), score));
            }
        }

        scored_results.sort_by(|a, b| b.1.cmp(&a.1));

        debug!("Found {} relevant documents", scored_results.len());

        scored_results
            .into_iter()
            .take(3)
            .map(|(doc, _)| doc)
            .collect()
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
