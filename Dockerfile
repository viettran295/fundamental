FROM rust:1.92.0-slim-bookworm@sha256:a8ce22642819f5f54d37e025f7020b51dcbac39e5d0ea907a63eda8b03458de5 AS builder
WORKDIR /app
RUN apt-get update &&               \
    apt-get install -y openssl      \
                        libssl-dev  \
                        pkg-config
COPY . .
RUN cargo build --release --verbose

# -- Runtime --
FROM debian:13-slim@sha256:4bcb9db66237237d03b55b969271728dd3d955eaaa254b9db8a3db94550b1885
RUN apt-get update &&                       \
    apt-get install -y openssl              \
                        ca-certificates &&  \
    update-ca-certificates --fresh &&       \
    rm -rf /var/lib/apt/lists/*
RUN useradd --create-home --uid 10001 app-user
USER app-user
WORKDIR /app
COPY --from=builder --chown=app-user:app-user /app/target/release/fundamental ./
ARG CACHE_DB_URI
ENV CACHE_DB_URI=$CACHE_DB_URI
ENV RUST_LOG=debug
EXPOSE 3000
CMD ["./fundamental"]