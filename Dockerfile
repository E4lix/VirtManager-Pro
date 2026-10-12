# Stage 1: Build Frontend
FROM node:20-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm install
COPY frontend/ ./
RUN npm run build

# Stage 2: Build Backend Rust
FROM rust:1.85-slim-bookworm AS backend-builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev build-essential curl unzip
WORKDIR /app/backend
COPY backend/Cargo.toml ./
# Pre-fetch dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo check && rm -rf src
COPY backend/ ./
RUN cargo build --release

# Stage 3: Runtime
FROM debian:13-slim
RUN apt-get update && apt-get install -y \
    qemu-utils \
    libvirt-clients \
    openssh-client \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=backend-builder /app/backend/target/release/virtmanager-backend /app/virtmanager-backend
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist
COPY templates /app/templates

EXPOSE 8080
CMD ["/app/virtmanager-backend"]
