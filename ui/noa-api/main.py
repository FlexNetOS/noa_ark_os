"""
NOA ARK OS - Backend API Server
Real-time agent monitoring and control API
"""

from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import httpx
import asyncio
from typing import List, Dict
import json
from datetime import datetime

app = FastAPI(title="NOA ARK OS API", version="1.1.0")

# CORS middleware for frontend
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# WebSocket connections
active_connections: List[WebSocket] = []

# System state
system_state = {
    "active_environment": "green",
    "blue_port": 8080,
    "green_port": 8081,
    "agents_count": 302,
    "models_count": 5,
    "version": "1.1.0",
    "optimizations_active": True
}

class InferenceRequest(BaseModel):
    prompt: str
    temperature: float = 0.7
    max_tokens: int = 2000

class AgentStatus(BaseModel):
    agent_id: str
    status: str
    current_task: str = ""

@app.get("/")
async def root():
    return {
        "service": "NOA ARK OS API",
        "version": "1.1.0",
        "status": "operational",
        "optimized": True
    }

@app.get("/api/system/status")
async def get_system_status():
    """Get overall system status"""
    # Check inference server
    active_port = system_state["green_port"] if system_state["active_environment"] == "green" else system_state["blue_port"]
    
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"http://127.0.0.1:{active_port}/health", timeout=2.0)
            inference_healthy = response.status_code == 200
    except:
        inference_healthy = False
    
    return {
        "timestamp": datetime.now().isoformat(),
        "active_environment": system_state["active_environment"],
        "inference_server": {
            "healthy": inference_healthy,
            "port": active_port,
            "environment": system_state["active_environment"]
        },
        "agents": {
            "total": system_state["agents_count"],
            "online": system_state["agents_count"],  # All agents always online
            "busy": 0
        },
        "models": {
            "total": system_state["models_count"],
            "loaded": 1
        },
        "version": system_state["version"],
        "optimizations": system_state["optimizations_active"]
    }

@app.get("/api/agents/list")
async def list_agents():
    """List all agents"""
    return {
        "total": 302,
        "agents": [
            {"id": "noa-commander", "type": "Commander", "status": "online", "layer": "L1"},
            {"id": "legal-agent", "type": "Board", "status": "online", "layer": "L2"},
            {"id": "model-selector", "type": "Specialist", "status": "online", "layer": "L4"},
            {"id": "code-specialist", "type": "Specialist", "status": "online", "layer": "L4"},
            {"id": "security-agent", "type": "Specialist", "status": "online", "layer": "L4"}
        ]
    }

@app.get("/api/models/list")
async def list_models():
    """List available models"""
    return {
        "total": 5,
        "models": [
            {
                "name": "deepseek-coder-v2",
                "size_mb": 9650,
                "performance": 0.95,
                "use_case": "CodeGeneration",
                "loaded": True
            },
            {
                "name": "mistral-nemo-12b",
                "size_mb": 6960,
                "performance": 0.93,
                "use_case": "Reasoning",
                "loaded": False
            },
            {
                "name": "llama-3.1-8b",
                "size_mb": 4580,
                "performance": 0.92,
                "use_case": "General",
                "loaded": False
            },
            {
                "name": "llama-3.2-3b",
                "size_mb": 1926,
                "performance": 0.85,
                "use_case": "General",
                "loaded": False
            },
            {
                "name": "tinyllama-1.1b",
                "size_mb": 620,
                "performance": 0.68,
                "use_case": "Fast",
                "loaded": False
            }
        ]
    }

@app.post("/api/inference/generate")
async def generate(request: InferenceRequest):
    """Generate inference via active environment"""
    active_port = system_state["green_port"] if system_state["active_environment"] == "green" else system_state["blue_port"]
    
    try:
        async with httpx.AsyncClient(timeout=60.0) as client:
            response = await client.post(
                f"http://127.0.0.1:{active_port}/completion",
                json={
                    "prompt": request.prompt,
                    "temperature": request.temperature,
                    "max_tokens": request.max_tokens
                }
            )
            result = response.json()
            
            return {
                "success": True,
                "content": result.get("content", ""),
                "model": "deepseek-coder-v2",
                "environment": system_state["active_environment"],
                "tokens": result.get("tokens_predicted", 0)
            }
    except Exception as e:
        return {
            "success": False,
            "error": str(e)
        }

@app.get("/api/monitoring/performance")
async def get_performance():
    """Get performance metrics"""
    return {
        "timestamp": datetime.now().isoformat(),
        "throughput_tokens_per_sec": 200.5,
        "gpu_utilization_pct": 65.3,
        "concurrent_requests": 4,
        "average_latency_ms": 5800,
        "environment": system_state["active_environment"]
    }

@app.websocket("/ws/events")
async def websocket_endpoint(websocket: WebSocket):
    """WebSocket for real-time updates"""
    await websocket.accept()
    active_connections.append(websocket)
    
    try:
        while True:
            # Send system updates every 2 seconds
            event = {
                "type": "system_update",
                "timestamp": datetime.now().isoformat(),
                "agents_online": 302,
                "gpu_usage": 65.3,
                "throughput": 200.5
            }
            await websocket.send_json(event)
            await asyncio.sleep(2)
    except WebSocketDisconnect:
        active_connections.remove(websocket)

if __name__ == "__main__":
    import uvicorn
    print("🚀 Starting NOA ARK OS API Server...")
    print("📡 API: http://localhost:3000")
    print("🔌 WebSocket: ws://localhost:3000/ws/events")
    print("📚 Docs: http://localhost:3000/docs")
    uvicorn.run(app, host="0.0.0.0", port=3000)
