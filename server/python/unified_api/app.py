"""Unified FastAPI gateway for the consolidated UI experience."""
from __future__ import annotations

from typing import Dict

from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware

from .event_bus import GLOBAL_EVENT_BUS
from .routers import agents, analytics, chat, ci, inference, self_status, storage, workflows


def create_app() -> FastAPI:
    app = FastAPI(title="NOA Unified Gateway", version="1.0.0")

    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    app.include_router(workflows.router, prefix="/api/workflows", tags=["workflows"])
    app.include_router(agents.router, prefix="/api/agents", tags=["agents"])
    app.include_router(ci.router, prefix="/api/ci", tags=["ci"])
    app.include_router(storage.router, prefix="/api/storage", tags=["storage"])
    app.include_router(inference.router, prefix="/api/inference", tags=["inference"])
    app.include_router(chat.router, prefix="/api/chat", tags=["chat"])
    app.include_router(analytics.router, prefix="/api/analytics", tags=["analytics"])
    app.include_router(self_status.router, prefix="/api", tags=["self"])

    @app.get("/api/health", tags=["meta"])
    async def health() -> Dict[str, str]:
        return {"status": "ok"}

    @app.websocket("/ws/{channel}")
    async def websocket_events(channel: str, websocket: WebSocket) -> None:
        await websocket.accept()
        try:
            async for event in GLOBAL_EVENT_BUS.subscribe(channel):
                await websocket.send_json(event)
        except WebSocketDisconnect:
            return

    return app


app = create_app()
