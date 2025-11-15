#!/bin/bash
# start-all-services.sh - Start all noa_ark_os services

set -e

echo "🚀 Starting noa_ark_os Unified Services..."
echo "=========================================="

# Color codes for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"

command -v docker >/dev/null 2>&1 || { echo -e "${RED}Error: docker is not installed${NC}" >&2; exit 1; }
command -v docker-compose >/dev/null 2>&1 || { echo -e "${RED}Error: docker-compose is not installed${NC}" >&2; exit 1; }

echo -e "${GREEN}✓ Docker and Docker Compose found${NC}"

# Create necessary directories
echo -e "${YELLOW}Creating directories...${NC}"
mkdir -p data logs config

# Start services
echo -e "${YELLOW}Starting services...${NC}"

# MicroAgentStack
echo -e "${YELLOW}Starting MicroAgentStack (Orchestrator)...${NC}"
cd repos/MicroAgentStack
if [ -f docker-compose.yml ]; then
    docker-compose up -d
    echo -e "${GREEN}✓ MicroAgentStack started${NC}"
else
    echo -e "${YELLOW}⚠ No docker-compose.yml found, skipping${NC}"
fi
cd ../..

# ark-os-noa
echo -e "${YELLOW}Starting ark-os-noa (Hive Mind)...${NC}"
cd repos/ark-os-noa
if [ -f docker-compose.yml ]; then
    docker-compose up -d
    echo -e "${GREEN}✓ ark-os-noa started${NC}"
else
    echo -e "${YELLOW}⚠ No docker-compose.yml found, skipping${NC}"
fi
cd ../..

# deflex-ai-os
echo -e "${YELLOW}Starting deflex-ai-os (File Operations)...${NC}"
cd repos/deflex-ai-os
if [ -f docker-compose.yml ]; then
    docker-compose up -d
    echo -e "${GREEN}✓ deflex-ai-os started${NC}"
else
    echo -e "${YELLOW}⚠ No docker-compose.yml found, skipping${NC}"
fi
cd ../..

# Wait for services to be healthy
echo -e "${YELLOW}Waiting for services to be ready...${NC}"
sleep 10

# Check service health
echo -e "${YELLOW}Checking service health...${NC}"
echo ""
echo "Service Status:"
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
echo ""

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}✓ All services started successfully!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Access points:"
echo "  - Orchestrator: http://localhost:8000"
echo "  - Documentation: http://localhost:8080"
echo ""
echo "To view logs: docker-compose logs -f"
echo "To stop services: ./scripts/stop-all-services.sh"
echo ""
