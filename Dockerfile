# P17 高可用部署 - Dockerfile
# 多阶段构建优化镜像大小

# ========== 构建阶段 ==========
FROM rust:1.82-bookworm AS builder

WORKDIR /app

# 安装依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 复制 Cargo 配置
COPY backend/Cargo.toml backend/Cargo.lock ./

# 创建空的 src 目录以利用 Docker 缓存层
RUN mkdir src && echo "fn main() {}" > src/main.rs

# 下载依赖（利用缓存）
RUN cargo build --release && rm -rf src

# 复制源代码
COPY backend/src ./src
COPY backend/migrations ./migrations

# 编译（ release 模式优化性能）
RUN cargo build --release && ls -lh target/release/

# ========== 运行阶段 ==========
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -u 1000 -g root lxproxy

WORKDIR /app

# 复制编译好的二进制文件
COPY --from=builder /app/target/release/lx_proxy_backend /app/lx_proxy_backend

# 复制迁移文件
COPY --from=builder /app/migrations /app/migrations

# 设置权限
RUN chown -R lxproxy:root /app && \
    chmod +x /app/lx_proxy_backend

# 切换到非 root 用户（安全加固）
USER lxproxy

# 暴露端口
EXPOSE 8080

# 健康检查端点
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# 环境变量
ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=8080

# 启动命令
CMD ["/app/lx_proxy_backend"]
