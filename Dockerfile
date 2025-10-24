# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.90
ARG APP_NAME=gatehook

# クロスコンパイル対応のビルドステージ
FROM --platform=$BUILDPLATFORM rust:${RUST_VERSION}-bookworm AS builder
ARG TARGETPLATFORM
ARG BUILDPLATFORM
ARG APP_NAME

WORKDIR /app

# ターゲットプラットフォームに応じたRustターゲットとZigターゲットを設定
RUN case "$TARGETPLATFORM" in \
    "linux/arm64") \
        echo "aarch64-unknown-linux-gnu" > /tmp/rust_target.txt && \
        echo "aarch64-linux-gnu" > /tmp/zig_target.txt \
        ;; \
    "linux/amd64") \
        echo "x86_64-unknown-linux-gnu" > /tmp/rust_target.txt && \
        echo "x86_64-linux-gnu" > /tmp/zig_target.txt \
        ;; \
    *) echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
    esac

# Zigのインストール（クロスコンパイル用リンカー）
RUN apt-get update && apt-get install -y wget xz-utils && \
    wget -q https://ziglang.org/download/0.11.0/zig-linux-x86_64-0.11.0.tar.xz && \
    tar -xf zig-linux-x86_64-0.11.0.tar.xz -C /usr/local && \
    ln -s /usr/local/zig-linux-x86_64-0.11.0/zig /usr/local/bin/zig && \
    rm zig-linux-x86_64-0.11.0.tar.xz && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# cargo-zigbuildのインストールとRustターゲットの追加
RUN cargo install cargo-zigbuild && \
    rustup target add $(cat /tmp/rust_target.txt)

# 依存関係のビルド（recipe.json方式）
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo zigbuild --release --target $(cat /tmp/rust_target.txt) && \
    rm -rf src

# アプリケーションのビルド
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo zigbuild --release --target $(cat /tmp/rust_target.txt) && \
    cp ./target/$(cat /tmp/rust_target.txt)/release/${APP_NAME} /bin/server

# 本番ステージ：distroless
FROM gcr.io/distroless/cc-debian12:nonroot AS runtime
COPY --from=builder /bin/server /app/gatehook
WORKDIR /app
ENTRYPOINT ["/app/gatehook"]
