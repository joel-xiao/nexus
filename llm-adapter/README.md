# llm-adapter

统一的 LLM API 适配器框架

## 功能特性

- 🎯 **统一接口** - 调用多种 LLM（OpenAI、DeepSeek、Doubao、千问、智谱等）
- 🚀 **限流控制** - 内置速率限制
- 💰 **计费跟踪** - 自动跟踪 token 使用和成本
- 🔧 **并发控制** - 控制并发请求数量
- ⚙️ **配置化** - 通过配置文件创建适配器
- 🔌 **易扩展** - 轻松添加新的 LLM 提供商

## 快速开始

### 安装

```toml
[dependencies]
llm-adapter = "0.1"
```

### 基本使用

```rust
use llm_adapter::{AdapterRegistry, AdapterConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建注册表
    let registry = AdapterRegistry::new();
    
    // 配置并注册适配器
    let config = AdapterConfig::new("openai".to_string())
        .with_api_key("sk-...".to_string())
        .with_model("gpt-3.5-turbo".to_string())
        .with_base_url("https://api.openai.com/v1".to_string());
    
    registry.register_from_config(config).await?;
    
    // 使用适配器
    let adapter = registry.get("openai").await.unwrap();
    let response = adapter.invoke("你好，介绍一下你自己").await?;
    
    println!("Response: {}", response);
    Ok(())
}
```

## 支持的提供商

- ✅ OpenAI (GPT-3.5, GPT-4)
- ✅ DeepSeek
- ✅ Doubao（豆包）
- ✅ Qianwen（千问）
- ✅ Zhipu（智谱）
- ✅ 通用 HTTP API (自定义)
- ✅ Mock (测试用)

## 高级功能

### 限流

```rust
let config = AdapterConfig::new("openai".to_string())
    .with_metadata(
        "rate_limit_rps".to_string(), 
        serde_json::json!(10)
    );
```

### 计费跟踪

```rust
let config = AdapterConfig::new("openai".to_string())
    .with_metadata(
        "input_price_per_1k".to_string(),
        serde_json::json!(0.0015)
    );
```

### 并发控制

```rust
let config = AdapterConfig::new("openai".to_string())
    .with_metadata(
        "max_concurrent".to_string(),
        serde_json::json!(5)
    );
```

## 架构

```
AdapterRegistry
    ↓
WrappedAdapter (限流 + 计费 + 并发控制)
    ↓
Adapter 实现 (OpenAI, DeepSeek, etc.)
    ↓
LLM API
```

## 测试

```bash
# 运行 llm-adapter 单元 + 集成测试
./scripts/test/run_tests.sh

# 带性能测试（如已创建）
./scripts/test/run_tests.sh --performance
```

## License

MIT

