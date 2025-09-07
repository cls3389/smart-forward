# 极致优化的智能网络转发器 Docker 镜像
# 目标: 最小体积 + 完整功能
FROM rust:1.88-alpine AS builder

WORKDIR /app

# 安装最小构建依赖
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static

# 优化编译参数 (修复LTO冲突)
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

# ===== 运行时镜像 - 使用最小 Alpine =====
FROM alpine:3.18

# 只安装绝对必要的运行时依赖
RUN apk add --no-cache ca-certificates tzdata && \
    rm -rf /var/cache/apk/* /tmp/* /var/tmp/* && \
    adduser -D -s /bin/false smartforward

WORKDIR /app

# 复制二进制文件
COPY --from=builder /app/target/release/smart-forward /usr/local/bin/smart-forward

# 创建最小配置 (单行优化)
RUN printf 'logging:\n  level: "info"\n  format: "text"\nnetwork:\n  listen_addr: "0.0.0.0"\nbuffer_size: 8192\nrules:\n  - name: "HTTPS"\n    listen_port: 443\n    protocol: "tcp"\n    targets:\n      - "example.com:443"\n' > /app/config.yaml && \
    mkdir -p /app/logs && \
    chown -R smartforward:smartforward /app

USER smartforward

EXPOSE 443 99 6690 999

ENV RUST_LOG=info TZ=Asia/Shanghai

# 轻量级健康检查
HEALTHCHECK --interval=30s --timeout=5s --start-period=3s --retries=2 \
    CMD /usr/local/bin/smart-forward --validate-config || exit 1

CMD ["/usr/local/bin/smart-forward", "--config", "/app/config.yaml"]
