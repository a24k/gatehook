# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.85
ARG APP_NAME=gatehook

# cargo-chefを使った依存関係キャッシング
FROM --platform=$BUILDPLATFORM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION}-bookworm AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG APP_NAME

# 依存関係のビルド（キャッシュ可能）
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

# アプリケーションのビルド
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo build --release --bin ${APP_NAME} && \
    cp ./target/release/${APP_NAME} /bin/server

# 本番ステージ：distroless
FROM --platform=$TARGETPLATFORM gcr.io/distroless/cc-debian12:nonroot AS runtime
COPY --from=builder /bin/server /app/gatehook
WORKDIR /app
EXPOSE 8000
ENTRYPOINT ["/app/gatehook"]
