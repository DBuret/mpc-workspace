# Build pour SearXNG (agent par défaut)
docker buildx build \
  --build-arg MCP_AGENT_NAME=mcp-searxng-bridge \
  -t mcp-searxng-bridge:latest \
  --platform linux/amd64,linux/arm64 \
  .

# Build pour un futur agent (ex: mcp-database-bridge)
docker buildx build \
  --build-arg MCP_AGENT_NAME=mcp-database-bridge \
  -t mcp-database-bridge:latest \
  --platform linux/amd64,linux/arm64 \
  .

# Build multi-architectures simultanément
docker buildx build \
  --build-arg MCP_AGENT_NAME=mcp-searxng-bridge \
  -t dburet/mcp-searxng-bridge:latest \
  --platform linux/amd64,linux/arm64 \
  --push .
