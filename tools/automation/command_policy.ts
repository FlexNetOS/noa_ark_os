import { readFileSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";
import process from "node:process";

export type PolicyRule = {
  id: string;
  type: "exact" | "regex";
  value: string;
  flags?: string;
  scope?: "command" | "line";
  action: "allow" | "deny";
  description?: string;
};

export type CommandPolicy = {
  version: number;
  description?: string;
  rules: PolicyRule[];
  defaultAction?: "allow" | "deny";
};

type CompiledRule = PolicyRule & { regex?: RegExp; scope: "command" | "line" };

type CachedPolicy = {
  path: string;
  mtimeMs: number;
  policy: CommandPolicy;
  compiled: CompiledRule[];
};

let cache: CachedPolicy | undefined;

function getDefaultPolicyPath(): string {
  if (process.env.COMMAND_POLICY_PATH && process.env.COMMAND_POLICY_PATH.trim().length > 0) {
    return path.resolve(process.cwd(), process.env.COMMAND_POLICY_PATH);
  }
  return fileURLToPath(new URL("./policy.yaml", import.meta.url));
}

function parsePolicyFile(filePath: string): CommandPolicy {
  const raw = readFileSync(filePath, "utf8");
  const data = JSON.parse(raw);
  if (!data || typeof data !== "object") {
    throw new Error(`Invalid policy file: ${filePath}`);
  }
  if (typeof data.version !== "number") {
    throw new Error("Policy file is missing a numeric version field");
  }
  if (!Array.isArray(data.rules)) {
    throw new Error("Policy file must contain an array of rules");
  }
  const rules: PolicyRule[] = data.rules.map((rule: unknown, index: number) => {
    if (!rule || typeof rule !== "object") {
      throw new Error(`Rule at index ${index} is not an object`);
    }
    const r = rule as Record<string, unknown>;
    if (typeof r.id !== "string" || r.id.trim() === "") {
      throw new Error(`Rule at index ${index} is missing an id`);
    }
    if (r.type !== "exact" && r.type !== "regex") {
      throw new Error(`Rule ${r.id} has unsupported type: ${String(r.type)}`);
    }
    if (typeof r.value !== "string" || r.value.length === 0) {
      throw new Error(`Rule ${r.id} must provide a value`);
    }
    const action = r.action === "allow" ? "allow" : r.action === "deny" ? "deny" : undefined;
    if (!action) {
      throw new Error(`Rule ${r.id} must declare action as allow or deny`);
    }
    const scope = r.scope === "command" ? "command" : "line";
    const flags = typeof r.flags === "string" ? r.flags : undefined;
    const description = typeof r.description === "string" ? r.description : undefined;
    return {
      id: r.id,
      type: r.type,
      value: r.value,
      flags,
      scope,
      action,
      description,
    } satisfies PolicyRule;
  });
  const defaultAction = data.defaultAction === "allow" ? "allow" : "deny";
  return { version: data.version, description: data.description, rules, defaultAction };
}

function compileRule(rule: PolicyRule): CompiledRule {
  if (rule.type === "regex") {
    const regex = new RegExp(rule.value, rule.flags ?? "");
    return { ...rule, regex, scope: rule.scope ?? "line" };
  }
  return { ...rule, scope: rule.scope ?? "line" };
}

function loadPolicy(policyPath?: string): CachedPolicy {
  const resolved = policyPath ? path.resolve(policyPath) : getDefaultPolicyPath();
  const stats = statSync(resolved);
  if (cache && cache.path === resolved && cache.mtimeMs === stats.mtimeMs) {
    return cache;
  }
  const policy = parsePolicyFile(resolved);
  const compiled = policy.rules.map((rule) => compileRule(rule));
  cache = { path: resolved, mtimeMs: stats.mtimeMs, policy, compiled };
  return cache;
}

export type Evaluation = {
  allowed: boolean;
  rule?: CompiledRule;
  reason?: string;
};

function matches(rule: CompiledRule, commandLine: string, tokens: string[]): boolean {
  if (rule.type === "exact") {
    if (rule.scope === "command") {
      return tokens[0] === rule.value;
    }
    return commandLine === rule.value || commandLine.startsWith(`${rule.value} `);
  }
  const target = rule.scope === "command" ? tokens[0] ?? "" : commandLine;
  return rule.regex!.test(target);
}

export function evaluateCommand(
  args: string[],
  options: { policyPath?: string } = {},
): Evaluation {
  if (args.length === 0) {
    throw new Error("Cannot evaluate empty command");
  }
  const commandLine = args.join(" ");
  const tokens = [...args];
  const { compiled, policy } = loadPolicy(options.policyPath);
  for (const rule of compiled) {
    if (matches(rule, commandLine, tokens)) {
      return {
        allowed: rule.action === "allow",
        rule,
        reason: rule.description,
      };
    }
  }
  return {
    allowed: policy.defaultAction === "allow",
    reason: "Default policy action",
  };
}

export function ensureCommandAllowed(
  args: string[],
  options: { policyPath?: string; context?: string } = {},
): void {
  const evaluation = evaluateCommand(args, options);
  if (!evaluation.allowed) {
    const context = options.context ? `${options.context}: ` : "";
    const ruleHint = evaluation.rule ? ` (rule: ${evaluation.rule.id})` : "";
    const reason = evaluation.reason ? ` - ${evaluation.reason}` : "";
    throw new Error(`${context}command blocked by policy${ruleHint}: ${args.join(" ")}${reason}`);
  }
}

export function getCompiledPolicy(options: { policyPath?: string } = {}): CompiledRule[] {
  return loadPolicy(options.policyPath).compiled;
}

export function getPolicy(options: { policyPath?: string } = {}): CommandPolicy {
  return loadPolicy(options.policyPath).policy;
}
