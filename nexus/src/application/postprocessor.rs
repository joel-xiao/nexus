use crate::monitor::audit::AuditLog;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// 安全地截取 UTF-8 字符串到指定的字符数（不是字节数）
/// 如果字符串被截取，会在末尾添加 "..."
fn truncate_str_safe(text: &str, max_chars: usize) -> String {
    let char_count = text.chars().count();
    if char_count <= max_chars {
        text.to_string()
    } else {
        format!("{}...", text.chars().take(max_chars).collect::<String>())
    }
}

fn truncate_str_start_safe(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect::<String>()
}

fn truncate_str_end_safe(text: &str, max_chars: usize) -> String {
    let char_count = text.chars().count();
    if char_count <= max_chars {
        text.to_string()
    } else {
        text.chars()
            .skip(char_count - max_chars)
            .collect::<String>()
    }
}

#[derive(Clone, Debug)]
pub struct ProcessingContext {
    pub request_id: String,
    pub user_id: Option<String>,
    pub adapter_name: String,
    pub original_input: String,
    pub processed_input: Option<String>,
    pub original_output: String,
    pub processed_output: Option<String>,
    pub metadata: serde_json::Value,
    pub start_time: std::time::Instant,
}

impl ProcessingContext {
    pub fn new(user_id: Option<String>, adapter_name: String, input: String) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            adapter_name,
            original_input: input.clone(),
            processed_input: None,
            original_output: String::new(),
            processed_output: None,
            metadata: serde_json::json!({}),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn with_output(mut self, output: String) -> Self {
        self.original_output = output.clone();
        self.processed_output = Some(output);
        self
    }

    pub fn set_metadata(&mut self, key: &str, value: serde_json::Value) {
        self.metadata[key] = value;
    }
}

#[async_trait]
pub trait Postprocessor: Send + Sync {
    fn name(&self) -> &str;

    fn priority(&self) -> u32 {
        100
    }

    async fn pre_process(&self, context: &mut ProcessingContext) -> anyhow::Result<()> {
        let _ = context;
        Ok(())
    }

    async fn post_process(&self, context: &mut ProcessingContext) -> anyhow::Result<()> {
        let _ = context;
        Ok(())
    }
}

pub struct AuditPostprocessor {
    audit_log: Arc<AuditLog>,
}

impl AuditPostprocessor {
    pub fn new(audit_log: Arc<AuditLog>) -> Self {
        Self { audit_log }
    }
}

#[async_trait]
impl Postprocessor for AuditPostprocessor {
    fn name(&self) -> &str {
        "audit"
    }

    fn priority(&self) -> u32 {
        10
    }

    async fn pre_process(&self, context: &mut ProcessingContext) -> anyhow::Result<()> {
        debug!("Audit: Pre-processing request {}", context.request_id);

        self.audit_log.log_action(
            "request_started",
            "invoke",
            &context.request_id,
            context.user_id.as_deref(),
            "pending",
            serde_json::json!({
                "adapter": context.adapter_name,
                "input_length": context.original_input.len(),
                "input_preview": truncate_str_safe(&context.original_input, 100),
            }),
        );

        Ok(())
    }

    async fn post_process(&self, context: &mut ProcessingContext) -> anyhow::Result<()> {
        let duration = context.start_time.elapsed().as_secs_f64();
        let output = context
            .processed_output
            .as_ref()
            .unwrap_or(&context.original_output);

        debug!("Audit: Post-processing request {}", context.request_id);

        self.audit_log.log_action(
            "model_call",
            "invoke",
            &context.request_id,
            context.user_id.as_deref(),
            "completed",
            serde_json::json!({
                "adapter": context.adapter_name,
                "duration_seconds": duration,
                "output_length": output.len(),
            }),
        );

        self.audit_log.log_action(
            "response_completed",
            "invoke",
            &context.request_id,
            context.user_id.as_deref(),
            "success",
            serde_json::json!({
                "adapter": context.adapter_name,
                "duration_seconds": duration,
                "input_length": context.original_input.len(),
                "output_length": output.len(),
                "output_preview": truncate_str_safe(output, 200),
            }),
        );

        Ok(())
    }
}

