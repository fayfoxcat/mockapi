# 多阶段构建 Dockerfile
# 第一阶段：构建
FROM rust:1.75-alpine AS builder

# 安装构建依赖
RUN apk add --no-cache musl-dev

# 设置工作目录
WORKDIR /app

# 复制项目文件
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY static/ ./static/

# 设置构建优化环境变量
ENV CARGO_INCREMENTAL=0
ENV RUSTFLAGS="-C target-cpu=native -C link-arg=-s"

# 构建应用
RUN cargo build --release --target x86_64-unknown-linux-musl

# 第二阶段：运行时
FROM scratch

# 从构建阶段复制可执行文件
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/mockapi /mockapi

# 创建数据目录
VOLUME ["/data"]

# 暴露端口
EXPOSE 8344

# 设置入口点
ENTRYPOINT ["/mockapi"]
CMD ["--host", "0.0.0.0", "--port", "8344"]