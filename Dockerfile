# 智能网络转发器 Docker 镜像
FROM rust:1.88-slim AS builder

# 设置工作目录
WORKDIR /app

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# 设置环境变量
ENV CARGO_TERM_COLOR=always
ENV RUST_BACKTRACE=1

# 复制 Cargo 文件
COPY Cargo.toml Cargo.lock ./

# 创建虚拟 main.rs 用于依赖预编译
RUN mkdir src && echo "fn main() {}" > src/main.rs

# 预编译依赖
RUN cargo build --release

# 复制源代码和配置文件
COPY src ./src
COPY config.yaml.example ./

# 构建应用
RUN cargo build --release --verbose

# 运行时镜像
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN useradd -r -s /bin/false smartforward

# 设置工作目录
WORKDIR /app

# 从构建镜像复制二进制文件和配置文件
COPY --from=builder /app/target/release/smart-forward /usr/local/bin/smart-forward
COPY --from=builder /app/config.yaml.example /app/config.yaml

# 创建日志目录
RUN mkdir -p /app/logs && chown smartforward:smartforward /app/logs

# 切换到非 root 用户
USER smartforward

# 暴露端口
EXPOSE 443 99 6690 999

# 设置环境变量
ENV RUST_LOG=info

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD /usr/local/bin/smart-forward --validate-config || exit 1

# 启动命令
CMD ["/usr/local/bin/smart-forward", "--config", "/app/config.yaml"]