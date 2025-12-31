const fs = require('fs');
const path = require('path');

const ROOT = __dirname;
const CONFIG_PATH = path.join(ROOT, 'tools', 'devshell', 'config.json');

function loadConfig() {
  try {
    const raw = fs.readFileSync(CONFIG_PATH, 'utf8');
    return JSON.parse(raw);
  } catch (error) {
    if (error.code === 'ENOENT') {
      return {};
    }
    throw error;
  }
}

const config = loadConfig();
const requiredPnpm = config?.pnpm?.requiredVersion;

function assertPnpmVersion() {
  if (!requiredPnpm) {
    return;
  }
  const agent = process.env.npm_config_user_agent || '';
  const match = agent.match(/pnpm\/(\d+\.\d+\.\d+)/);
  if (!match) {
    const message = [
      `This workspace must be installed with pnpm@${requiredPnpm}.`,
      `Detected agent: "${agent || 'unknown'}".`,
      `If you are invoking pnpm through corepack, run \`corepack prepare pnpm@${requiredPnpm} --activate\` first.`,
    ].join(' ');
    throw new Error(message);
  }
  const actual = match[1];
  if (actual !== requiredPnpm) {
    const message = [
      `pnpm@${requiredPnpm} is required for this workspace (detected pnpm@${actual}).`,
      `Use \`corepack prepare pnpm@${requiredPnpm} --activate\` or install the matching version manually.`,
    ].join(' ');
    throw new Error(message);
  }
}

module.exports = {
  hooks: {
    readPackage(pkg) {
      assertPnpmVersion();
      return pkg;
    },
  },
};
