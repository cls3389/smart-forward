# 智能网络转发器 Docker 镜像 - 修复编译问题
FROM rust:1.88-bookworm AS builder

WORKDIR /app

# 安装构建依赖 + 静态链接库
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libssl3 \
    ca-certificates \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

# 添加musl目标支持（静态链接）
RUN rustup target add x86_64-unknown-linux-musl

# 静态链接优化编译参数（修复运行时问题）
ENV RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-s"
ENV CARGO_TERM_COLOR=always
ENV PKG_CONFIG_ALLOW_CROSS=1

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 预构建依赖（使用musl静态链接）
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl && \
    rm -rf src

# 复制源代码并构建（静态链接版本）
COPY src ./src
RUN cargo build --release --target x86_64-unknown-linux-musl && \
    strip target/x86_64-unknown-linux-musl/release/smart-forward

# ===== 运行时镜像 - 使用Alpine Linux =====
FROM alpine:3.18

# 安装运行时依赖（包含防火墙工具）
RUN apk add --no-cache \
    ca-certificates \
    tzdata \
    iptables \
    ip6tables \
    && adduser -D -s /bin/false smartforward

WORKDIR /app

# 复制静态链接的二进制文件
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/smart-forward /usr/local/bin/smart-forward

# 创建测试配置（支持内核态转发）
RUN printf 'logging:\n  level: "info"\n  format: "text"\nnetwork:\n  listen_addr: "0.0.0.0"\nbuffer_size: 8192\nrules:\n  - name: "HTTP_TEST"\n    listen_port: 8080\n    protocol: "tcp"\n    targets:\n      - "httpbin.org:80"\n  - name: "DNS_TEST"\n    listen_port: 8053\n    protocol: "tcp"\n    targets:\n      - "1.1.1.1:53"\n  - name: "KERNEL_TEST"\n    listen_port: 9090\n    protocol: "tcp"\n    targets:\n      - "example.com:80"\n' > /app/config.yaml && \
    mkdir -p /app/logs && \
    chown -R smartforward:smartforward /app && \
    chmod +x /usr/local/bin/smart-forward

USER smartforward

EXPOSE 443 99 6690 999

ENV RUST_LOG=info TZ=Asia/Shanghai

# 健康检查：检查进程是否运行
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD pgrep smart-forward > /dev/null || exit 1

CMD ["/usr/local/bin/smart-forward", "-c", "/app/config.yaml"]
