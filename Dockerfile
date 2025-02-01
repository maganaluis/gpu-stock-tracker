# syntax=docker/dockerfile:1.4

# ----------- Stage 1: Builder ------------
FROM --platform=$BUILDPLATFORM rust:1.84-bullseye AS builder

# We need a few environment variables for cross-compilation:
#   BUILDPLATFORM / TARGETPLATFORM / TARGETARCH are automatically set by Buildx.
#   We'll configure cargo for release builds.
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

WORKDIR /app
COPY . .

# Build your Rust app in release mode
RUN cargo build --release

# ----------- Stage 2: Final (Distroless or minimal base) ------------
# Option A: Use a minimal base like Debian slim or Alpine:
FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/gpu-stock-tracker /usr/local/bin/gpu-stock-tracker
CMD ["/usr/local/bin/gpu-stock-tracker"]

# Option B: Use scratch (smallest possible image, but no shell)
# FROM scratch
# You won't have CA certificates by default in scratch; if your app needs SSL/TLS,
# copy the CA bundle from the builder or use a minimal Linux base above.
# COPY --from=builder /app/target/release/gpu-stock-tracker /gpu-stock-tracker
# ENTRYPOINT ["/gpu-stock-tracker"]