/// PII 脱敏后处理器 - 对敏感信息进行处理
pub struct PiiRedactionPostprocessor {
    patterns: Vec<PiiPattern>,
    redaction_mode: RedactionMode,
}

#[derive(Clone, Debug)]
pub enum RedactionMode {
    Mask,
    Remove,
    Hash,
    Replace(String),
}

#[derive(Clone, Debug)]
pub enum PiiPattern {
    Email,
    Phone,
    IdCard,
    BankCard,
    IpAddress,
    Custom(String),
}

impl PiiRedactionPostprocessor {
    pub fn new(mode: RedactionMode) -> Self {
        let patterns = vec![
            PiiPattern::Email,
            PiiPattern::Phone,
            PiiPattern::IdCard,
            PiiPattern::BankCard,
            PiiPattern::IpAddress,
        ];

        Self {
            patterns,
            redaction_mode: mode,
        }
    }

    pub fn with_patterns(mut self, patterns: Vec<PiiPattern>) -> Self {
        self.patterns = patterns;
        self
    }

    fn redact_text(&self, text: &str) -> String {
        use regex::Regex;
        use std::sync::OnceLock;

        fn email_regex() -> &'static Regex {
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| {
                Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")
                    .expect("Failed to compile email regex")
            })
        }

        fn phone_regex() -> &'static Regex {
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| {
                Regex::new(r"1[3-9]\d{9}|\d{3,4}-\d{7,8}").expect("Failed to compile phone regex")
            })
        }

        fn idcard_regex() -> &'static Regex {
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| {
                Regex::new(r"\d{17}[\dXx]|\d{15}").expect("Failed to compile ID card regex")
            })
        }

        fn bankcard_regex() -> &'static Regex {
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| {
                Regex::new(r"\d{16,19}").expect("Failed to compile bank card regex")
            })
        }

        fn ip_regex() -> &'static Regex {
            static REGEX: OnceLock<Regex> = OnceLock::new();
            REGEX.get_or_init(|| {
                Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b")
                    .expect("Failed to compile IP address regex")
            })
        }

        let mut result = text.to_string();

        for pattern in &self.patterns {
            let regex = match pattern {
                PiiPattern::Email => email_regex(),
                PiiPattern::Phone => phone_regex(),
                PiiPattern::IdCard => idcard_regex(),
                PiiPattern::BankCard => bankcard_regex(),
                PiiPattern::IpAddress => ip_regex(),
                PiiPattern::Custom(re) => match Regex::new(re) {
                    Ok(custom_regex) => {
                        result = custom_regex
                            .replace_all(&result, |caps: &regex::Captures| {
                                caps.get(0)
                                    .map(|m| self.apply_redaction(m.as_str()))
                                    .unwrap_or_default()
                            })
                            .to_string();
                        continue;
                    }
                    Err(_) => continue,
                },
            };

            result = regex
                .replace_all(&result, |caps: &regex::Captures| {
                    caps.get(0)
                        .map(|m| self.apply_redaction(m.as_str()))
                        .unwrap_or_default()
                })
                .to_string();
        }

        result
    }

    fn apply_redaction(&self, text: &str) -> String {
        match &self.redaction_mode {
            RedactionMode::Mask => {
                let char_count = text.chars().count();
                if char_count <= 4 {
                    "*".repeat(char_count)
                } else {
                    let start = truncate_str_start_safe(text, 2);
                    let end = truncate_str_end_safe(text, 2);
                    format!("{}****{}", start, end)
                }
            }
            RedactionMode::Remove => "[已脱敏]".to_string(),
            RedactionMode::Hash => {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                text.hash(&mut hasher);
                format!("hash_{:x}", hasher.finish())
            }
            RedactionMode::Replace(s) => s.clone(),
        }
    }
}

#[async_trait]
impl Postprocessor for PiiRedactionPostprocessor {
    fn name(&self) -> &str {
        "pii_redaction"
    }

    fn priority(&self) -> u32 {
        50
    }

