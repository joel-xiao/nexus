# Nexus 前端 API 开发指南

## 📋 项目信息

- **后端项目**: Nexus - 多模型 LLM 统一网关
- **前端项目名称建议**: `nexus-web` 或 `nexus-dashboard`
- **API 基础地址**: `http://localhost:3000`
- **API 文档**: `http://localhost:3000/docs` (Swagger UI)
- **OpenAPI JSON**: `http://localhost:3000/api-docs/openapi.json`

## 快速开始

1. 启动后端: `cargo run`
2. 访问文档: http://localhost:3000/docs
3. 下载 OpenAPI: http://localhost:3000/api-docs/openapi.json

## 统一响应格式

### 标准响应格式

```typescript
{ status: "ok", data?: any, message?: string }

{ status: "error", message: string }
```

### 响应处理说明

1. **JSON 序列化**: 所有响应都通过 `serde_json` 自动序列化，确保控制字符（如 `\u0000-\u001F`）被正确转义
2. **字符编码**: 响应使用 UTF-8 编码，支持所有 Unicode 字符
3. **MCP 消息**: `tasks` 字段包含 MCP（Model Context Protocol）消息对象，而非字符串，避免双重序列化

### 特殊响应格式

以下接口**直接返回数据**，不包装在 `{ status, data }` 中：

1. **路由规则列表** (`GET /api/config/routing/rules`)
   ```typescript
   RoutingRule[]
   ```

2. **功能标志列表** (`GET /api/config/flags`)
   ```typescript
   FeatureFlag[]
   ```

3. **功能标志检查** (`GET /api/config/flags/{name}/check`)
   ```typescript
   { name: string, enabled: boolean }
   ```

## TypeScript 类型定义

