#!/bin/bash
# session-start.sh - Display project status at session start
# Trigger: Every time Claude Code session starts

# Get Git info
get_git_info() {
    local branch uncommitted_count last_commit
    branch=$(git branch --show-current 2>/dev/null || echo "unknown")
    local status=$(git status --porcelain 2>/dev/null)
    uncommitted_count=$(echo "$status" | grep -c '.' 2>/dev/null || echo "0")
    last_commit=$(git log -1 --oneline 2>/dev/null || echo "unknown")

    echo "$branch|$uncommitted_count|$last_commit"
}

# Get TODO count from docs/TODO.md
get_todo_count() {
    local todo_path="docs/TODO.md"
    if [[ -f "$todo_path" ]]; then
        local content=$(cat "$todo_path")
        local pending=$(echo "$content" | grep -c '\- \[ \]' 2>/dev/null || echo "0")
        local completed=$(echo "$content" | grep -c '\- \[x\]' 2>/dev/null || echo "0")
        echo "$pending|$completed"
    else
        echo "0|0"
    fi
}

# Main execution
git_info=$(get_git_info)
todo_count=$(get_todo_count)

IFS='|' read -r branch uncommitted_count last_commit <<< "$git_info"
IFS='|' read -r pending_todos completed_todos <<< "$todo_count"

now=$(date "+%Y/%m/%d %H:%M:%S")

# Determine uncommitted status
if [[ "$uncommitted_count" -gt 0 ]]; then
    uncommitted_status="WARNING: Uncommitted changes ($uncommitted_count files)"
else
    uncommitted_status="OK: No uncommitted changes"
fi

# Format last commit info
if [[ "$last_commit" != "unknown" ]]; then
    last_commit_info="Latest commit: $last_commit"
else
    last_commit_info=""
fi

echo "## ZhangNote Session Started"
echo ""
echo "**Time**: $now"
echo "**Git Branch**: \`$branch\`"
echo ""
echo "$uncommitted_status"
echo ""
echo "**TODO**: $pending_todos pending / $completed_todos completed"
echo ""
if [[ -n "$last_commit_info" ]]; then
    echo "$last_commit_info"
fi
echo ""
echo "**Quick Commands**:"
echo "| /start | Quick project overview |"
echo "| /progress | View detailed progress |"
echo "| /next | Get next step suggestions |"
echo "| /update-status | Update project status |"
