#!/usr/bin/env node
const fs = require('fs');
const path = require('path');

const args = process.argv.slice(2);
if (args.length === 0) {
  process.exit(0);
}

let mode = 'posix';
let query = null;
for (let i = 0; i < args.length; i += 1) {
  const token = args[i];
  if (token === '--mode') {
    mode = args[i + 1] || mode;
    i += 1;
  } else if (!query) {
    query = token;
  }
}

const devshellDir = __dirname;
const workspaceRoot = path.resolve(devshellDir, '..', '..');
const configPath = path.join(devshellDir, 'config.json');

if (!fs.existsSync(configPath)) {
  process.exit(0);
}

let config;
try {
  config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
} catch (err) {
  console.error(`Error: Failed to parse config file at "${configPath}": ${err.message}`);
  process.exit(1);
}
const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));

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

function collectEnv(selectedMode) {
  const env = {};
  const common = config.env?.common || {};
  const platformKey = selectedMode.toLowerCase().startsWith('posix') ? 'posix' : 'windows';
  const platform = config.env?.[platformKey] || {};

  const pathSegments = [
    ...flattenPaths(common),
    ...flattenPaths(platform),
  ];

  const assignSection = (section) => {
    Object.entries(section).forEach(([key, value]) => {
      if (key === 'PATH_PREPEND' || key === 'path_prepend') {
        return;
      }
      env[key] = value;
    });
  };

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

function emit(value) {
  if (value === undefined || value === null) {
    return;
  }
  if (Array.isArray(value)) {
    console.log(value.join(mode.toLowerCase().startsWith('posix') ? ':' : ';'));
  } else if (typeof value === 'object') {
    console.log(JSON.stringify(value));
  } else {
    console.log(String(value));
  }
}

if (query === 'pnpm.requiredVersion') {
  emit(config.pnpm?.requiredVersion);
} else if (query === 'pnpm.defaultStoreDir') {
  emit(resolveValue(config.pnpm?.defaultStoreDir));
} else if (query.startsWith('env.')) {
  const { env, pathSegments } = collectEnv(mode);
  if (query === 'env.pathPrefix') {
    emit(pathSegments);
  } else {
    const key = query.slice('env.'.length);
    emit(env[key]);
  }
} else if (query === 'pnpm.aliases') {
  emit(config.pnpm?.aliases || {});
} else if (query === 'rustAnalyzer.checkCommand') {
  emit(config.rustAnalyzer?.checkCommand);
}
