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


# Runtime stage - use distroless for minimal attack surface
FROM gcr.io/distroless/cc-debian12

# Copy the binary
COPY --from=builder /app/target/release/portfolio_api /usr/local/bin/portfolio_api

# receive a value as ARG and pass it to the container as ENV
ARG CONFIGURATION_FILE
ENV CONFIGURATION_FILE=$CONFIGURATION_FILE

# Create a non-root user (so process don't run as root)
USER 10001:10001

EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/portfolio_api"]
