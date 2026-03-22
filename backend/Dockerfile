# LX-Proxy Backend Dockerfile
FROM rust:1.75-slim-bookworm as builder

WORKDIR /app

# 安装依赖
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# 复制 Cargo 文件
COPY Cargo.toml Cargo.lock* ./
COPY src ./src

# 编译
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

WORKDIR /app

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 从 builder 复制二进制文件
COPY --from=builder /app/target/release/lx_proxy_backend /app/lx_proxy_backend

# 复制环境配置示例
COPY .env.example /app/.env.example

# 暴露端口
EXPOSE 3000

# 运行
CMD ["./lx_proxy_backend"]
