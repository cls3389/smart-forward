# 智能网络转发器 Docker 镜像
FROM rust:1.88-slim AS builder

# 设置工作目录
WORKDIR /app

# 安装构建依赖
RUN apt-get update && \
    apt-get install -y \
        pkg-config \
        libssl-dev \
        ca-certificates \
        build-essential && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# 设置环境变量
ENV CARGO_TERM_COLOR=always
ENV RUST_BACKTRACE=1

# 复制 Cargo 文件
COPY Cargo.toml Cargo.lock ./

# 创建虚拟 main.rs 用于依赖预编译
RUN mkdir src && echo "fn main() {}" > src/main.rs

# 预编译依赖
RUN cargo build --release

# 复制源代码
COPY src ./src

# 构建应用
RUN cargo build --release --verbose

# 运行时镜像 - 使用 distroless 镜像减小大小
FROM gcr.io/distroless/cc-debian11

# 从构建镜像复制二进制文件
COPY --from=builder /app/target/release/smart-forward /smart-forward

# 暴露端口
EXPOSE 443 99 6690 999

# 设置环境变量
ENV RUST_LOG=info

# 启动命令
ENTRYPOINT ["/smart-forward"]