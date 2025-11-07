use prometheus::{Encoder, TextEncoder, Registry, Counter, Histogram, HistogramOpts, Gauge, Opts};

/// Prometheus 指标收集器
pub struct PrometheusMetrics {
    registry: Registry,
    
    // HTTP 相关指标
    pub http_requests_total: Counter,
    pub http_request_duration_seconds: Histogram,
    pub http_request_size_bytes: Histogram,
    pub http_response_size_bytes: Histogram,
    
    // 适配器相关指标
    pub adapter_calls_total: Counter,
    pub adapter_duration_seconds: Histogram,
    pub adapter_errors_total: Counter,
    
    // 任务队列相关指标
    pub task_queue_size: Gauge,
    pub tasks_processed_total: Counter,
    pub tasks_failed_total: Counter,
    
    // 系统相关指标
    pub cache_hits_total: Counter,
    pub cache_misses_total: Counter,
}

impl PrometheusMetrics {
    pub fn new() -> anyhow::Result<Self> {
        let registry = Registry::new();
        
        // HTTP 指标
        let http_requests_total = Counter::with_opts(
            Opts::new("http_requests_total", "Total number of HTTP requests")
                .namespace("nexus")
        )?;
        registry.register(Box::new(http_requests_total.clone()))?;
        
        let http_request_duration_seconds = Histogram::with_opts(
            HistogramOpts::new("http_request_duration_seconds", "HTTP request duration in seconds")
                .namespace("nexus")
                .const_label("method", "all"),
        )?;
        registry.register(Box::new(http_request_duration_seconds.clone()))?;
        
        let http_request_size_bytes = Histogram::with_opts(
            HistogramOpts::new("http_request_size_bytes", "HTTP request size in bytes")
                .namespace("nexus"),
        )?;
        registry.register(Box::new(http_request_size_bytes.clone()))?;
        
        let http_response_size_bytes = Histogram::with_opts(
            HistogramOpts::new("http_response_size_bytes", "HTTP response size in bytes")
                .namespace("nexus"),
        )?;
        registry.register(Box::new(http_response_size_bytes.clone()))?;
        
        // 适配器指标
        let adapter_calls_total = Counter::with_opts(
            Opts::new("adapter_calls_total", "Total number of adapter calls")
                .namespace("nexus"),
        )?;
        registry.register(Box::new(adapter_calls_total.clone()))?;
        
        let adapter_duration_seconds = Histogram::with_opts(
            HistogramOpts::new("adapter_duration_seconds", "Adapter call duration in seconds")
                .namespace("nexus"),
        )?;
        registry.register(Box::new(adapter_duration_seconds.clone()))?;
        
        let adapter_errors_total = Counter::with_opts(
            Opts::new("adapter_errors_total", "Total number of adapter errors")
                .namespace("nexus"),
        )?;
        registry.register(Box::new(adapter_errors_total.clone()))?;
        
        // 任务队列指标
        let task_queue_size = Gauge::with_opts(
            Opts::new("task_queue_size", "Current size of task queue")
                .namespace("nexus"),
        )?;
        registry.register(Box::new(task_queue_size.clone()))?;
        
        let tasks_processed_total = Counter::with_opts(
            Opts::new("tasks_processed_total", "Total number of processed tasks")
                .namespace("nexus"),
        )?;
        registry.register(Box::new(tasks_processed_total.clone()))?;
        
        let tasks_failed_total = Counter::with_opts(
            Opts::new("tasks_failed_total", "Total number of failed tasks")
                .namespace("nexus"),
        )?;
        registry.register(Box::new(tasks_failed_total.clone()))?;
        
        // 缓存指标
        let cache_hits_total = Counter::with_opts(
            Opts::new("cache_hits_total", "Total number of cache hits")
                .namespace("nexus"),
        )?;
        registry.register(Box::new(cache_hits_total.clone()))?;
        
        let cache_misses_total = Counter::with_opts(
            Opts::new("cache_misses_total", "Total number of cache misses")
                .namespace("nexus"),
        )?;
        registry.register(Box::new(cache_misses_total.clone()))?;
        
        Ok(Self {
            registry,
            http_requests_total,
            http_request_duration_seconds,
            http_request_size_bytes,
            http_response_size_bytes,
            adapter_calls_total,
            adapter_duration_seconds,
            adapter_errors_total,
            task_queue_size,
            tasks_processed_total,
            tasks_failed_total,
            cache_hits_total,
            cache_misses_total,
        })
    }
    
    /// 收集所有指标并格式化为 Prometheus 文本格式
    pub fn gather(&self) -> anyhow::Result<String> {
        let metric_families = self.registry.gather();
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
    
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

impl Default for PrometheusMetrics {
    fn default() -> Self {
        Self::new().expect("Failed to create Prometheus metrics")
    }
}