```typescript
interface ApiResponse<T = any> {
  status: 'ok' | 'error';
  data?: T;
  message?: string;
}

interface InvokeRequest {
  input: string;
  adapter?: string;
  user_id?: string;
}

interface InvokeResponse {
  result: string;
  tasks: Array<{
    mcp_version: string;
    state: any;
    memory: any[];
    tools: any[];
    provenance: any[];
    meta: any;
  }>;  // MCP 消息对象数组，而非字符串
  adapter_used: string;
}

interface AdapterConfig {
  name: string;
  api_key?: string;
  model?: string;
  base_url?: string;
  enabled: boolean;
}

type RoutingStrategy = 'round_robin' | 'random' | 'weighted' | 'least_connections' | 'user_based' | 'hash_based';

interface RoutingRule {
  name: string;
  strategy: RoutingStrategy;
  models: Array<{
    model_name: string;
    adapter_name: string;
    weight: number;
    enabled: boolean;
  }>;
  priority?: number;
}

interface FeatureFlag {
  name: string;
  status: 'enabled' | 'disabled' | 'gradual';
  description?: string;
  percentage?: number;
}

interface PromptConfig {
  name: string;
  template: string;
  enabled: boolean;
}

interface BillingStats {
  total_tokens?: number;
  total_cost?: number;
  requests?: number;
}

interface HealthResponse {
  status: 'healthy';
}

interface ReadinessResponse {
  ready: boolean;
}


## API 接口清单

### 1. 健康检查

#### 1.1 健康检查
- **GET** `/health`
- **响应**: `{ status: "ok", data: { status: "healthy" } }`

#### 1.2 就绪检查
- **GET** `/ready`
- **响应**: `{ status: "ok", data: { ready: boolean } }`

### 2. 模型调用

#### 2.1 调用模型
- **POST** `/api/invoke`
- **请求体**:
  ```json
  {
    "input": "你好",
    "adapter": "openai",
    "user_id": "user123"
  }
  ```
- **响应**: 
  ```json
  { 
    "status": "ok", 
    "data": { 
      "result": "模型生成的响应内容",
      "tasks": [
        {
          "mcp_version": "1.0",
          "state": { "step": "pending", "task": "预处理" },
          "memory": [...],
          "tools": [...],
          "provenance": [],
          "meta": { "timestamp": "2025-11-07T06:05:27.461793+00:00" }
        }
      ],
      "adapter_used": "曦听"
    }
  }
  ```
  **注意**: `tasks` 字段现在返回 MCP 消息对象数组，而不是 JSON 字符串数组

### 3. 适配器管理

#### 3.1 获取适配器列表
- **GET** `/api/config/adapters`
- **响应**: `{ status: "ok", data: { adapters: AdapterConfig[] } }`

#### 3.2 获取单个适配器
- **GET** `/api/config/adapters/{name}`
- **响应**: `{ status: "ok", data: { adapter: AdapterConfig } }`

#### 3.3 创建/更新适配器
- **PUT** `/api/config/reload/adapter`
- **请求体**:
  ```json
  {
    "name": "openai",
    "api_key": "sk-...",
    "model": "gpt-4",
    "base_url": "https://api.openai.com/v1",
    "enabled": true
  }
  ```
- **响应**: `{ status: "ok", message: "Adapter {name} reloaded and registered" }`

#### 3.4 删除适配器
- **DELETE** `/api/config/adapters/{name}`
- **响应**: `{ status: "ok", message: "Adapter {name} deleted" }`

#### 3.5 获取适配器计费统计
- **GET** `/api/config/adapters/{name}/billing`
- **响应**: `{ status: "ok", data: { stats: BillingStats } }`

### 4. 路由规则管理

#### 4.1 获取路由规则列表
- **GET** `/api/config/routing/rules`
- **响应**: `RoutingRule[]` (直接返回数组，不包装)

#### 4.2 创建路由规则
- **POST** `/api/config/routing/rules`
- **请求体**: `RoutingRule`
- **响应**: `{ status: "ok", data: { rule: string } }`

#### 4.3 获取单个路由规则
- **GET** `/api/config/routing/rules/{name}`
- **响应**: `{ status: "ok", data: { rule: RoutingRule } }`

#### 4.4 更新路由规则
- **PUT** `/api/config/routing/rules/{name}`
- **请求体**: `RoutingRule` (不需要name字段)
- **响应**: `{ status: "ok", data: { rule: string } }`

#### 4.5 删除路由规则
- **DELETE** `/api/config/routing/rules/{name}`
- **响应**: `{ status: "ok", message: "Routing rule {name} deleted" }`

### 5. 功能标志管理

#### 5.1 获取功能标志列表
- **GET** `/api/config/flags`
- **响应**: `FeatureFlag[]` (直接返回数组，不包装)

#### 5.2 创建功能标志
- **POST** `/api/config/flags`
- **请求体**: `FeatureFlag`
- **响应**: `{ status: "ok", data: { name: string, enabled: boolean } }`

#### 5.3 获取单个功能标志
- **GET** `/api/config/flags/{name}`
- **响应**: `{ status: "ok", data: { flag: FeatureFlag } }`

#### 5.4 更新功能标志
- **PUT** `/api/config/flags/{name}`
- **请求体**: `{ status: string, description?: string, percentage?: number }` (不需要name字段)
- **响应**: `{ status: "ok", data: { name: string, enabled: boolean } }`

#### 5.5 删除功能标志
- **DELETE** `/api/config/flags/{name}`
- **响应**: `{ status: "ok", message: "Feature flag {name} deleted" }`

#### 5.6 检查功能标志状态
- **GET** `/api/config/flags/{name}/check`
- **响应**: `{ name: string, enabled: boolean }` (直接返回对象，不包装)

### 6. 提示词管理

#### 6.1 获取提示词列表
- **GET** `/api/config/prompts`
- **响应**: `{ status: "ok", data: { prompts: PromptConfig[] } }`

#### 6.2 获取单个提示词
- **GET** `/api/config/prompts/{name}`
- **响应**: `{ status: "ok", data: { prompt: PromptConfig } }`

#### 6.3 创建/更新提示词
- **PUT** `/api/config/reload/prompt`
- **请求体**:
  ```json
  {
    "name": "default_prompt",
    "template": "You are a helpful assistant. {{input}}",
    "enabled": true
  }
  ```
- **响应**: `{ status: "ok", message: "Prompt {name} reloaded" }`

#### 6.4 删除提示词
- **DELETE** `/api/config/prompts/{name}`
- **响应**: `{ status: "ok", message: "Prompt {name} deleted" }`

**注意**: 前端使用 `content` 字段，后端使用 `template` 字段。前端会自动转换。

### 7. 配置导入导出

#### 7.1 导出配置
- **GET** `/api/config/export`
- **响应**: `{ status: "ok", data: { adapters: [], routing_rules: [], flags: [], prompts: [] } }`

#### 7.2 导入配置
- **POST** `/api/config/import`
- **请求体**: `{ adapters: [], routing_rules: [], flags: [], prompts: [] }`
- **响应**: `{ status: "ok", message: "Configuration imported and adapters registered successfully" }`


## 代码示例

### API 客户端封装

```typescript
import axios from 'axios';

