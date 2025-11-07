.PHONY: build docker-build docker-push deploy rollback test clean help

# 变量
IMAGE_NAME ?= nexus
IMAGE_TAG ?= latest
DOCKER_REGISTRY ?= 
NAMESPACE ?= nexus

help: ## 显示帮助信息
	@echo "Nexus 部署 Makefile"
	@echo ""
	@echo "可用命令:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-15s %s\n", $$1, $$2}'

build: ## 构建 Rust 项目
	cargo build --release

docker-build: ## 构建 Docker 镜像
	docker build -t $(IMAGE_NAME):$(IMAGE_TAG) -f Dockerfile .
	@echo "✅ Docker 镜像构建完成: $(IMAGE_NAME):$(IMAGE_TAG)"

docker-push: docker-build ## 推送 Docker 镜像到 registry
ifdef DOCKER_REGISTRY
	@echo "📤 推送镜像到 $(DOCKER_REGISTRY)..."
	docker tag $(IMAGE_NAME):$(IMAGE_TAG) $(DOCKER_REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG)
	docker push $(DOCKER_REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG)
else
	@echo "⚠️  未设置 DOCKER_REGISTRY，跳过推送"
endif

docker-compose-up: ## 使用 docker-compose 启动服务
	docker-compose up -d

docker-compose-down: ## 停止 docker-compose 服务
	docker-compose down

docker-compose-logs: ## 查看 docker-compose 日志
	docker-compose logs -f

deploy: docker-push ## 部署到 Kubernetes
	@echo "🚀 部署到 Kubernetes..."
	kubectl apply -f deploy/k8s/namespace.yaml
	kubectl apply -f deploy/k8s/configmap.yaml
	kubectl apply -f deploy/k8s/redis-deployment.yaml
	kubectl apply -f deploy/k8s/deployment.yaml
	kubectl apply -f deploy/k8s/hpa.yaml
	kubectl rollout status deployment/nexus -n $(NAMESPACE) --timeout=300s
	@echo "✅ 部署完成"

rollback: ## 回滚到上一个版本
	@echo "🔄 回滚部署..."
	kubectl rollout undo deployment/nexus -n $(NAMESPACE)
	kubectl rollout status deployment/nexus -n $(NAMESPACE) --timeout=300s
	@echo "✅ 回滚完成"

canary: ## 金丝雀部署（需要提供 VERSION 和 PERCENTAGE）
	@if [ -z "$(VERSION)" ] || [ -z "$(PERCENTAGE)" ]; then \
		echo "❌ 用法: make canary VERSION=v1.2.0 PERCENTAGE=10"; \
		exit 1; \
	fi
	@echo "🪶 金丝雀部署版本 $(VERSION) ($(PERCENTAGE)% 流量)..."
	@./deploy/canary-deploy.sh $(VERSION) $(PERCENTAGE)

test: ## 运行测试
	./scripts/test/run_tests.sh

lint: ## 代码检查
	cargo clippy -- -D warnings

clean: ## 清理构建产物
	cargo clean
	docker system prune -f

status: ## 查看部署状态
	@echo "📊 Kubernetes 部署状态:"
	@kubectl get pods -n $(NAMESPACE)
	@echo ""
	@echo "📊 服务状态:"
	@kubectl get svc -n $(NAMESPACE)
	@echo ""
	@echo "📊 HPA 状态:"
	@kubectl get hpa -n $(NAMESPACE)

logs: ## 查看日志
	kubectl logs -f deployment/nexus -n $(NAMESPACE)

metrics: ## 查看 Prometheus metrics
	@echo "📊 访问 Prometheus metrics:"
	@echo "http://localhost:9090"
	@echo ""
	@echo "或者直接查询:"
	@kubectl port-forward svc/nexus-metrics 3000:3000 -n $(NAMESPACE) &

