#!/bin/bash
# Clean temp files
rm -rf /mnt/c/Users/<YourUsername>/AppData/Local/Temp/*
# Move Downloads to D:
mv /mnt/c/Users/<YourUsername>/Downloads/* /mnt/d/AI-Workspace/Downloads/
# List top folders
du -h --max-depth=1 /mnt/c | sort -hr | head -20
du -h --max-depth=1 /mnt/d | sort -hr | head -20
# Prune Docker
docker system prune -a --volumes -f
# List Docker and Python envs
docker system df
find /mnt/c /mnt/d -type d -name "venv" 2>/dev/null
# (add more as you go)

