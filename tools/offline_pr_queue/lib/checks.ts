import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import crypto from "node:crypto";
import { writeJson } from "./fs.ts";
import { ensureCommandAllowed } from "../../automation/command_policy.ts";

type CheckStatus = "passed" | "failed";

export type CheckResult = {
  name: string;
  status: CheckStatus;
  stdout: string;
  stderr: string;
  code: number;
};

type CommandRunner = () => CheckResult;

type LogPathResolver = (id: number, name: string) => string;

const MAX_LOG_BUFFER = 10 * 1024 * 1024; // 10MB to capture verbose tool output

function runCommand(name: string, command: string, args: string[], options: Record<string, unknown> = {}): CheckResult {
  ensureCommandAllowed([command, ...args], { context: `offline-pr-check:${name}` });
  const result = spawnSync(command, args, {
    cwd: process.cwd(),
    stdio: ["ignore", "pipe", "pipe"],
    encoding: "utf8",
    maxBuffer: MAX_LOG_BUFFER,
    env: {
      ...process.env,
      OFFLINE_FIRST: process.env.OFFLINE_FIRST ?? "true",
      ONLINE_GITHUB_MODE: process.env.ONLINE_GITHUB_MODE ?? "false",
    },
    ...options,
  });

  if (result.error) {
    return {
      name,
      status: "failed",
      stdout: result.stdout ?? "",
      stderr: String(result.error.message ?? result.error),
      code: 1,
    };
  }

  return {
    name,
    status: result.status === 0 ? "passed" : "failed",
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
    code: result.status ?? 1,
  };
}

function readPackageMetadata(): string[] {
  try {
    const pkg = JSON.parse(readFileSync("package.json", "utf8"));
    return Object.keys(pkg.dependencies ?? {}).concat(Object.keys(pkg.devDependencies ?? {}));
  } catch (error) {
    return [];
  }
}

function readCargoMetadata(): string[] {
  try {
    const cargoToml = readFileSync("Cargo.lock", "utf8");
    return cargoToml
      .split("\n")
      .filter((line) => line.startsWith("name = "))
      .map((line) => line.split("=")[1].trim().replace(/"/g, ""));
  } catch (error) {
    return [];
  }
}

function generateSbom(): { generatedAt: string; packages: string[]; checksum: string } {
  const packages = [...new Set([...readPackageMetadata(), ...readCargoMetadata()])].sort();
  return {
    generatedAt: new Date().toISOString(),
    packages,
    checksum: crypto.createHash("sha256").update(packages.join(";"), "utf8").digest("hex"),
  };
}

function scanForLicenses(): { licenseDetected: string; length: number } {
  const license = readFileSync("LICENSE", "utf8");
  return {
    licenseDetected: license.split("\n")[0],
    length: license.length,
  };
}

function scanForSecrets(): { findings: Array<{ file: string; issue: string }>; gitignoreSample: string } {
  let gitignore = "";
  try {
    gitignore = readFileSync(".gitignore", "utf8");
  } catch (error) {
    gitignore = "";
  }
  const findings: Array<{ file: string; issue: string }> = [];
  const secretPatterns = [/API_KEY/i, /SECRET/i, /TOKEN/i];
  const workspaceFiles = ["package.json", "Cargo.toml", "pnpm-workspace.yaml"];
  for (const file of workspaceFiles) {
    try {
      const text = readFileSync(file, "utf8");
      if (secretPatterns.some((pattern) => pattern.test(text))) {
        findings.push({ file, issue: "Potential secret-like token detected" });
      }
    } catch (error) {
      // ignore
    }
  }
  return { findings, gitignoreSample: gitignore.slice(0, 200) };
}

export function performChecks(id: number, logPathResolver: LogPathResolver): CheckResult[] {
  const results: CheckResult[] = [];
  const commands: CommandRunner[] = [
    () => runCommand("lint", "pnpm", ["lint"]),
    () => runCommand("typecheck", "pnpm", ["typecheck"]),
    () => runCommand("test", "pnpm", ["test"]),
  ];

  for (const command of commands) {
    const result = command();
    results.push(result);
    const path = logPathResolver(id, result.name);
    writeJson(path, {
      name: result.name,
      status: result.status,
      code: result.code,
      stdout: result.stdout,
      stderr: result.stderr,
      completedAt: new Date().toISOString(),
    });
  }

  const sbom = generateSbom();
  const sbomPath = logPathResolver(id, "sbom").replace(/\.log$/, ".json");
  writeJson(sbomPath, sbom);
  results.push({ name: "sbom", status: "passed", stdout: JSON.stringify(sbom), stderr: "", code: 0 });

  const license = scanForLicenses();
  const licensePath = logPathResolver(id, "license").replace(/\.log$/, ".json");
  writeJson(licensePath, license);
  results.push({ name: "license", status: "passed", stdout: JSON.stringify(license), stderr: "", code: 0 });

  const secrets = scanForSecrets();
  const secretStatus = secrets.findings.length === 0 ? "passed" : "failed";
  const secretPath = logPathResolver(id, "secret-scan").replace(/\.log$/, ".json");
  writeJson(secretPath, secrets);
  results.push({ name: "secret-scan", status: secretStatus, stdout: JSON.stringify(secrets), stderr: "", code: secretStatus === "passed" ? 0 : 1 });

  return results;
}
