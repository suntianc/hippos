#!/bin/bash
# rules-loader.sh - Intelligent rule loader
# Trigger: SessionStart (on session start)
# Function: Load global behavior rules, ensure AI follows project conventions

RULES_DIR=".claude/rules"
CLAUDE_MD="CLAUDE.md"
OPENCODE_AGENTS=".opencode/AGENTS.md"

# Global rules loaded at session start
GLOBAL_RULES=("00-global.md")

echo "## [Session Init] Project Rules Loading"
echo ""

# Check CLAUDE.md
if [[ -f "$CLAUDE_MD" ]]; then
    echo "### Core Configuration"
    echo ""
    echo "- **Project Config**: \`$CLAUDE_MD\` - Must read and follow"
    echo ""
fi

# Check AGENTS.md
if [[ -f "$OPENCODE_AGENTS" ]]; then
    echo "- **Agent Guide**: \`$OPENCODE_AGENTS\` - Agent behavior conventions"
    echo ""
fi

# Load global rules
if [[ -d "$RULES_DIR" ]]; then
    echo "### Global Behavior Rules (SessionStart Loaded)"
    echo ""

    for rule in "${GLOBAL_RULES[@]}"; do
        rule_path="$RULES_DIR/$rule"
        if [[ -f "$rule_path" ]]; then
            echo "- \`$rule_path\` - Global behavior conventions"
        fi
    done

    echo ""
    echo "### Other Rules (Loaded on Demand)"
    echo ""
    echo "The following rules will be automatically loaded at appropriate times:"
    echo ""
    echo "| Rule File | Load Timing | Purpose |"
    echo "|-----------|-------------|---------|"
    echo "| \`01-code-quality.md\` | PreToolUse (Write/Edit) | Code Verification |"
    echo "| \`02-code-style.md\` | PreToolUse (Write/Edit) | Code Style |"
    echo "| \`03-security.md\` | PreToolUse (Write/Edit) | Security Check |"
    echo "| \`04-performance.md\` | PreToolUse (Write/Edit) | Performance |"
    echo "| \`05-documentation.md\` | Stop | Documentation Sync |"
    echo "| \`06-context7-query.md\` | UserPromptSubmit | External Query |"
    echo "| \`07-refactoring.md\` | PreToolUse (Read) | Refactoring Detection |"
    echo ""
fi

echo "### Required Actions"
echo ""
echo "1. **Read** \`CLAUDE.md\` for project configuration"
echo "2. **Read** \`.claude/rules/00-global.md\` for global behavior conventions"
echo "3. **Follow** all rules marked as MUST (mandatory requirements)"
echo ""
echo "**Not reading rule files = non-compliant implementation**"
