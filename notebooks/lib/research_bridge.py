"""Notebook-friendly helper for calling the digest agent via the gateway.

The bridge handles the following responsibilities for researchers:

* Issue capability tokens so that every call to the gateway carries
  the correct authentication material.
* Provide small wrappers around the digest agent's `/research/*` and
  `/knowledge/*` endpoints.
* Persist analytics artefacts into ``storage/analytics/pipelines`` so
  notebook experiments stay reproducible.
* Surface access to the sanitized sample datasets shipped in
  ``.workspace/metrics``.

Typical usage inside a notebook::

    from notebooks.lib.research_bridge import ResearchBridge

    bridge = ResearchBridge()
    response = bridge.research_query(
        query="Summarise sanitized battery telemetry",
        data_sources=["research_notes_sample.csv"],
    )
    bridge.push_summary(
        pipeline_name="digest-agent-metrics",
        summary={"headline": response["message"]},
    )

The module is intentionally dependency-light so it can run in
standard notebook kernels without additional setup.
"""

from __future__ import annotations

import json
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Mapping, MutableMapping, Optional, Sequence

import requests

from core.kernel.security.tokens import issue_capability_token

__all__ = [
    "GatewayClientConfig",
    "GatewayRequestError",
    "GatewayClient",
    "ResearchBridge",
]


@dataclass(frozen=True)
class GatewayClientConfig:
    """Configuration bundle for :class:`GatewayClient`."""

    gateway_url: str = "http://localhost:8060"
    service_id: str = "digest"
    client_id: str = "research-notebook"
    fs_scope: str = "fs.analytics.write"
    network_scope: str = "net.digest"
    rate_limit_per_minute: int = 60
    token_lifetime_seconds: int = 900


class GatewayRequestError(RuntimeError):
    """Raised when a gateway call fails."""


class GatewayClient:
    """Issue capability tokens and perform signed requests to the gateway."""

    def __init__(
        self,
        config: GatewayClientConfig,
    ) -> None:
        self.config = config
        self._token: Optional[str] = None
        self._token_expires_at: float = 0.0

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------
    def _issue_token(self) -> str:
        token = issue_capability_token(
            client_id=self.config.client_id,
            fs_scopes=[self.config.fs_scope],
            network_scopes=[self.config.network_scope],
            rate_limit_per_minute=self.config.rate_limit_per_minute,
            lifetime_seconds=self.config.token_lifetime_seconds,
        )
        self._token_expires_at = time.time() + self.config.token_lifetime_seconds - 5
        return token

    def _ensure_token(self) -> str:
        if not self._token or time.time() >= self._token_expires_at:
            self._token = self._issue_token()
        assert self._token is not None  # mypy appeasement
        return self._token

    def _build_url(self, path: str) -> str:
        trimmed_gateway = self.config.gateway_url.rstrip("/")
        trimmed_path = path if path.startswith("/") else f"/{path}"
        return f"{trimmed_gateway}/{self.config.service_id}{trimmed_path}"

    def _headers(self) -> Dict[str, str]:
        token = self._ensure_token()
        return {
            "Authorization": f"Bearer {token}",
            "X-NOA-Client": self.config.client_id,
            "X-NOA-Service": self.config.service_id,
            "Content-Type": "application/json",
            "Accept": "application/json",
        }

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------
    def request(
        self,
        *,
        method: str,
        path: str,
        json_payload: Optional[Mapping[str, Any]] = None,
        params: Optional[Mapping[str, Any]] = None,
        timeout: float = 30.0,
    ) -> Dict[str, Any]:
        """Send a JSON request through the gateway and return the JSON body."""

        url = self._build_url(path)
        try:
            response = requests.request(
                method=method.upper(),
                url=url,
                headers=self._headers(),
                json=json_payload,
                params=params,
                timeout=timeout,
            )
        except requests.RequestException as exc:  # pragma: no cover - network guard
            raise GatewayRequestError(f"Gateway request failed: {exc}") from exc

        if response.status_code >= 400:
            raise GatewayRequestError(
                f"Gateway responded with {response.status_code}: {response.text.strip()}"
            )

        if not response.content:
            return {}

        try:
            return response.json()
        except json.JSONDecodeError as exc:
            raise GatewayRequestError("Gateway response was not valid JSON") from exc


