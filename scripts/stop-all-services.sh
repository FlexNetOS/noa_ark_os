#!/bin/bash
# stop-all-services.sh - Stop all noa_ark_os services

set -e

echo "🛑 Stopping noa_ark_os Services..."
echo "==================================="

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Stop MicroAgentStack
echo -e "${YELLOW}Stopping MicroAgentStack...${NC}"
cd repos/MicroAgentStack
if [ -f docker-compose.yml ]; then
    docker-compose down
    echo -e "${GREEN}✓ MicroAgentStack stopped${NC}"
fi
cd ../..

# Stop ark-os-noa
echo -e "${YELLOW}Stopping ark-os-noa...${NC}"
cd repos/ark-os-noa
if [ -f docker-compose.yml ]; then
    docker-compose down
    echo -e "${GREEN}✓ ark-os-noa stopped${NC}"
fi
cd ../..

# Stop deflex-ai-os
echo -e "${YELLOW}Stopping deflex-ai-os...${NC}"
cd repos/deflex-ai-os
if [ -f docker-compose.yml ]; then
    docker-compose down
    echo -e "${GREEN}✓ deflex-ai-os stopped${NC}"
fi
cd ../..

echo -e "${GREEN}==================================="
echo -e "✓ All services stopped${NC}"
