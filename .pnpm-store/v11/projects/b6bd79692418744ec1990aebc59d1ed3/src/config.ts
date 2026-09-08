import path from "node:path";
import os from "node:os";

export const config = {
  repo: process.env.AGENT_REPO || "mr-exception/salimon",
  concurrency: Number(process.env.AGENT_CONCURRENCY || 1),
  label: process.env.AGENT_LABEL || "auto-verification",
  image: process.env.AGENT_IMAGE || "agent-control-worker:local",
  codexHome: process.env.AGENT_CODEX_HOME || path.join(os.homedir(), ".codex"),
  data: path.resolve(".data"),
  pollMs: Number(process.env.AGENT_POLL_MS || 30000),
};
if (!/^[\w.-]+\/[\w.-]+$/.test(config.repo)) throw Error("AGENT_REPO must be owner/repository");
if (!Number.isInteger(config.concurrency) || config.concurrency < 1 || config.concurrency > 8) throw Error("AGENT_CONCURRENCY must be 1–8");
if (!Number.isFinite(config.pollMs) || config.pollMs < 5000) throw Error("AGENT_POLL_MS must be at least 5000");
