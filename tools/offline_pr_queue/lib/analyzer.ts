type Heuristic = {
  pattern: RegExp;
  summary: string;
  suggestions: string[];
};

const HEURISTICS: Heuristic[] = [
  {
    pattern: /SyntaxError|Unexpected token/i,
    summary: "Syntax issue detected in recent changes.",
    suggestions: ["Review recent code edits for unmatched brackets or misplaced keywords.", "Run the TypeScript compiler locally to pinpoint the line."],
  },
  {
    pattern: /TypeError: (.*)/i,
    summary: "Runtime type mismatch occurred during tests.",
    suggestions: ["Add guards around the failing call site.", "Extend test coverage to enforce the expected types."],
  },
  {
    pattern: /ReferenceError: (.*) is not defined/i,
    summary: "Reference error encountered in execution.",
    suggestions: ["Verify imports for the missing symbol.", "Ensure build artifacts are generated before running the suite."],
  },
  {
    pattern: /secret/i,
    summary: "Potential secret detected during scan.",
    suggestions: ["Rotate the affected secret immediately.", "Move credentials into the vault service configuration."],
  },
];

export type TriageAnalysis = {
  check: string;
  summary: string;
  suggestions: string[];
  matched: string | null;
};

export function analyzeLog(name: string, log: string): TriageAnalysis {
  for (const heuristic of HEURISTICS) {
    if (heuristic.pattern.test(log)) {
      return {
        check: name,
        summary: heuristic.summary,
        suggestions: heuristic.suggestions,
        matched: heuristic.pattern.source,
      };
    }
  }

  return {
    check: name,
    summary: "Investigation required for failing check.",
    suggestions: ["Re-run the suite with verbose logging.", "Escalate to maintainers with the captured artifact."],
    matched: null,
  };
}
