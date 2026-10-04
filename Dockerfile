# Build stage - use specific version with updates
FROM rust:1.98-bookworm AS builder
#FROM rust:1.98-slim AS builder
#FROM rust:1.98-trixie AS builder
# Doeas exist an image without critical vulnerabilities ???

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/portfolio_api*

# Copy real source and build
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx

# Build the application
RUN cargo build --release


# Tooling stage - provides the static curl used by the HEALTHCHECK below,
# so the runtime image can stay distroless
FROM debian:bookworm-slim AS tools
ARG CURL_VERSION=8.22.0
ADD https://github.com/stunnel/static-curl/releases/download/${CURL_VERSION}/curl-linux-x86_64-glibc-${CURL_VERSION}.tar.xz /tmp/curl.tar.xz
RUN tar -xf /tmp/curl.tar.xz -C /usr/local/bin curl

# Runtime stage - use distroless for minimal attack surface
FROM gcr.io/distroless/cc-debian12

COPY --from=tools /usr/local/bin/curl /usr/local/bin/curl
# Copy the binary
COPY --from=builder /app/target/release/portfolio_api /usr/local/bin/portfolio_api

# receive a value as ARG and pass it to the container as ENV
ARG CONFIGURATION_FILE
ENV CONFIGURATION_FILE=$CONFIGURATION_FILE

# Create a non-root user (so process don't run as root)
USER 10001:10001

EXPOSE 3000

# Liveness probe.
# Readiness variant: curl -sf 'http://localhost:3000/health?ready=true'
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD ["/usr/local/bin/curl", "-sf", "http://localhost:3000/health"]

ENTRYPOINT ["/usr/local/bin/portfolio_api"]