    async fn pre_process(&self, context: &mut ProcessingContext) -> anyhow::Result<()> {
        debug!(
            "PII Redaction: Processing input for request {}",
            context.request_id
        );

        let redacted_input = self.redact_text(&context.original_input);
        if redacted_input != context.original_input {
            context.processed_input = Some(redacted_input);
            context.set_metadata("pii_redacted_input", serde_json::json!(true));
            info!(
                "PII redaction applied to input for request {}",
                context.request_id
            );
        }

        Ok(())
    }

    async fn post_process(&self, context: &mut ProcessingContext) -> anyhow::Result<()> {
        let output = context
            .processed_output
            .as_ref()
            .unwrap_or(&context.original_output);

        debug!(
            "PII Redaction: Processing output for request {}",
            context.request_id
        );

        let redacted_output = self.redact_text(output);
        if redacted_output != *output {
            context.processed_output = Some(redacted_output);
            context.set_metadata("pii_redacted_output", serde_json::json!(true));
            info!(
                "PII redaction applied to output for request {}",
                context.request_id
            );
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormattedResponse {
    pub content: String,
    pub agents: Vec<AgentOutput>,
    pub metadata: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentOutput {
    pub agent_id: String,
    pub agent_name: String,
    pub content: String,
    pub confidence: Option<f64>,
    pub metadata: serde_json::Value,
}

pub struct FormattingPostprocessor {
    format_mode: FormatMode,
    merge_strategy: MergeStrategy,
}

#[derive(Clone, Debug)]
pub enum FormatMode {
    Plain,
    Json,
    Markdown,
    Structured,
}

#[derive(Clone, Debug)]
pub enum MergeStrategy {
    Concatenate,
    WeightedAverage,
    Vote,
    Best,
}

impl FormattingPostprocessor {
    pub fn new(format_mode: FormatMode, merge_strategy: MergeStrategy) -> Self {
        Self {
            format_mode,
            merge_strategy,
        }
    }

    fn format_output(&self, context: &ProcessingContext, agents: Vec<AgentOutput>) -> String {
        match &self.format_mode {
            FormatMode::Plain => self.merge_agent_outputs(&agents, &self.merge_strategy),
            FormatMode::Json => {
                let formatted = FormattedResponse {
                    content: self.merge_agent_outputs(&agents, &self.merge_strategy),
                    agents,
                    metadata: context.metadata.clone(),
                    timestamp: chrono::Utc::now(),
                };
                serde_json::to_string_pretty(&formatted).unwrap_or_default()
            }
            FormatMode::Markdown => {
                let mut markdown = String::from("# 响应结果\n\n**主内容：**\n\n");
                markdown.push_str(&self.merge_agent_outputs(&agents, &self.merge_strategy));
                markdown.push_str("\n\n");

                if agents.len() > 1 {
                    markdown.push_str("## Agent 输出详情\n\n");
                    for (idx, agent) in agents.iter().enumerate() {
                        markdown.push_str(&format!(
                            "### Agent {}: {}\n\n",
                            idx + 1,
                            agent.agent_name
                        ));
                        markdown.push_str(&agent.content);
                        markdown.push_str("\n\n");
                        if let Some(conf) = agent.confidence {
                            markdown.push_str(&format!("**置信度：** {:.2}%\n\n", conf * 100.0));
                        }
                    }
                }

                markdown
            }
            FormatMode::Structured => {
                let formatted = FormattedResponse {
                    content: self.merge_agent_outputs(&agents, &self.merge_strategy),
                    agents,
                    metadata: context.metadata.clone(),
                    timestamp: chrono::Utc::now(),
                };
                format!(
                    "结构化输出：\n主内容: {}\nAgent数量: {}\n时间戳: {}",
                    formatted.content,
                    formatted.agents.len(),
                    formatted.timestamp.format("%Y-%m-%d %H:%M:%S")
                )
            }
        }
    }

    fn merge_agent_outputs(&self, agents: &[AgentOutput], strategy: &MergeStrategy) -> String {
        if agents.is_empty() {
            return String::new();
        }

        if agents.len() == 1 {
            return agents[0].content.clone();
        }

        match strategy {
            MergeStrategy::Concatenate => agents
                .iter()
                .map(|a| format!("[{}]: {}", a.agent_name, a.content))
                .collect::<Vec<_>>()
                .join("\n\n"),
            MergeStrategy::WeightedAverage => {
                let total_weight: f64 = agents.iter().map(|a| a.confidence.unwrap_or(1.0)).sum();

                agents
                    .iter()
                    .map(|a| {
                        let weight = a.confidence.unwrap_or(1.0) / total_weight;
                        format!(
                            "[{} (权重: {:.2}%)]: {}",
                            a.agent_name,
                            weight * 100.0,
                            a.content
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n")
            }
            MergeStrategy::Vote => agents
                .iter()
                .enumerate()
                .map(|(idx, a)| format!("[投票 {} - {}]: {}", idx + 1, a.agent_name, a.content))
                .collect::<Vec<_>>()
                .join("\n\n"),
            MergeStrategy::Best => agents
                .iter()
                .max_by(|a, b| {
                    a.confidence
                        .unwrap_or(0.0)
                        .partial_cmp(&b.confidence.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|a| a.content.clone())
                .unwrap_or_default(),
        }
    }
}

#[async_trait]
impl Postprocessor for FormattingPostprocessor {
    fn name(&self) -> &str {
        "formatting"
    }

    fn priority(&self) -> u32 {
        100
    }

    async fn post_process(&self, context: &mut ProcessingContext) -> anyhow::Result<()> {
        debug!(
            "Formatting: Processing output for request {}",
            context.request_id
        );

        let output = context
            .processed_output
            .as_ref()
            .unwrap_or(&context.original_output);

        let agents: Vec<AgentOutput> = if let Some(agents_json) = context.metadata.get("agents") {
            serde_json::from_value(agents_json.clone()).unwrap_or_else(|_| {
                vec![AgentOutput {
                    agent_id: context.request_id.clone(),
                    agent_name: context.adapter_name.clone(),
                    content: output.clone(),
                    confidence: None,
                    metadata: serde_json::json!({}),
                }]
            })
        } else {
            vec![AgentOutput {
                agent_id: context.request_id.clone(),
                agent_name: context.adapter_name.clone(),
                content: output.clone(),
                confidence: None,
                metadata: serde_json::json!({}),
            }]
        };

        let formatted = self.format_output(context, agents);
        context.processed_output = Some(formatted);

        Ok(())
    }
}

/// 后处理器链管理器
pub struct PostprocessorChain {
    processors: Vec<Arc<dyn Postprocessor>>,
}

impl PostprocessorChain {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    pub fn add(mut self, processor: Arc<dyn Postprocessor>) -> Self {
        self.processors.push(processor);
        self.processors.sort_by_key(|p| p.priority());
        self
    }

    pub fn with_defaults(
        audit_log: Arc<AuditLog>,
        pii_mode: RedactionMode,
        format_mode: FormatMode,
        merge_strategy: MergeStrategy,
    ) -> Self {
        Self::new()
            .add(Arc::new(AuditPostprocessor::new(audit_log)))
            .add(Arc::new(PiiRedactionPostprocessor::new(pii_mode)))
            .add(Arc::new(FormattingPostprocessor::new(
                format_mode,
                merge_strategy,
            )))
    }

    pub async fn pre_process(&self, context: &mut ProcessingContext) -> anyhow::Result<()> {
        for processor in &self.processors {
            processor.pre_process(context).await?;
        }
        Ok(())
    }

    pub async fn post_process(&self, context: &mut ProcessingContext) -> anyhow::Result<()> {
        for processor in &self.processors {
            processor.post_process(context).await?;
        }
        Ok(())
    }

    pub fn list_processors(&self) -> Vec<String> {
        self.processors
            .iter()
            .map(|p| p.name().to_string())
            .collect()
    }
}

impl Default for PostprocessorChain {
    fn default() -> Self {
        Self::new()
    }
}
