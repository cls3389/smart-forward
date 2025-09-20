# Smart Forward - 预编译二进制Docker镜像
# 使用GitHub Actions预编译的musl静态二进制文件

FROM alpine:3.18

# 安装运行时依赖（包含防火墙工具）
RUN apk add --no-cache \
    ca-certificates \
    tzdata \
    iptables \
    ip6tables \
    && adduser -D -s /bin/false smartforward

# 创建应用目录
RUN mkdir -p /app/logs && \
    chown -R smartforward:smartforward /app

# 复制对应架构的二进制文件（由GitHub Actions构建）
ARG TARGETARCH
COPY binaries/$TARGETARCH/smart-forward /usr/local/bin/smart-forward
RUN chmod +x /usr/local/bin/smart-forward

# 创建默认配置
RUN echo 'logging:' > /app/config.yaml && \
    echo '  level: "info"' >> /app/config.yaml && \
    echo '  format: "text"' >> /app/config.yaml && \
    echo 'network:' >> /app/config.yaml && \
    echo '  listen_addrs:' >> /app/config.yaml && \
    echo '    - "0.0.0.0"' >> /app/config.yaml && \
    echo 'buffer_size: 8192' >> /app/config.yaml && \
    echo 'rules:' >> /app/config.yaml && \
    echo '  - name: "HTTP_TEST"' >> /app/config.yaml && \
    echo '    listen_port: 8080' >> /app/config.yaml && \
    echo '    protocol: "tcp"' >> /app/config.yaml && \
    echo '    targets:' >> /app/config.yaml && \
    echo '      - "httpbin.org:80"' >> /app/config.yaml && \
    chown smartforward:smartforward /app/config.yaml

USER smartforward

EXPOSE 443 99 6690 999 8080

ENV RUST_LOG=info TZ=Asia/Shanghai

# 健康检查：检查进程是否运行
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD pgrep smart-forward > /dev/null || exit 1

CMD ["/usr/local/bin/smart-forward", "-c", "/app/config.yaml"]