class ResearchBridge:
    """Convenience façade for digest-agent notebooks."""

    def __init__(
        self,
        *,
        gateway_config: Optional[GatewayClientConfig] = None,
        analytics_root: Path | str = Path("storage/analytics/pipelines"),
        sample_root: Path | str = Path(".workspace/metrics"),
    ) -> None:
        self.gateway = GatewayClient(gateway_config or GatewayClientConfig())
        self.analytics_root = Path(analytics_root)
        self.sample_root = Path(sample_root)
        self.analytics_root.mkdir(parents=True, exist_ok=True)

    # ------------------------------------------------------------------
    # Gateway-backed helpers
    # ------------------------------------------------------------------
    def research_query(
        self,
        *,
        query: str,
        data_sources: Sequence[str],
        analysis_type: str = "exploratory",
        parameters: Optional[Mapping[str, Any]] = None,
    ) -> Dict[str, Any]:
        payload = {
            "query": query,
            "data_sources": list(data_sources),
            "analysis_type": analysis_type,
            "parameters": dict(parameters or {}),
        }
        return self.gateway.request(method="POST", path="/research/query", json_payload=payload)

    def generate_insights(
        self,
        *,
        project_id: str,
        hypotheses: Sequence[str],
        context: Optional[Mapping[str, Any]] = None,
    ) -> Dict[str, Any]:
        payload = {
            "project_id": project_id,
            "hypotheses": list(hypotheses),
            "context": dict(context or {}),
        }
        return self.gateway.request(method="POST", path="/research/insights", json_payload=payload)

    def extract_knowledge(
        self,
        *,
        source: str,
        content: str,
        metadata: Optional[Mapping[str, Any]] = None,
    ) -> Dict[str, Any]:
        payload = {
            "source": source,
            "content": content,
            "metadata": dict(metadata or {}),
        }
        return self.gateway.request(method="POST", path="/knowledge/extract", json_payload=payload)

    def synthesize_knowledge(
        self,
        *,
        topic: str,
        nodes: Sequence[Mapping[str, Any]],
        options: Optional[Mapping[str, Any]] = None,
    ) -> Dict[str, Any]:
        payload = {
            "topic": topic,
            "nodes": list(nodes),
            "options": dict(options or {}),
        }
        return self.gateway.request(method="POST", path="/knowledge/synthesize", json_payload=payload)

    # ------------------------------------------------------------------
    # Analytics persistence helpers
    # ------------------------------------------------------------------
    def push_summary(
        self,
        *,
        pipeline_name: str,
        summary: Mapping[str, Any],
        metadata: Optional[Mapping[str, Any]] = None,
        filename: str = "summary.json",
    ) -> Path:
        payload: Dict[str, Any] = {
            "summary": dict(summary),
            "metadata": dict(metadata or {}),
        }
        return self._write_pipeline_file(pipeline_name, filename, payload)

    def push_metrics(
        self,
        *,
        pipeline_name: str,
        metrics: Mapping[str, Any],
        filename: str = "metrics.json",
    ) -> Path:
        return self._write_pipeline_file(pipeline_name, filename, dict(metrics))

    def _write_pipeline_file(self, pipeline_name: str, filename: str, data: MutableMapping[str, Any]) -> Path:
        destination_dir = self.analytics_root / pipeline_name
        destination_dir.mkdir(parents=True, exist_ok=True)
        destination_file = destination_dir / filename
        destination_file.write_text(json.dumps(data, indent=2, sort_keys=True))
        return destination_file

    # ------------------------------------------------------------------
    # Sample dataset helpers
    # ------------------------------------------------------------------
    def list_sample_datasets(self) -> Sequence[Path]:
        return sorted(self.sample_root.glob("*") if self.sample_root.exists() else [])

    def sample_dataset_path(self, name: str) -> Path:
        candidate = self.sample_root / name
        if not candidate.exists():
            raise FileNotFoundError(
                f"Sample dataset '{name}' not found in {self.sample_root}. "
                "Run list_sample_datasets() to inspect options."
            )
        return candidate

