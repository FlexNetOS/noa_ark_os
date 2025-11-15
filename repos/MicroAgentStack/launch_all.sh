#!/bin/bash

echo "Activating Python venv..."
source venv/bin/activate

echo "Starting Ollama LLM server..."
ollama serve > ollama.log 2>&1 &

# Optional: Start other LLM/MLLM services if you have them (uncomment and edit as needed)
# echo "Starting MLLM-Vision server..."
# python3 path/to/mllm_vision_server.py > mllm_vision.log 2>&1 &

echo "Launching FastAPI orchestrator (can act as CEO/board interface)..."
uvicorn main:app --reload > orchestrator.log 2>&1 &

echo "Launching file watcher for continuous manifest-driven automation..."
python3 watch_manifest.py > watcher.log 2>&1 &

# If you want, also launch Director/CommanderChief agents for each stack, or let orchestrator spin them up as needed:
# python3 agents/CommanderChiefAgent_DataStack/main.py > datastack_director.log 2>&1 &
# python3 agents/CommanderChiefAgent_DevOps/main.py > devops_director.log 2>&1 &

echo "All services started!"
echo "Monitor logs with: tail -f ollama.log orchestrator.log watcher.log"
echo "Happy autonomous agent orchestration! 🚀"
