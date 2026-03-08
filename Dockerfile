# --- Étape 1 : Build multi-agents ---
FROM rust:1.85-slim AS builder

# Installation des dépendances système
RUN apt-get update && apt-get install -y \
    musl-tools \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copie des fichiers de workspace + lockfile
COPY Cargo.toml Cargo.lock ./

# Pré-build des dépendances (cache Docker)
RUN mkdir -p core/mcp-network-core/src servers/mcp-searxng-bridge/src \
    && echo "fn main() {}" > core/mcp-network-core/src/lib.rs \
    && echo "fn main() {}" > servers/mcp-searxng-bridge/src/main.rs \
    && cargo build --release --target x86_64-unknown-linux-musl

# Copie du code source réel
COPY . .

# Build de TOUS les agents du workspace
RUN rustup target add x86_64-unknown-linux-musl \
    && cargo build --release --target x86_64-unknown-linux-musl \
    && ls -la target/x86_64-unknown-linux-musl/release/

# --- Étape 2 : Runtime final ---
FROM scratch

# --- LABEL OCI STANDARDS ---
LABEL org.opencontainers.image.title="MCP SearXNG Rust Bridge"
LABEL org.opencontainers.image.description="High-performance MCP server bridge connecting AI agents to SearXNG via SSE. Features web search and smart Markdown scraping."
LABEL org.opencontainers.image.vendor="DBuret"
LABEL org.opencontainers.image.authors="DBuret"

LABEL org.opencontainers.image.url="https://github.com/DBuret/mcp-searxng-rs"
LABEL org.opencontainers.image.source="https://github.com/DBuret/mcp-searxng-bridge"
LABEL org.opencontainers.image.documentation="https://github.com/DBuret/mcp-searxng-bridge/blob/main/README.adoc"

LABEL org.opencontainers.image.version="0.3.1"
LABEL org.opencontainers.image.revision="7bae13f" 

LABEL org.opencontainers.image.licenses="MIT"

LABEL com.paitrimony.mcp.protocol_version="2024-11-05"
LABEL com.paitrimony.mcp.transport="sse"
LABEL com.paitrimony.mcp.tools="search,fetch_page"

# Certificats SSL
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# === COPIE DYNAMIQUE DU BINAIRE SELON LE BUILD ARG ===
ARG MCP_AGENT_NAME=mcp-searxng-bridge
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/${MCP_AGENT_NAME} /app/mcp-bridge

# Variables d'environnement par défaut (spécifiques à SearXNG)
ENV MCP_SX_URL="http://172.17.0.1:18080"
ENV MCP_SX_PORT="3000"
ENV MCP_SX_LOG="info"

WORKDIR /app
EXPOSE 3000
USER 1000

ENTRYPOINT ["./mcp-bridge"]
