# 智能网络转发器 Docker 镜像 - 使用Ubuntu确保兼容性
FROM rust:1.88-bookworm AS builder

WORKDIR /app

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 优化编译参数
ENV RUSTFLAGS="-C link-arg=-s"
ENV CARGO_TERM_COLOR=always

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 预构建依赖 (重要的缓存层)
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制源代码并构建
COPY src ./src
RUN cargo build --release && \
    strip target/release/smart-forward

# ===== 运行时镜像 - 使用Ubuntu确保兼容性 =====
FROM ubuntu:22.04

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    tzdata \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false smartforward

WORKDIR /app

# 复制二进制文件
COPY --from=builder /app/target/release/smart-forward /usr/local/bin/smart-forward

# 创建最小配置 - 使用可靠的外部服务测试
RUN printf 'logging:\n  level: "info"\n  format: "text"\nnetwork:\n  listen_addr: "0.0.0.0"\nbuffer_size: 8192\nrules:\n  - name: "HTTP_TEST"\n    listen_port: 8080\n    protocol: "tcp"\n    targets:\n      - "httpbin.org:80"\n  - name: "DNS_TEST"\n    listen_port: 9090\n    protocol: "tcp"\n    targets:\n      - "1.1.1.1:53"\n' > /app/config.yaml && \
    mkdir -p /app/logs && \
    chown -R smartforward:smartforward /app

USER smartforward

EXPOSE 443 99 6690 999

ENV RUST_LOG=info TZ=Asia/Shanghai

# 健康检查：检查进程是否运行
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD pgrep smart-forward > /dev/null || exit 1

CMD ["/usr/local/bin/smart-forward", "--config", "/app/config.yaml"]