const API_BASE_URL = 'http://localhost:3000';

const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: { 'Content-Type': 'application/json' },
});

export const nexusApi = {
  // 健康检查
  async checkHealth(): Promise<ApiResponse<HealthResponse>> {
    return apiClient.get('/health');
  },

  async checkReady(): Promise<ApiResponse<ReadinessResponse>> {
    return apiClient.get('/ready');
  },

  // 模型调用
  async invokeModel(request: InvokeRequest): Promise<ApiResponse<InvokeResponse>> {
    return apiClient.post('/api/invoke', request);
  },

  // 适配器管理
  async listAdapters(): Promise<ApiResponse<{ adapters: AdapterConfig[] }>> {
    return apiClient.get('/api/config/adapters');
  },

  async getAdapter(name: string): Promise<ApiResponse<{ adapter: AdapterConfig }>> {
    return apiClient.get(`/api/config/adapters/${name}`);
  },

  async reloadAdapter(config: AdapterConfig): Promise<ApiResponse> {
    return apiClient.put('/api/config/reload/adapter', config);
  },

  async deleteAdapter(name: string): Promise<ApiResponse> {
    return apiClient.delete(`/api/config/adapters/${name}`);
  },

  async getAdapterBilling(name: string): Promise<ApiResponse<{ stats: BillingStats }>> {
    return apiClient.get(`/api/config/adapters/${name}/billing`);
  },

  // 路由规则
  async listRoutingRules(): Promise<RoutingRule[]> {
    return apiClient.get('/api/config/routing/rules');
  },

  async getRoutingRule(name: string): Promise<ApiResponse<{ rule: RoutingRule }>> {
    return apiClient.get(`/api/config/routing/rules/${name}`);
  },

  async createRoutingRule(rule: RoutingRule): Promise<ApiResponse<{ rule: string }>> {
    return apiClient.post('/api/config/routing/rules', rule);
  },

  async updateRoutingRule(name: string, rule: Omit<RoutingRule, 'name'>): Promise<ApiResponse<{ rule: string }>> {
    return apiClient.put(`/api/config/routing/rules/${name}`, rule);
  },

  async deleteRoutingRule(name: string): Promise<ApiResponse> {
    return apiClient.delete(`/api/config/routing/rules/${name}`);
  },

  // 功能标志
  async listFlags(): Promise<FeatureFlag[]> {
    return apiClient.get('/api/config/flags');
  },

  async getFlag(name: string): Promise<ApiResponse<{ flag: FeatureFlag }>> {
    return apiClient.get(`/api/config/flags/${name}`);
  },

  async createFlag(flag: FeatureFlag): Promise<ApiResponse<{ name: string; enabled: boolean }>> {
    return apiClient.post('/api/config/flags', flag);
  },

  async updateFlag(name: string, flag: Omit<FeatureFlag, 'name'>): Promise<ApiResponse<{ name: string; enabled: boolean }>> {
    return apiClient.put(`/api/config/flags/${name}`, flag);
  },

  async deleteFlag(name: string): Promise<ApiResponse> {
    return apiClient.delete(`/api/config/flags/${name}`);
  },

  async checkFlag(name: string): Promise<{ name: string; enabled: boolean }> {
    return apiClient.get(`/api/config/flags/${name}/check`);
  },

  // 提示词管理
  async listPrompts(): Promise<ApiResponse<{ prompts: PromptConfig[] }>> {
    return apiClient.get('/api/config/prompts');
  },

  async getPrompt(name: string): Promise<ApiResponse<{ prompt: PromptConfig }>> {
    return apiClient.get(`/api/config/prompts/${name}`);
  },

  async reloadPrompt(prompt: PromptConfig): Promise<ApiResponse> {
    return apiClient.put('/api/config/reload/prompt', prompt);
  },

  async deletePrompt(name: string): Promise<ApiResponse> {
    return apiClient.delete(`/api/config/prompts/${name}`);
  },

  // 配置导入导出
  async exportConfig(): Promise<ApiResponse> {
    return apiClient.get('/api/config/export');
  },

  async importConfig(config: any): Promise<ApiResponse> {
    return apiClient.post('/api/config/import', config);
  },
};
```

### React Hook 示例

```typescript
import { useState } from 'react';
import { nexusApi } from './api/client';

