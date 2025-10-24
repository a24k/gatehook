# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.90

# Build stage with cross-compilation support
FROM --platform=$BUILDPLATFORM rust:${RUST_VERSION}-bookworm AS builder
ARG TARGETPLATFORM
ARG BUILDPLATFORM

WORKDIR /app

# Set Rust target based on platform (using musl for static linking)
RUN case "$TARGETPLATFORM" in \
    "linux/arm64") \
        echo "aarch64-unknown-linux-musl" > /tmp/rust_target.txt \
        ;; \
    "linux/amd64") \
        echo "x86_64-unknown-linux-musl" > /tmp/rust_target.txt \
        ;; \
    *) echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
    esac

# Install Zig (cross-compilation linker)
RUN apt-get update && apt-get install -y wget xz-utils && \
    wget -q https://ziglang.org/download/0.11.0/zig-linux-x86_64-0.11.0.tar.xz && \
    tar -xf zig-linux-x86_64-0.11.0.tar.xz -C /usr/local && \
    ln -s /usr/local/zig-linux-x86_64-0.11.0/zig /usr/local/bin/zig && \
    rm zig-linux-x86_64-0.11.0.tar.xz && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Install cargo-zigbuild and add Rust target
RUN cargo install cargo-zigbuild && \
    rustup target add $(cat /tmp/rust_target.txt)

# Build application
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo zigbuild --release --target $(cat /tmp/rust_target.txt) && \
    cp ./target/$(cat /tmp/rust_target.txt)/release/gatehook ./gatehook

# Runtime stage: distroless static (for statically-linked binaries)
FROM gcr.io/distroless/static-debian12:nonroot AS runtime

COPY --from=builder /app/gatehook /app/gatehook
WORKDIR /app

ENTRYPOINT ["/app/gatehook"]
