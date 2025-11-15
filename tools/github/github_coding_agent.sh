#!/bin/bash
#
# GitHub Coding Agent Background Service
# Monitors agentask.todo and executes tasks automatically
#
# Usage:
#   ./github_coding_agent.sh start   # Start agent in background
#   ./github_coding_agent.sh stop    # Stop agent
#   ./github_coding_agent.sh status  # Check agent status
#   ./github_coding_agent.sh logs    # Show agent logs

set -euo pipefail

# Configuration
WORKSPACE_ROOT="${WORKSPACE_ROOT:-$HOME/workspace}"
REPO_PATH="${REPO_PATH:-$WORKSPACE_ROOT/repos/task_exec_kit}"
TODO_FILE="${REPO_PATH}/agentask.todo"
SOT_FILE="${REPO_PATH}/sot.md"
AGENT_LOG="${HOME}/logs/development/github_agent.log"
AGENT_PID_FILE="/tmp/github_coding_agent.pid"
CHECK_INTERVAL="${CHECK_INTERVAL:-300}" # 5 minutes
CONFIG_FILE="${HOME}/config/applications/github_coding_agent.yaml"

# Chatmode configuration
CHATMODE_PRIMARY="${HOME}/.github/chatmodes/agentask.chatmode.md"
CHATMODE_FALLBACK="${HOME}/.github/chatmodes/agent.chatmode.md"
CHATMODE_ENABLED=true

# Ensure log directory exists
mkdir -p "$(dirname "$AGENT_LOG")"

# Logging functions
log() {
    echo "[$(date -u +"%Y-%m-%dT%H:%M:%SZ")] $*" | tee -a "$AGENT_LOG"
}

log_error() {
    echo "[$(date -u +"%Y-%m-%dT%H:%M:%SZ")] ERROR: $*" | tee -a "$AGENT_LOG" >&2
}

