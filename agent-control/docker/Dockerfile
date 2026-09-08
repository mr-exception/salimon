FROM node:22-bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends git gh gosu ca-certificates curl build-essential pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN npm install -g @openai/codex
RUN useradd -m -u 1000 agent || true
# node image already owns uid 1000; use that user consistently.
RUN mkdir -p /home/agent/.codex /workspace && chown -R 1000:1000 /home/agent /workspace
COPY docker/entrypoint.sh /usr/local/bin/agent-entrypoint
RUN chmod 755 /usr/local/bin/agent-entrypoint
ENV HOME=/home/agent CODEX_HOME=/home/agent/.codex
WORKDIR /workspace
ENTRYPOINT ["/usr/local/bin/agent-entrypoint"]
