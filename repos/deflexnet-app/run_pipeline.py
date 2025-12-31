"""Command line entry point for the Digest pipeline."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from digest_pipeline import (
    CRMStranglerStage,
    ClassifierStage,
    EmbeddingsStage,
    EnvSynthesisStage,
    GraphExtractStage,
    IntakeStage,
    IntegratorStage,
    Pipeline,
    PipelineContext,
    RegistrarStage,
    ReverseEngineerStage,
    RunnerStage,
    SafetyStage,
)


def build_pipeline() -> Pipeline:
    stages = [
        IntakeStage(),
        ClassifierStage(),
        GraphExtractStage(),
        EmbeddingsStage(),
        EnvSynthesisStage(),
        SafetyStage(),
        RunnerStage(),
        ReverseEngineerStage(),
        IntegratorStage(),
        RegistrarStage(),
        CRMStranglerStage(),
    ]
    return Pipeline(stages)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run the DeflexNet digest pipeline")
    parser.add_argument("repo", type=Path, nargs="?", default=Path.cwd(), help="Repository path to analyse")
    parser.add_argument("--output", type=Path, default=Path("build/digest"), help="Directory for generated artifacts")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    repo_path = args.repo.resolve()
    output_path = args.output.resolve()
    output_path.mkdir(parents=True, exist_ok=True)

    pipeline = build_pipeline()
    context = PipelineContext(repo_path=repo_path, output_path=output_path)
    result = pipeline.run(context)

    summary_path = output_path / "summary.json"
    summary_path.write_text(json.dumps({"logs": result.logs}, indent=2), encoding="utf-8")
    print(f"Digest pipeline completed. Logs written to {summary_path}")


if __name__ == "__main__":
    main()
