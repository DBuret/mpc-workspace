# ==========================================
# STAGE 1 : CARGO CHEF (Préparation du cache)
# ==========================================
# On utilise une image avec cargo-chef préinstallé
FROM lukemathwalker/cargo-chef:latest-rust-1.85 AS chef
WORKDIR /app

# On analyse l'ensemble du projet pour créer une "recette"
FROM chef AS planner
COPY . .
# Chef trouve TOUS les Cargo.toml et crée un recipe.json
RUN cargo chef prepare --recipe-path recipe.json

# ==========================================
# STAGE 2 : BUILDER (Mise en cache & Compilation)
# ==========================================
FROM chef AS builder

# Nécessaire pour notre code (SSL, etc.)
RUN apt-get update && apt-get install -y musl-tools pkg-config libssl-dev ca-certificates

WORKDIR /app
# On copie uniquement la recette générée
COPY --from=planner /app/recipe.json recipe.json

# CHEF MAGIC : Il crée l'arborescence dummy pour TOUS les serveurs
# et compile les dépendances. Si un seul Cargo.toml change, il refait cette étape.
RUN rustup target add x86_64-unknown-linux-musl \
    && cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

# MAINTENANT, on copie notre vrai code source complet
COPY . .

# On compile nos vrais binaires. Les dépendances sont déjà compilées !
RUN cargo build --release --target x86_64-unknown-linux-musl


# ==========================================
# STAGE 3 : BASE RUNTIME
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
# STAGE 4 : VOS AGENTS (Cibles)
# ==========================================
FROM base-runtime AS mcp-searxng-bridge

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/mcp-searxng-bridge /app/mcp-bridge

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

EXPOSE 3000
ENTRYPOINT ["./mcp-bridge"]


# ==========================================
# STAGE 4x : AGENT TEMPLATE / NOUVEL AGENT
# ==========================================
#FROM base-runtime AS 
#
# COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/mcp-searxng-bridge /app/mcp-bridge
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

#EXPOSE 3000
#ENTRYPOINT ["./mcp-bridge"]
