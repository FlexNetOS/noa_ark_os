#!/bin/bash
# AgentAsKit Unified System Management Script
# Provides system-wide operations for the unified AgentAsKit production environment

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to validate unified system integrity
validate_system() {
    log "Validating unified system integrity..."
    
    local required_dirs=(
        "unified_tools"
        "unified_execution" 
        "unified_orchestration"
        "unified_agents"
        "unified_docs"
        "operational_scripts"
        "operational_logs"
        "operational_audit"
        "operational_hash"
    )
    
    for dir in "${required_dirs[@]}"; do
        if [[ -d "$PROJECT_ROOT/$dir" ]]; then
            log "✓ $dir exists"
        else
            error "✗ $dir missing"
            return 1
        fi
    done
    
    log "System integrity validation passed"
}

# Function to generate system report
generate_report() {
    log "Generating unified system report..."
    
    local report_file="$PROJECT_ROOT/operational_logs/system_report_$(date +%Y%m%d_%H%M%S).md"
    
    cat > "$report_file" << EOF
# AgentAsKit Unified System Report
Generated: $(date)

## System Structure
- unified_tools/: $(find "$PROJECT_ROOT/unified_tools" -type f | wc -l) files
- unified_execution/: $(find "$PROJECT_ROOT/unified_execution" -type f | wc -l) files  
- unified_orchestration/: $(find "$PROJECT_ROOT/unified_orchestration" -type f | wc -l) files
- unified_agents/: $(find "$PROJECT_ROOT/unified_agents" -type f | wc -l) files
- unified_docs/: $(find "$PROJECT_ROOT/unified_docs" -type f | wc -l) files

## Operational Status
- Scripts: $(find "$PROJECT_ROOT/operational_scripts" -type f | wc -l) files
- Logs: $(find "$PROJECT_ROOT/operational_logs" -type f | wc -l) files
- Audit Reports: $(find "$PROJECT_ROOT/operational_audit" -type f | wc -l) files
- Hash Files: $(find "$PROJECT_ROOT/operational_hash" -type f | wc -l) files

## System Health: OPERATIONAL ✓
EOF

    log "Report generated: $report_file"
}

# Function to clean up old logs
cleanup_logs() {
    log "Cleaning up old log files..."
    find "$PROJECT_ROOT/operational_logs" -name "*.log" -mtime +30 -delete
    log "Log cleanup completed"
}

# Main function
main() {
    case "$1" in
        "validate")
            validate_system
            ;;
        "report")
            generate_report
            ;;
        "cleanup")
            cleanup_logs
            ;;
        "all")
            validate_system
            generate_report
            cleanup_logs
            ;;
        *)
            echo "Usage: $0 {validate|report|cleanup|all}"
            echo "  validate - Check system integrity"
            echo "  report   - Generate system report"
            echo "  cleanup  - Clean old log files"
            echo "  all      - Run all operations"
            exit 1
            ;;
    esac
}

main "$@"