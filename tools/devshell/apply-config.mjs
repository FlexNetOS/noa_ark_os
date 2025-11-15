#!/usr/bin/env node
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const mode = process.argv[2] || 'posix';
const devshellDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(devshellDir, '..', '..');
const configPath = path.join(devshellDir, 'config.json');

function loadConfig() {
  if (!fs.existsSync(configPath)) {
    return {};
  }
  return JSON.parse(fs.readFileSync(configPath, 'utf8'));
}

function resolveValue(value) {
  if (typeof value !== 'string') {
    return value;
  }
  if (value.startsWith('${')) {
    return value;
  }
  if (path.isAbsolute(value)) {
    return value;
  }
  return path.resolve(workspaceRoot, value);
}

function flattenPaths(section) {
  const entries = [];
  if (!section) {
    return entries;
  }
  const raw = section.PATH_PREPEND || section.path_prepend;
  if (Array.isArray(raw)) {
    raw.forEach((item) => entries.push(resolveValue(item)));
  } else if (typeof raw === 'string' && raw.trim().length > 0) {
    entries.push(resolveValue(raw.trim()));
  }
  return entries;
}

function collectEnv(config) {
  const env = {};
  const common = config.env?.common || {};
  const platformKey = mode.toLowerCase().startsWith('posix') ? 'posix' : 'windows';
  const platform = config.env?.[platformKey] || {};

  const pathSegments = [
    ...flattenPaths(common),
    ...flattenPaths(platform),
  ];

  function assignSection(section) {
    Object.entries(section).forEach(([key, value]) => {
      if (key === 'PATH_PREPEND' || key === 'path_prepend') {
        return;
      }
      env[key] = value;
    });
  }

  assignSection(common);
  assignSection(platform);

  Object.entries(env).forEach(([key, value]) => {
    if (Array.isArray(value)) {
      env[key] = value.map((item) => resolveValue(item));
    } else {
      env[key] = resolveValue(value);
    }
  });

  return { env, pathSegments };
}

function renderPosix(config) {
  const lines = [];
  const { env, pathSegments } = collectEnv(config);
  if (config.pnpm?.requiredVersion) {
    lines.push(`export NOA_PNPM_REQUIRED="${config.pnpm.requiredVersion}"`);
  }
  if (config.pnpm?.defaultStoreDir) {
    lines.push(`export PNPM_STORE_DIR="${resolveValue(config.pnpm.defaultStoreDir)}"`);
  }
  if (pathSegments.length > 0) {
    lines.push(`export PATH="${pathSegments.join(':')}:$PATH"`);
  }
  Object.entries(env).forEach(([key, value]) => {
    if (Array.isArray(value)) {
      lines.push(`export ${key}="${value.join(':')}"`);
    } else {
      lines.push(`export ${key}="${value}"`);
    }
  });
  if (config.rustAnalyzer?.checkCommand) {
    lines.push(`export RUST_ANALYZER_CHECK_COMMAND="${config.rustAnalyzer.checkCommand}"`);
  }
  const aliases = config.pnpm?.aliases || {};
  Object.entries(aliases).forEach(([alias, command]) => {
    lines.push(`alias ${alias}='${command}'`);
  });
  return lines.join('\n');
}

function renderPwsh(config) {
  const lines = [];
  const { env, pathSegments } = collectEnv(config);
  if (config.pnpm?.requiredVersion) {
    lines.push(`$env:NOA_PNPM_REQUIRED = "${config.pnpm.requiredVersion}"`);
  }
  if (config.pnpm?.defaultStoreDir) {
    lines.push(`$env:PNPM_STORE_DIR = "${resolveValue(config.pnpm.defaultStoreDir)}"`);
  }
  if (pathSegments.length > 0) {
    const joined = pathSegments.join(';');
    lines.push(`$env:Path = "${joined};" + $env:Path`);
  }
  Object.entries(env).forEach(([key, value]) => {
    if (Array.isArray(value)) {
      lines.push(`$env:${key} = "${value.join(';')}"`);
    } else {
      lines.push(`$env:${key} = "${value}"`);
    }
  });
  if (config.rustAnalyzer?.checkCommand) {
    lines.push(`$env:RUST_ANALYZER_CHECK_COMMAND = "${config.rustAnalyzer.checkCommand}"`);
  }
  const aliases = config.pnpm?.aliases || {};
  Object.entries(aliases).forEach(([alias, command]) => {
    lines.push(`Set-Alias -Name ${alias} -Value "${command}" -Force`);
  });
  return lines.join('\n');
}

const config = loadConfig();

if (mode.toLowerCase().startsWith('posix')) {
  console.log(renderPosix(config));
} else if (mode.toLowerCase().startsWith('pwsh') || mode.toLowerCase().startsWith('powershell')) {
  console.log(renderPwsh(config));
} else {
  console.error(`Unknown mode "${mode}". Expected 'posix' or 'pwsh'.`);
  process.exitCode = 1;
}
