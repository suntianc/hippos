#!/bin/bash
# plan-reread.sh - Force re-read planning files before making changes
# Trigger: PreToolUse (on Write, Edit, Bash tools)
# Function: Ensure AI stays aligned with the current plan

# Get tool name from arguments or environment
tool_name="${1:-$CLAUDE_TOOL_NAME}"

# Only trigger for specific tools that make changes
CHANGE_TOOLS=("Write" "Edit" "Bash" "write" "edit" "bash")

# Check if current tool is in change tools list
is_change_tool=false
for tool in "${CHANGE_TOOLS[@]}"; do
    if [[ "$tool_name" == "$tool" ]]; then
        is_change_tool=true
        break
    fi
done

if [[ "$is_change_tool" == false ]]; then
    exit 0
fi

# Planning files to check
PLANNING_FILES=(
    "planning/task_plan.md"
    "planning/progress.md"
    "planning/findings.md"
    ".claude/planning/task_plan.md"
    ".claude/planning/progress.md"
)

found_files=()
for file in "${PLANNING_FILES[@]}"; do
    if [[ -f "$file" ]]; then
        found_files+=("$file")
    fi
done

if [[ ${#found_files[@]} -gt 0 ]]; then
    echo "## [Reminder] Planning Files Available"
    echo ""
    echo "Before making changes, ensure alignment with current plan:"
    echo ""

    for file in "${found_files[@]}"; do
        echo "- \`$file\`"
    done

    echo ""
    echo "### Quick Check"
    echo ""
    echo "1. Is this change part of the current task in the plan?"
    echo "2. Have you updated progress.md with your progress?"
    echo "3. Are you following the approach defined in task_plan.md?"
    echo ""
    echo "**If unsure, re-read the planning files before proceeding.**"
fi
