FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:${TARGETARCH}-musl AS builder

ARG TARGETARCH

RUN if [ "$TARGETARCH" = "amd64" ]; then \
      echo "x86_64-unknown-linux-musl" > /target; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
      echo "aarch64-unknown-linux-musl" > /target; \
    else \
      echo "Unsupported platform: $TARGETARCH"; \
      exit 1; \
    fi

COPY Cargo.toml .
COPY Cargo.lock .
RUN mkdir -p src \
    && echo 'fn main() {}' > src/main.rs \
    && cargo build --release --target $(cat /target)

COPY src src
RUN CARGO_BUILD_INCREMENTAL=true cargo build --release --target $(cat /target) \
    && cp target/$(cat /target)/release/gatehook target/release/gatehook


FROM alpine

COPY --from=builder /home/rust/src/target/release/gatehook /gatehook
CMD [ "/gatehook" ]
