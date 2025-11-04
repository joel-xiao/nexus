use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct McpMessage {
    pub mcp_version: String,
    pub state: serde_json::Value,
    pub memory: Vec<serde_json::Value>,
    pub tools: Vec<serde_json::Value>,
    pub provenance: Vec<serde_json::Value>,
    pub meta: serde_json::Value,
}

impl McpMessage {
    pub fn new(content: String) -> Self {
        Self {
            mcp_version: "1.0".to_string(),
            state: serde_json::json!({"step":"init"}),
            memory: vec![serde_json::json!({"role":"system","content":content})],
            tools: vec![],
            provenance: vec![],
            meta: serde_json::json!({}),
        }
    }

    pub fn from_task(task: &str, context: Option<&str>) -> Self {
        let mut memory = vec![serde_json::json!({"role":"system","content":task})];
        if let Some(ctx) = context {
            memory.push(serde_json::json!({"role":"user","content":ctx}));
        }
        Self {
            mcp_version: "1.0".to_string(),
            state: serde_json::json!({"step":"pending","task":task}),
            memory,
            tools: vec![],
            provenance: vec![],
            meta: serde_json::json!({"timestamp":chrono::Utc::now().to_rfc3339()}),
        }
    }

    pub fn add_tool(&mut self, tool: serde_json::Value) {
        self.tools.push(tool);
    }

    pub fn add_result(&mut self, result: &str) {
        self.memory.push(serde_json::json!({"role":"assistant","content":result}));
        self.state = serde_json::json!({"step":"completed"});
    }
}

