FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:${TARGETARCH}-musl AS chef
RUN cargo install cargo-chef
WORKDIR /app

ARG TARGETARCH
RUN if [ "$TARGETARCH" = "amd64" ]; then \
      echo "x86_64-unknown-linux-musl" > /target; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
      echo "aarch64-unknown-linux-musl" > /target; \
    else \
      echo "Unsupported platform: $TARGETARCH"; \
      exit 1; \
    fi

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching layer
RUN cargo chef cook --release --target $(cat /target) --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --target $(cat /target) \
    && cp target/$(cat /target)/release/gatehook target/release/gatehook

FROM --platform=$TARGETPLATFORM alpine
COPY --from=builder /app/target/release/gatehook /gatehook
CMD [ "/gatehook" ]