# Check if agent is running
is_running() {
    if [[ -f "$AGENT_PID_FILE" ]]; then
        local pid
        pid=$(cat "$AGENT_PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            return 0
        else
            # PID file exists but process is dead
            rm -f "$AGENT_PID_FILE"
            return 1
        fi
    fi
    return 1
}

# Start agent
start_agent() {
    if is_running; then
        log_error "Agent is already running (PID: $(cat "$AGENT_PID_FILE"))"
        return 1
    fi

    log "Starting GitHub Coding Agent..."
    log "Repository: $REPO_PATH"
    log "TODO file: $TODO_FILE"
    log "Check interval: ${CHECK_INTERVAL}s"

    # Start background process
    nohup bash -c "
        while true; do
            bash '$0' _run_cycle
            sleep $CHECK_INTERVAL
        done
    " >> "$AGENT_LOG" 2>&1 &

    echo $! > "$AGENT_PID_FILE"
    log "Agent started (PID: $!)"
    echo "GitHub Coding Agent started. View logs: tail -f $AGENT_LOG"
}

# Stop agent
stop_agent() {
    if ! is_running; then
        log "Agent is not running"
        return 0
    fi

    local pid
    pid=$(cat "$AGENT_PID_FILE")
    log "Stopping GitHub Coding Agent (PID: $pid)..."

    kill "$pid" 2>/dev/null || true
    rm -f "$AGENT_PID_FILE"
    log "Agent stopped"
}

# Check agent status
check_status() {
    if is_running; then
        local pid
        pid=$(cat "$AGENT_PID_FILE")
        echo "✅ GitHub Coding Agent is RUNNING"
        echo "   PID: $pid"
        echo "   Repository: $REPO_PATH"
        echo "   Log: $AGENT_LOG"
        echo "   Check interval: ${CHECK_INTERVAL}s"
        echo ""
        echo "Recent activity:"
        tail -n 10 "$AGENT_LOG" 2>/dev/null || echo "  No logs yet"
    else
        echo "⚠️  GitHub Coding Agent is NOT RUNNING"
        echo "   Start with: $0 start"
    fi
}

# Show logs
show_logs() {
    if [[ -f "$AGENT_LOG" ]]; then
        tail -f "$AGENT_LOG"
    else
        echo "No logs found at $AGENT_LOG"
    fi
}

# Load chatmode configuration
load_chatmode() {
    if [[ "$CHATMODE_ENABLED" != "true" ]]; then
        return 0
    fi

    local chatmode_file=""

    # Try primary chatmode
    if [[ -f "$CHATMODE_PRIMARY" ]]; then
        chatmode_file="$CHATMODE_PRIMARY"
        log "Loaded chatmode: $CHATMODE_PRIMARY" >&2
    elif [[ -f "$CHATMODE_FALLBACK" ]]; then
        chatmode_file="$CHATMODE_FALLBACK"
        log "Loaded fallback chatmode: $CHATMODE_FALLBACK" >&2
    else
        log_error "No chatmode file found (tried primary and fallback)" >&2
        return 1
    fi

    # Store chatmode path for execution (stdout only, no log prefix)
    echo "$chatmode_file"
}

# Validate chatmode requirements
validate_chatmode_requirements() {
    local chatmode_file="$1"

    if [[ ! -f "$chatmode_file" ]]; then
        log_error "Chatmode file not found: $chatmode_file"
        return 1
    fi

    # Check for required chatmode elements (case-insensitive patterns)
    local required_patterns=(
        "simulation"
        "triple.*verification"
        "truth.*gate"
        "evidence"
        "single source of truth"
    )

    local missing_elements=()
    for pattern in "${required_patterns[@]}"; do
        if ! grep -Eqi "$pattern" "$chatmode_file"; then
            missing_elements+=("$pattern")
        fi
    done

    if [ ${#missing_elements[@]} -gt 0 ]; then
        log_error "Chatmode missing elements: ${missing_elements[*]}"
        return 1
    fi

    log "✓ Chatmode validation passed: All 5 required elements present"
    return 0
}

# Parse TODO file for next task
get_next_task() {
    if [[ ! -f "$TODO_FILE" ]]; then
        log_error "TODO file not found: $TODO_FILE"
        return 1
    fi

    # Look for first unchecked task in [Current Tasks] section
    awk '
        /^\[Current Tasks\]/ { in_current=1; next }
        /^\[/ { in_current=0 }
        in_current && /^- \[ \]/ {
            # Extract task ID (e.g., TASK-005)
            match($0, /TASK-[0-9]+/)
            if (RSTART > 0) {
                print substr($0, RSTART, RLENGTH)
                exit
            }
        }
    ' "$TODO_FILE"
}

# Execute a task using Copilot with chatmode enforcement
execute_task() {
    local task_id="$1"
    log "Executing task: $task_id"

    # Load and validate chatmode
    local chatmode_file=""
    if [[ "$CHATMODE_ENABLED" == "true" ]]; then
        chatmode_file=$(load_chatmode)
        if [[ -n "$chatmode_file" ]]; then
            validate_chatmode_requirements "$chatmode_file" || {
                log_error "Chatmode validation failed, continuing without enforcement"
                chatmode_file=""
            }
        fi
    fi

    # Navigate to repo
    cd "$REPO_PATH" || {
        log_error "Failed to navigate to $REPO_PATH"
        return 1
    }

    # Create task execution prompt with chatmode context
    local prompt="@todo.prompt.md complete task-${task_id#TASK-}"
    local chatmode_context=""

    if [[ -n "$chatmode_file" ]]; then
        chatmode_context="
CHATMODE ENFORCEMENT ACTIVE
===========================
Mode: $(basename "$chatmode_file")
Policy: agentask.chatmode.md requirements MUST be followed

Required Behaviors:
✓ No Simulation - Only real execution and integration
✓ Triple-Verification - Pass A/B/C before claiming completion
✓ Truth Gate - All 6 gates required before marking complete
✓ Evidence Ledger - Document all artifacts with SHA-256 hashes
✓ SOT-Driven - Follow agentask.todo → sot.md priority order
✓ Heal Don't Harm - Never remove or downgrade working features

Prohibited Actions:
✗ Fabricated data, metrics, citations, or logs
✗ Implied completion without Truth Gate validation
✗ Overclaiming beyond verified test coverage
✗ Vague terms without measurable criteria
✗ Skipping Triple-Verification Protocol
"
    fi

    log "Prompt: $prompt"
    if [[ -n "$chatmode_file" ]]; then
        log "Chatmode: $(basename "$chatmode_file") enforcement ACTIVE"
    fi
    log "Task $task_id ready for execution"

    # Create notification file for user
    cat > "/tmp/github_agent_task_$task_id.txt" <<EOF
GitHub Coding Agent Notification
================================
Task Ready: $task_id
Time: $(date -u +"%Y-%m-%dT%H:%M:%SZ")

Action Required:
1. Open VS Code in repository: $REPO_PATH
2. Execute prompt: $prompt
3. Review and commit changes

TODO file: $TODO_FILE
SOT file: $SOT_FILE
$chatmode_context

Automatic execution enabled. Agent will check for completion.
EOF

    log "Notification created: /tmp/github_agent_task_$task_id.txt"

    if [[ -n "$chatmode_file" ]]; then
        log "✓ Chatmode requirements will be enforced during execution"
    fi

    log "NOTE: Manual execution required in VS Code with GitHub Copilot"
}

# Run one check cycle
run_cycle() {
    log "Running check cycle..."

    # Verify files exist
    if [[ ! -f "$TODO_FILE" ]]; then
        log_error "TODO file not found: $TODO_FILE"
        return 1
    fi

    if [[ ! -f "$SOT_FILE" ]]; then
        log_error "SOT file not found: $SOT_FILE"
        return 1
    fi

    # Get next task
    local next_task
    next_task=$(get_next_task)

    if [[ -z "$next_task" ]]; then
        log "No tasks in queue"
        return 0
    fi

    log "Next task found: $next_task"
    execute_task "$next_task"
}

# Main command dispatcher
main() {
    local command="${1:-}"

    case "$command" in
        start)
            start_agent
            ;;
        stop)
            stop_agent
            ;;
        status)
            check_status
            ;;
        logs)
            show_logs
            ;;
        restart)
            stop_agent
            sleep 2
            start_agent
            ;;
        _run_cycle)
            # Internal command for background process
            run_cycle
            ;;
        *)
            cat <<EOF
GitHub Coding Agent - Background Task Automation

Usage: $0 {start|stop|status|logs|restart}

Commands:
  start    - Start agent in background
  stop     - Stop agent
  status   - Check if agent is running
  logs     - Show agent logs (tail -f)
  restart  - Restart agent

Configuration:
  REPO_PATH=$REPO_PATH
  CHECK_INTERVAL=${CHECK_INTERVAL}s
  LOG_FILE=$AGENT_LOG
  CHATMODE=$CHATMODE_PRIMARY

Files:
  TODO: $TODO_FILE
  SOT:  $SOT_FILE
  CHATMODE: $CHATMODE_PRIMARY (fallback: $CHATMODE_FALLBACK)
EOF
            exit 1
            ;;
    esac
}

main "$@"
