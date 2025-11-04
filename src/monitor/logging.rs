use tracing_subscriber::{
    fmt::{self, MakeWriter},
    EnvFilter, Registry,
};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tokio::sync::mpsc;
use tokio::io::AsyncWriteExt;

/// 异步日志写入器 - 将日志写入到通道，由后台任务异步处理
pub struct AsyncLogWriter {
    sender: mpsc::UnboundedSender<Vec<u8>>,
}

impl AsyncLogWriter {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<Vec<u8>>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (Self { sender: tx }, rx)
    }
    
    /// 启动后台日志写入任务
    pub fn spawn_writer_task(mut receiver: mpsc::UnboundedReceiver<Vec<u8>>) {
        tokio::spawn(async move {
            // 尝试打开日志文件，如果失败则使用 stdout
            let mut file = tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("nexus.log")
                .await
                .ok();
            
            while let Some(bytes) = receiver.recv().await {
                if let Some(ref mut f) = file {
                    let _ = f.write_all(&bytes).await;
                    let _ = f.flush().await;
                } else {
                    // 回退到 stdout
                    let _ = tokio::io::stdout().write_all(&bytes).await;
                }
            }
        });
    }
}

impl<'a> MakeWriter<'a> for AsyncLogWriter {
    type Writer = AsyncLogWriterImpl;

    fn make_writer(&'a self) -> Self::Writer {
        AsyncLogWriterImpl {
            sender: self.sender.clone(),
            buffer: Vec::new(),
        }
    }
}

/// 实际的日志写入实现
pub struct AsyncLogWriterImpl {
    sender: mpsc::UnboundedSender<Vec<u8>>,
    buffer: Vec<u8>,
}

impl std::io::Write for AsyncLogWriterImpl {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        if !self.buffer.is_empty() {
            let _ = self.sender.send(std::mem::take(&mut self.buffer));
        }
        Ok(())
    }
}

/// 初始化结构化 JSON 日志和异步写入
pub fn init_logging() -> anyhow::Result<()> {
    // 从环境变量读取日志级别，默认 INFO
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    // 创建异步日志写入器
    let (writer, receiver) = AsyncLogWriter::new();
    
    // 启动后台写入任务
    AsyncLogWriter::spawn_writer_task(receiver);
    
    // 配置 JSON 格式化层（用于结构化日志）
    let json_layer = fmt::layer()
        .json() // 使用 JSON 格式
        .with_writer(writer)
        .with_target(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true);
    
    // 配置普通格式化层（用于开发环境的可读性）
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true);
    
    // 根据环境变量决定使用哪个格式
    let use_json = std::env::var("LOG_FORMAT")
        .unwrap_or_else(|_| "json".to_string())
        .to_lowercase() == "json";
    
    let registry = Registry::default()
        .with(env_filter);
    
    if use_json {
        registry
            .with(json_layer)
            .init();
    } else {
        registry
            .with(fmt_layer)
            .init();
    }
    
    tracing::info!("Logging initialized with format: {}", if use_json { "JSON" } else { "Pretty" });
    
    Ok(())
}

/// 初始化 Jaeger tracing
/// 注意：需要确保 Jaeger 和 OpenTelemetry Collector 正在运行
pub fn init_tracing(_service_name: &str) -> anyhow::Result<()> {
    let jaeger_endpoint = std::env::var("JAEGER_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());
    
    tracing::info!(
        "Jaeger tracing will be initialized (endpoint: {}). \
        Make sure Jaeger/OTLP collector is running. \
        Tracing data will be sent via OpenTelemetry protocol.",
        jaeger_endpoint
    );
    
    // 简化实现：实际使用时需要根据 opentelemetry 版本调整
    // 这里提供一个基础框架，具体实现需要根据实际依赖版本调整
    
    // 注意：完整的 Jaeger 集成需要：
    // 1. 运行 OpenTelemetry Collector 或 Jaeger
    // 2. 配置正确的 OTLP endpoint
    // 3. 根据实际的 opentelemetry crate 版本调整 API 调用
    
    Ok(())
}

