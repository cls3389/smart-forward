# 超小体积但有完整功能的智能网络转发器 Docker 镜像
# 使用 Alpine Linux 作为基础镜像，体积小但功能完整
FROM rust:1.88-alpine AS builder

# 设置工作目录
WORKDIR /app

# 安装构建依赖
RUN apk add --no-cache \
    pkgconfig \
    openssl-dev \
    musl-dev \
    ca-certificates

# 设置环境变量
ENV CARGO_TERM_COLOR=always
ENV RUST_BACKTRACE=1
ENV RUSTFLAGS="-C target-cpu=native -C link-arg=-s"

# 复制 Cargo 文件
COPY Cargo.toml Cargo.lock ./

# 创建虚拟 main.rs 用于依赖预编译
RUN mkdir src && echo "fn main() {}" > src/main.rs

# 预编译依赖
RUN cargo build --release

# 复制源代码
COPY src ./src

# 构建应用（静态链接）
RUN cargo build --release --verbose

# 运行时镜像 - 使用最小的 Alpine
FROM alpine:3.18

# 只安装必要的运行时依赖
RUN apk add --no-cache \
    ca-certificates \
    tzdata \
    && rm -rf /var/cache/apk/* /tmp/* /var/tmp/*

# 创建非 root 用户
RUN adduser -D -s /bin/false smartforward

# 设置工作目录
WORKDIR /app

# 从构建镜像复制二进制文件
COPY --from=builder /app/target/release/smart-forward /usr/local/bin/smart-forward

# 创建最小配置文件
RUN echo 'logging:' > /app/config.yaml && \
    echo '  level: "info"' >> /app/config.yaml && \
    echo '  format: "text"' >> /app/config.yaml && \
    echo 'network:' >> /app/config.yaml && \
    echo '  listen_addr: "0.0.0.0"' >> /app/config.yaml && \
    echo 'buffer_size: 8192' >> /app/config.yaml && \
    echo 'rules:' >> /app/config.yaml && \
    echo '  - name: "HTTPS"' >> /app/config.yaml && \
    echo '    listen_port: 443' >> /app/config.yaml && \
    echo '    protocol: "tcp"' >> /app/config.yaml && \
    echo '    targets:' >> /app/config.yaml && \
    echo '      - "example.com:443"' >> /app/config.yaml

# 创建日志目录
RUN mkdir -p /app/logs && chown smartforward:smartforward /app/logs

# 切换到非 root 用户
USER smartforward

# 暴露端口
EXPOSE 443 99 6690 999

# 设置环境变量
ENV RUST_LOG=info
ENV TZ=Asia/Shanghai

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD /usr/local/bin/smart-forward --validate-config || exit 1

# 启动命令
CMD ["/usr/local/bin/smart-forward", "--config", "/app/config.yaml"]
