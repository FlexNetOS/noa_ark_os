#!/bin/bash
#
# Ollama Model Manager for task_exec_kit
# Manages model lifecycle, benchmarking, and registry updates
#

set -euo pipefail

MODELS_YAML="${MODELS_YAML:-$HOME/workspace/repos/task_exec_kit/implementation/config/ollama/models.yaml}"
OLLAMA_ENDPOINT="${OLLAMA_ENDPOINT:-http://localhost:11434}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}✓${NC} $*"
}

error() {
    echo -e "${RED}✗${NC} $*" >&2
}

warn() {
    echo -e "${YELLOW}⚠${NC} $*"
}

# List all installed models
cmd_list() {
    echo "=== Installed Ollama Models ==="
    ollama list
    echo ""
    echo "=== Registry Status ==="
    if command -v yq >/dev/null 2>&1; then
        yq '.registry[] | "- " + .name + " (" + .model_id + ") - " + .status' "$MODELS_YAML"
    else
        warn "yq not installed, showing raw YAML"
        grep "name:" "$MODELS_YAML" || echo "No models in registry"
    fi
}

# Bulk pull models by tier
cmd_bulk_pull() {
    local tier="${1:-}"
    if [[ -z "$tier" ]]; then
        error "Usage: $0 bulk-pull <tier>"
        echo "Tiers: tiny, small, medium, large, all"
        return 1
    fi

    log "Bulk pulling models for tier: $tier"

    if ! command -v yq >/dev/null 2>&1; then
        error "yq required for bulk operations. Install: snap install yq"
        return 1
    fi

    local models=()
    if [[ "$tier" == "all" ]]; then
        mapfile -t models < <(yq '.registry[] | select(.source == "ollama") | .model_id' "$MODELS_YAML")
    else
        mapfile -t models < <(yq ".registry[] | select(.source == \"ollama\" and .tier == \"$tier\") | .model_id" "$MODELS_YAML")
    fi

    local total=${#models[@]}
    local success=0
    local failed=0

    log "Found $total models to pull"

    for model in "${models[@]}"; do
        log "Pulling $model..."
        if ollama pull "$model"; then
            ((success++))
        else
            error "Failed to pull $model"
            ((failed++))
        fi
    done

    echo ""
    log "Summary: $success succeeded, $failed failed (out of $total)"
}

# Pull a new model
cmd_pull() {
    local model="${1:-}"
    if [[ -z "$model" ]]; then
        error "Usage: $0 pull <model_name>"
        echo "Examples:"
        echo "  $0 pull llama3.2:1b"
        echo "  $0 pull mistral:7b"
        echo "  $0 pull qwen2.5:7b"
        exit 1
    fi

    log "Pulling model: $model"
    if ollama pull "$model"; then
        log "Successfully pulled $model"
        log "Updating registry..."
        # Add to registry if yq available
        if command -v yq >/dev/null 2>&1; then
            yq eval ".registry += [{\"name\": \"$model\", \"model_id\": \"$model\", \"source\": \"ollama\", \"status\": \"active\", \"installed_date\": \"$(date +%Y-%m-%d)\"}]" -i "$MODELS_YAML"
            log "Registry updated"
        else
            warn "yq not available, manual registry update needed"
        fi
    else
        error "Failed to pull $model"
        exit 1
    fi
}

# Benchmark a model's performance
cmd_benchmark() {
    local model="${1:-llama3.2:1b}"
    log "Benchmarking $model..."

    # Create temporary prompt file
    local prompt="Write a simple 'hello world' program in Python with comments"

    echo "Running inference test..."
    local output
    output=$(ollama run "$model" "$prompt" --verbose 2>&1)

    # Extract metrics
    local tokens_per_sec=$(echo "$output" | grep "eval rate:" | awk '{print $3}')
    local total_duration=$(echo "$output" | grep "total duration:" | awk '{print $3}')
    local eval_count=$(echo "$output" | grep "eval count:" | awk '{print $3}')

    echo ""
    echo "=== Benchmark Results ==="
    echo "Model: $model"
    echo "Total Duration: $total_duration"
    echo "Tokens Generated: $eval_count"
    echo "Generation Speed: $tokens_per_sec tokens/s"
    echo ""

    # Update registry with performance data
    if command -v yq >/dev/null 2>&1; then
        yq eval "(.registry[] | select(.model_id == \"$model\").performance.tokens_per_second) = $tokens_per_sec" -i "$MODELS_YAML"
        yq eval "(.registry[] | select(.model_id == \"$model\").performance.last_benchmark) = \"$(date -Iseconds)\"" -i "$MODELS_YAML"
        log "Performance metrics updated in registry"
    fi
}

# Check Ollama service status
cmd_status() {
    echo "=== Ollama Service Status ==="

    # Check if service is running
    if pgrep -f "ollama serve" > /dev/null; then
        log "Service: RUNNING"
        local pid=$(pgrep -f "ollama serve")
        echo "  PID: $pid"
    else
        error "Service: NOT RUNNING"
        echo "  Start with: ollama serve &"
        return 1
    fi

    echo ""
    echo "=== API Health Check ==="
    if curl -sf "$OLLAMA_ENDPOINT/api/tags" > /dev/null; then
        log "API: RESPONDING"
        echo "  Endpoint: $OLLAMA_ENDPOINT"
    else
        error "API: NOT RESPONDING"
        return 1
    fi

    echo ""
    echo "=== Loaded Models ==="
    curl -s "$OLLAMA_ENDPOINT/api/tags" | jq -r '.models[] | "- \(.name) (\(.size / 1024 / 1024 / 1024 | floor)GB)"' 2>/dev/null || ollama list

    echo ""
    echo "=== Disk Usage ==="
    local models_dir="${OLLAMA_MODELS:-$HOME/.ollama/models}"
    if [[ -d "$models_dir" ]]; then
        local usage=$(du -sh "$models_dir" 2>/dev/null | cut -f1)
        echo "  Models directory: $models_dir"
        echo "  Total size: ${usage:-unknown}"
    fi
}

# Remove a model
cmd_remove() {
    local model="${1:-}"
    if [[ -z "$model" ]]; then
        error "Usage: $0 remove <model_name>"
        exit 1
    fi

    warn "Removing model: $model"
    if ollama rm "$model"; then
        log "Successfully removed $model"
        # Update registry status
        if command -v yq >/dev/null 2>&1; then
            yq eval "(.registry[] | select(.model_id == \"$model\").status) = \"removed\"" -i "$MODELS_YAML"
        fi
    else
        error "Failed to remove $model"
        exit 1
    fi
}

# Show registry
cmd_registry() {
    echo "=== Model Registry ==="
    if command -v yq >/dev/null 2>&1; then
        yq '.' "$MODELS_YAML"
    else
        cat "$MODELS_YAML"
    fi
}

# Main command dispatcher
main() {
    local command="${1:-help}"

    case "$command" in
        list|ls)
            cmd_list
            ;;
        pull)
            shift
            cmd_pull "$@"
            ;;
        bulk-pull|bulk)
            shift
            cmd_bulk_pull "$@"
            ;;
        benchmark|bench)
            shift
            cmd_benchmark "$@"
            ;;
        status)
            cmd_status
            ;;
        remove|rm)
            shift
            cmd_remove "$@"
            ;;
        registry)
            cmd_registry
            ;;
        help|--help|-h)
            cat <<EOF
Ollama Model Manager for task_exec_kit

Usage: $(basename "$0") COMMAND [OPTIONS]

Commands:
  list, ls              List installed models and registry status
  pull MODEL            Pull and install a new model
  bulk-pull TIER        Bulk pull models by tier (tiny|small|medium|large|all)
  benchmark MODEL       Benchmark model performance
  status                Check Ollama service status and health
  remove MODEL          Remove an installed model
  registry              Show full model registry
  help                  Show this help message

Examples:
  $(basename "$0") list
  $(basename "$0") pull llama3.2:1b
  $(basename "$0") bulk-pull tiny
  $(basename "$0") benchmark llama3.2:1b
  $(basename "$0") status
  $(basename "$0") remove old-model:latest

Environment Variables:
  MODELS_YAML           Path to models registry (default: ~/workspace/repos/task_exec_kit/implementation/config/ollama/models.yaml)
  OLLAMA_ENDPOINT       Ollama API endpoint (default: http://localhost:11434)

EOF
            ;;
        *)
            error "Unknown command: $command"
            echo "Run '$0 help' for usage information"
            exit 1
            ;;
    esac
}

main "$@"
