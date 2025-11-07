/// llm-adapter - 统一的 LLM API 适配器框架
/// 
/// 提供统一接口调用多种 LLM 提供商（OpenAI、DeepSeek、Doubao 等）
/// 支持限流、计费、并发控制等功能

pub mod config;
pub mod registry;
pub mod factory;
pub mod generic;
pub mod wrapper;
pub mod providers;

// 基础设施模块
pub mod rate_limit;
pub mod billing;
pub mod guard;

// 重新导出核心类型
pub use config::AdapterConfig;
pub use registry::{Adapter, AdapterRegistry};
pub use factory::AdapterFactory;
pub use generic::{GenericAdapter, RequestConfig, AuthType};
pub use wrapper::WrappedAdapter;

// 重新导出基础设施
pub use rate_limit::RateLimiter;
pub use billing::BillingTracker;
pub use guard::ConcurrencyGuard;

