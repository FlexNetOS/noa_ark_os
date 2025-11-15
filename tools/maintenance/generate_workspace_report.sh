#!/usr/bin/env bash
#
# Workspace Monitoring and Reporting Script
# Adapted from CRC drop `scripts-1/generate_workspace_report.sh`

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
REPORT_DIR="$PROJECT_ROOT/.workspace/reports"
TIMESTAMP="$(date +"%Y%m%d_%H%M%S")"
REPORT_FILE="$REPORT_DIR/workspace_analysis_${TIMESTAMP}.md"

mkdir -p "$REPORT_DIR"
cd "$PROJECT_ROOT"

BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}🔍 Generating Comprehensive Workspace Analysis Report${NC}"
echo "=================================================="

cat > "$REPORT_FILE" <<EOF
# Workspace Analysis Report
**Generated:** $(date)
**Project:** NOA ARK OS
**Analysis ID:** ${TIMESTAMP}

---

## 📊 Executive Summary

EOF

echo -e "${BLUE}[INFO]${NC} Analysing workspace metrics..."
TOTAL_SIZE="$(du -sh . | cut -f1)"
cat >> "$REPORT_FILE" <<EOF
**Total Workspace Size:** $TOTAL_SIZE

## 📁 File Distribution

| File Type | Count | Notes |
|-----------|-------|-------|
| Rust (.rs) | $(find . -name '*.rs' -not -path './.workspace/*' | wc -l) | Source code |
| Markdown (.md) | $(find . -name '*.md' -not -path './.workspace/*' | wc -l) | Documentation |
| Config (.toml/.yaml/.json) | $(find . \\( -name '*.toml' -o -name '*.yaml' -o -name '*.json' \\) -not -path './.workspace/*' | wc -l) | Configuration files |
| Log files (.log) | $(find . -name '*.log' -not -path './.workspace/*' | wc -l) | Log artefacts |

EOF

echo -e "${BLUE}[INFO]${NC} Capturing directory footprint..."
{
    echo "## 🏗️ Directory Structure Analysis"
    echo ""
    echo "### Top 10 Largest Directories"
    echo '```'
    du -sh */ 2>/dev/null | sort -rh | head -10
    echo '```'
    echo ""
} >> "$REPORT_FILE"

echo -e "${BLUE}[INFO]${NC} Summarising phase directories..."
{
    echo "### Phase Distribution"
    for phase_dir in phase*/; do
        if [ -d "$phase_dir" ]; then
            PHASE_SIZE="$(du -sh "$phase_dir" | cut -f1)"
            PHASE_FILES="$(find "$phase_dir" -type f | wc -l)"
            echo "- **${phase_dir%/}**: $PHASE_SIZE ($PHASE_FILES files)"
        fi
    done
    echo ""
} >> "$REPORT_FILE"

echo -e "${BLUE}[INFO]${NC} Analysing documentation footprint..."
{
    echo "## 📚 Documentation Analysis"
    echo ""
    echo "### Markdown Files by Size"
    echo '```'
    find . -name "*.md" -not -path './.workspace/*' -exec du -h {} \; | sort -rh
    echo '```'
    echo ""
    README_COUNT="$(find . -name 'README.md' -not -path './.workspace/*' | wc -l)"
    echo "### Duplicate README Files"
    echo "- README files found: $README_COUNT"
    if [ "$README_COUNT" -gt 1 ]; then
        find . -name 'README.md' -not -path './.workspace/*' | while read -r file; do
            SIZE="$(du -h \"$file\" | cut -f1)"
            echo "  - \`$file\` ($SIZE)"
        done
    fi
    echo ""
} >> "$REPORT_FILE"

echo -e "${BLUE}[INFO]${NC} Inspecting build artefacts..."
{
    echo "## 🛠️ Build Artefacts Analysis"
    TARGET_DIRS="$(find . -name 'target' -type d | wc -l)"
    if [ "$TARGET_DIRS" -gt 0 ]; then
        echo "**Target directories found**: $TARGET_DIRS"
        find . -name 'target' -type d | while read -r dir; do
            SIZE="$(du -sh \"$dir\" | cut -f1)"
            echo "- \`$dir\`: $SIZE"
        done
    else
        echo "**Target directories**: None detected (✅ clean)"
    fi
    echo ""
} >> "$REPORT_FILE"

echo -e "${BLUE}[INFO]${NC} Checking for empty directories..."
EMPTY_DIRS="$(find . -type d -empty -not -path './.workspace/*' | wc -l)"
{
    echo "## 🗂️ Directory Maintenance"
    echo "- Empty directories: $EMPTY_DIRS"
    echo ""
} >> "$REPORT_FILE"

echo "✅ Report generated at $REPORT_FILE"
