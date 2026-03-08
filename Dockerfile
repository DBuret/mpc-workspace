# ==========================================
# STAGE 1 : BUILDER COMMUN (Compile tout le workspace)
# ==========================================
FROM rust:1.85-slim AS builder

RUN apt-get update && apt-get install -y \
    musl-tools \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache des dépendances (optionnel mais recommandé pour la vitesse)
COPY Cargo.toml Cargo.lock ./
# On crée des fichiers dummy pour tromper Cargo et mettre en cache les dépendances
RUN mkdir -p core/mcp-network-core/src servers/mcp-searxng-bridge/src servers/template/src \
    && echo "fn main() {}" > core/mcp-network-core/src/lib.rs \
    && echo "fn main() {}" > servers/mcp-searxng-bridge/src/main.rs \
    && echo "fn main() {}" > servers/template/src/main.rs \
    && cargo build --release --target x86_64-unknown-linux-musl

# Copie du vrai code et compilation de TOUS les binaires
COPY . .
RUN rustup target add x86_64-unknown-linux-musl \
    && cargo build --release --target x86_64-unknown-linux-musl


# ==========================================
# STAGE 2 : BASE RUNTIME (Tronc commun pour les images finales)
# ==========================================
FROM scratch AS base-runtime
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Labels communs à tous les projets du workspace
LABEL org.opencontainers.image.vendor="DBuret"
LABEL org.opencontainers.image.authors="DBuret"
LABEL org.opencontainers.image.licenses="MIT"

WORKDIR /app
USER 1000


# ==========================================
# STAGE 3A : AGENT SEARXNG_BRIDGE (Cible spécifique)
# ==========================================
FROM base-runtime AS mcp-searxng-bridge

# Labels OCI spécifiques à SearXNG
LABEL org.opencontainers.image.title="MCP SearXNG Bridge"
LABEL org.opencontainers.image.description="MCP server bridging AI agents to SearXNG with web scraping."
LABEL org.opencontainers.image.documentation="https://github.com/DBuret/mcp-searxng-bridge/blob/main/README.adoc"
LABEL org.opencontainers.image.url="https://github.com/DBuret/mcp-searxng-bridge"
LABEL com.paitrimony.mcp.tools="search,fetch_page"

# ENV spécifiques à SearXNG
ENV MCP_SEARXNG_BRIDGE_URL="http://172.17.0.1:18080"
ENV MCP_SEARXNG_BRIDGE_PORT="3000"
ENV MCP_SEARXNG_BRIDGE_LOG="info"

# Copie exclusive de CE binaire
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/mcp-searxng-bridge /app/mcp-bridge

EXPOSE 3000
ENTRYPOINT ["./mcp-bridge"]


# ==========================================
# STAGE 3X : AGENT TEMPLATE / NOUVEL AGENT
# ==========================================
#FROM base-runtime AS mcp-template-bridge
#
# Labels OCI spécifiques au Template
#LABEL org.opencontainers.image.title="MCP Template Bridge"
#LABEL org.opencontainers.image.description="An example template for new MCP agents."
#LABEL org.opencontainers.image.url="https://github.com/DBuret/mcp-searxng-bridge"
#LABEL com.paitrimony.mcp.tools="hello_world"
#
# ENV spécifiques au Template
#ENV MCP_TEMPLATE_PORT="3001"
#ENV MCP_TEMPLATE_LOG="info"

# Copie exclusive de CE binaire
#COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/mcp-template-bridge /app/mcp-bridge

#EXPOSE 3001
#ENTRYPOINT ["./mcp-bridge"]
