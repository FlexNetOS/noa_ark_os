# DeflexNet Digest Pipeline

This repository contains a lightweight, fully offline-friendly pipeline that
builds a "digest" of any source workspace. The pipeline is implemented as a
series of agents/stages that walk a repository, collect metadata, and emit
registry-ready artifacts for downstream systems.

## Features

The pipeline mirrors the "Everything" flow outlined in the product brief:

1. **Intake** – clone/fetch, provenance hints, and file census.
2. **Classifier** – detect languages, build systems, and license files.
3. **Graph Extract** – produce a minimal knowledge graph (`kg.json`).
4. **Embeddings** – generate deterministic pseudo embeddings for every file.
5. **Environment Synthesis** – surface runtime/configuration cues.
6. **Safety** – prepare SBOM/vulnerability/secrets placeholders.
7. **Runner** – draft build/test runbooks based on detected tooling.
8. **Reverse Engineer** – stage future binary/HTTP/fuzzing probes.
9. **Integrator** – emit SDK/adapter stubs and telemetry hooks.
10. **Registrar** – author `profile.json`, `system_card.md`, and summary logs.
11. **CRM Strangler** – capture proxy rollout toggles for controlled adoption.

All stages are pure Python, require no third-party packages, and execute within
this workspace or on air-gapped hosts.

## Running the pipeline

```bash
python run_pipeline.py /path/to/repo --output build/digest
```

The command analyses the given repository and writes machine readable artifacts
under the specified `--output` directory. A `summary.json` file captures
structured logs for each stage. For development convenience, the default output
path is `build/digest` within the current working directory.

## Tests

A minimal smoke test verifies that the pipeline runs end-to-end with the staged
components.

```bash
pytest
```

## Project layout

```
digest_pipeline/
  __init__.py          # exports pipeline primitives and stage classes
  pipeline.py          # orchestration base classes
  stages.py            # implementation of each pipeline agent
run_pipeline.py         # CLI entry point
tests/test_pipeline.py  # pytest smoke test
```

The pipeline emits artifacts directly into the output directory, so downstream
systems can upload or version the bundle without additional serialization
steps.
