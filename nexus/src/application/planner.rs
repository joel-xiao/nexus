use crate::infrastructure::messaging::mcp::message::McpMessage;
use tracing::{debug, info};

pub struct Planner {
    max_parallel_tasks: usize,
}

impl Planner {
    pub fn new() -> Self {
        Self {
            max_parallel_tasks: 5,
        }
    }

    pub async fn split_task(&self, input: &str) -> Vec<McpMessage> {
        info!("Splitting task: {}", input);

        let tasks = self.analyze_task(input);
        debug!("Generated {} subtasks", tasks.len());

        tasks
            .into_iter()
            .take(self.max_parallel_tasks)
            .map(|(task, ctx)| McpMessage::from_task(&task, ctx.as_deref()))
            .collect()
    }

    fn analyze_task(&self, input: &str) -> Vec<(String, Option<String>)> {
        let lower = input.to_lowercase();

        if lower.contains("分析") || lower.contains("分析") || lower.contains("analyze") {
            vec![
                ("数据收集".to_string(), Some(input.to_string())),
                ("数据分析".to_string(), Some(input.to_string())),
                ("结果总结".to_string(), Some(input.to_string())),
            ]
        } else if lower.contains("生成") || lower.contains("generate") || lower.contains("create")
        {
            vec![
                ("内容规划".to_string(), Some(input.to_string())),
                ("内容生成".to_string(), Some(input.to_string())),
                ("质量检查".to_string(), Some(input.to_string())),
            ]
        } else if lower.contains("搜索") || lower.contains("search") || lower.contains("查找") {
            vec![
                ("信息检索".to_string(), Some(input.to_string())),
                ("结果筛选".to_string(), Some(input.to_string())),
                ("答案总结".to_string(), Some(input.to_string())),
            ]
        } else {
            vec![
                ("预处理".to_string(), Some(input.to_string())),
                ("主处理".to_string(), Some(input.to_string())),
                ("后处理".to_string(), Some(input.to_string())),
            ]
        }
    }
}

impl Default for Planner {
    fn default() -> Self {
        Self::new()
    }
}
