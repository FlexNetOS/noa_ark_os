"""Router package for the unified API."""

from . import agents, analytics, chat, ci, inference, self_status, storage, workflows

__all__ = [
    "agents",
    "analytics",
    "chat",
    "ci",
    "inference",
    "self_status",
    "storage",
    "workflows",
]
