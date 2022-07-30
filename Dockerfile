FROM lukemathwalker/cargo-chef:latest-rust-1.61.0 as chef
WORKDIR /app

RUN apt update && apt install lld clang -y

FROM chef as planner
COPY . .
#  Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json


FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE true
# Let's build our binary!
# We'll use the release profile to make it faaaast
RUN cargo build --release --bin zero2prod


FROM debian:bullseye-slim AS runtime
WORKDIR /app

# Install OpenSSL - it is dynamically linked by some of our dependencies
# Install ca-certificates - it is needed to verify TLS certificates
# when establishing HTTPS connections
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT production
# When `docker run` is executed, launch the binary!
ENTRYPOINT ["./zero2prod"]

# build the image
# 最后的 . 说明将当前路径作为 build context，这样 COPY ADD 命令就会从当前文件夹与 docker 环境产生联系
# host.docker.internal 我理解跟 user/.docker/config.json 中配置的代理配置的一样
# --network host，使用跟宿主相同的网络，用于 vpn
# 跟服务器不需要 build-arg 和 network
# docker build --build-arg http_proxy=http://host.docker.internal:1087 --build-arg https_proxy=http://host.docker.internal:1087 --network host --tag zero2prod --file ./Dockerfile .

# docker run -p 8000:8000 zero2prod