# === Stage 1: Frontend Build ===
FROM node:20-alpine AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ .
RUN npm run build

# === Stage 2: Chef (Planner) ===
FROM rust:slim-bookworm AS chef
RUN apt-get update && apt-get install -y cmake clang pkg-config libssl-dev perl && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef
WORKDIR /app

# === Stage 3: Recipe Planner ===
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# === Stage 4: Backend Builder ===
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this layer is cached until Cargo.toml changes
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin backend

# === Stage 5: Final Runtime ===
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies (Certbot & DNS plugins)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    certbot \
    python3-certbot-dns-cloudflare \
    python3-certbot-dns-route53 \
    python3-certbot-dns-digitalocean \
    python3-certbot-dns-google \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries and assets
COPY --from=builder /app/target/release/backend /app/pingora-pm
COPY --from=frontend-builder /app/frontend/dist /app/static

# Setup directories
RUN mkdir -p /app/data /etc/letsencrypt

# Expose ports (8080: Proxy, 81: UI)
EXPOSE 8080 81

CMD ["/app/pingora-pm"]
