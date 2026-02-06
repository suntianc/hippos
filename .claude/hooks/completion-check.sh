#!/bin/bash
# completion-check.sh - Task completion check
# Trigger: Stop (at session end)
# Function: Ensure documentation updates, correct task completion, follow doc sync rules

# Planning file paths
TASK_PLAN="planning/task_plan.md"
PROGRESS="planning/progress.md"
FINDINGS="planning/findings.md"

# Check if planning files exist
planning_exists=false
progress_exists=false

if [[ -f "$TASK_PLAN" ]] || [[ -f ".claude/planning/task_plan.md" ]]; then
    planning_exists=true
fi

if [[ -f "$PROGRESS" ]] || [[ -f ".claude/planning/progress.md" ]]; then
    progress_exists=true
fi

echo "## [Pre-Completion Check] Task Verification Checklist"
echo ""

# ============================================
# Documentation Sync Check (from 05-documentation.md)
# ============================================
echo "### 1. Documentation Sync Check"
echo ""
echo "If this session involves the following changes, you **must** update related documentation:"
echo ""
echo "| Change Type | Documents to Update |"
echo "|-------------|---------------------|"
echo "| API endpoint changes | \`docs/API.md\`, README sections |"
echo "| Component props/interface changes | Component docs, type definitions |"
echo "| Database schema changes | DB docs, type definitions |"
echo "| New/removed features | Feature docs, user guides |"
echo "| Config/env variable changes | \`README.md\`, \`.env.template\` |"
echo "| Architecture changes | \`docs/ARCHITECTURE.md\`, system diagrams |"
echo ""
echo "**Self-check: Did this session trigger any of the above conditions?**"
echo ""

# ============================================
# Planning File Status
# ============================================
if [[ "$planning_exists" == true ]] || [[ "$progress_exists" == true ]]; then
    echo "### 2. Planning File Status"
    echo ""

    if [[ "$planning_exists" == true ]]; then
        echo "- [ ] **Task Plan**: Has task status been updated?"
    fi

    if [[ "$progress_exists" == true ]]; then
        echo "- [ ] **Progress File**: Is progress record up to date?"
    fi

    if [[ -f "$FINDINGS" ]]; then
        echo "- [ ] **Findings**: Have key findings been recorded?"
    fi

    echo ""
fi

# ============================================
# Code Quality Verification
# ============================================
echo "### 3. Code Quality Verification"
echo ""
echo "Before completing the session, verify the following:"
echo ""
echo "**Code Checks**"
echo "- [ ] No TypeScript/linting errors (run \`lsp_diagnostics\`)"
echo "- [ ] No \`as any\`, \`@ts-ignore\` or type suppression"
echo "- [ ] All new code follows existing patterns"
echo ""
echo "**Test Verification**"
echo "- [ ] Build passes (if applicable)"
echo "- [ ] Tests pass (if applicable)"
echo ""

# ============================================
# Cleanup Check
# ============================================
echo "### 4. Cleanup Check"
echo ""
echo "- [ ] Temporary files deleted"
echo "- [ ] Debug code removed"
echo "- [ ] console.log statements cleaned up"
echo ""

# ============================================
# Incomplete Task Handling
# ============================================
echo "### If Task is Incomplete"
echo ""
echo "If task is not fully complete, record in \`progress.md\`:"
echo ""
echo "- Completed content"
echo "- Remaining todo items"
echo "- Any blocking issues or decisions needed"
echo ""
echo "**Do NOT mark as complete if verification fails.**"