export function useInvoke() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const invoke = async (request: InvokeRequest) => {
    setLoading(true);
    setError(null);
    try {
      return await nexusApi.invokeModel(request);
    } catch (err: any) {
      setError(err.message);
      return null;
    } finally {
      setLoading(false);
    }
  };

  return { invoke, loading, error };
}
```


## 前端功能需求

### 1. 聊天界面
- 消息输入和发送
- 显示模型返回结果
- 适配器选择
- 加载状态

### 2. 适配器管理
- 列表展示
- 添加/编辑/删除适配器
- 启用/禁用切换
- 计费统计查看

### 3. 路由规则管理
- 创建/编辑路由规则
- 选择路由策略
- 配置模型权重
- 优先级设置

### 4. 功能标志管理
- 创建/编辑功能标志
- 启用/禁用/渐进式发布
- 状态检查

### 5. 提示词管理
- 创建/编辑提示词模板
- Handlebars 语法支持
- 模板预览

### 6. 配置管理
- 导入/导出配置文件
- 一键备份/恢复

### 7. 仪表盘
- 系统健康状态
- API 调用统计
- 适配器使用情况
- 计费统计图表

## 技术栈建议

### 推荐方案: React + TypeScript

```bash
npm create vite@latest nexus-web -- --template react-ts
cd nexus-web
npm install axios zustand react-router-dom antd recharts
```

**技术栈**:
- React 18 + TypeScript
- Ant Design (UI 组件库)
- Zustand (状态管理)
- Axios (HTTP 客户端)
- React Router (路由)
- Recharts (图表)

## 完整文档

更多详细信息请访问:
- **Swagger UI**: http://localhost:3000/docs
- **OpenAPI JSON**: http://localhost:3000/api-docs/openapi.json

所有接口的完整定义、请求/响应示例、参数说明都在 Swagger UI 中可以查看和测试。

---

## 重要说明

### 响应格式处理

1. **标准格式**: 大部分接口返回 `{ status: "ok", data: {...} }` 格式
2. **直接返回数组**: 路由规则列表和功能标志列表直接返回数组
3. **直接返回对象**: 功能标志检查接口直接返回 `{ name, enabled }` 对象

### 字段映射

- **提示词**: 前端使用 `content` 字段，后端使用 `template` 字段，前端会自动转换
- **适配器**: 返回的字段不包含 `metadata`，只包含前端需要的字段
- **提示词**: 返回的字段不包含 `metadata`，只包含前端需要的字段

### 错误处理

所有错误响应格式为：
```typescript
{ status: "error", message: "错误描述信息" }
```

常见 HTTP 状态码：
- `200`: 成功
- `400`: 请求参数错误
- `404`: 资源不存在
- `500`: 服务器内部错误

---

**文档版本**: 1.1.0  
**最后更新**: 2024  
**维护者**: 后端团队
