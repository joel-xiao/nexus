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

```typescript
// 成功响应
{ status: "ok", data?: any, message?: string }

// 错误响应
{ status: "error", message: string }
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
  tasks: string[];
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
```


## API 接口清单

### 1. 健康检查
- `GET /health` - 服务健康状态
- `GET /ready` - 服务就绪状态

### 2. 模型调用
- `POST /api/invoke`
  ```json
  {
    "input": "你好",
    "adapter": "openai",
    "user_id": "user123"
  }
  ```

### 3. 适配器管理
- `GET /api/config/adapters` - 获取所有适配器
- `GET /api/config/adapters/{name}` - 获取单个适配器
- `DELETE /api/config/adapters/{name}` - 删除适配器
- `PUT /api/config/reload/adapter` - 添加/更新适配器
  ```json
  {
    "name": "openai",
    "api_key": "sk-...",
    "model": "gpt-4",
    "base_url": "https://api.openai.com/v1",
    "enabled": true
  }
  ```
- `GET /api/config/adapters/{name}/billing` - 获取计费统计

### 4. 路由规则管理
- `GET /api/config/routing/rules` - 获取所有路由规则
- `POST /api/config/routing/rules` - 创建路由规则
- `GET /api/config/routing/rules/{name}` - 获取单个规则
- `PUT /api/config/routing/rules/{name}` - 更新路由规则
- `DELETE /api/config/routing/rules/{name}` - 删除路由规则

### 5. 功能标志管理
- `GET /api/config/flags` - 获取所有功能标志
- `POST /api/config/flags` - 创建功能标志
- `GET /api/config/flags/{name}` - 获取单个功能标志
- `PUT /api/config/flags/{name}` - 更新功能标志
- `DELETE /api/config/flags/{name}` - 删除功能标志
- `GET /api/config/flags/{name}/check` - 检查功能标志状态

### 6. 提示词管理
- `GET /api/config/prompts` - 获取所有提示词
- `GET /api/config/prompts/{name}` - 获取单个提示词
- `DELETE /api/config/prompts/{name}` - 删除提示词
- `PUT /api/config/reload/prompt` - 添加/更新提示词

### 7. 配置导入导出
- `GET /api/config/export` - 导出配置
- `POST /api/config/import` - 导入配置


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
  // 模型调用
  async invokeModel(request: InvokeRequest): Promise<InvokeResponse> {
    return apiClient.post('/api/invoke', request);
  },

  // 适配器管理
  async listAdapters() {
    return apiClient.get('/api/config/adapters');
  },

  async reloadAdapter(config: ReloadAdapterRequest) {
    return apiClient.put('/api/config/reload/adapter', config);
  },

  // 路由规则
  async listRoutingRules() {
    return apiClient.get('/api/config/routing/rules');
  },

  async createRoutingRule(rule: CreateRuleRequest) {
    return apiClient.post('/api/config/routing/rules', rule);
  },

  // 功能标志
  async listFlags() {
    return apiClient.get('/api/config/flags');
  },

  async createFlag(flag: CreateFlagRequest) {
    return apiClient.post('/api/config/flags', flag);
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

**文档版本**: 1.0.0
