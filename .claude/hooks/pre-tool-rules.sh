#!/bin/bash
# pre-tool-rules.sh - PreToolUse rule injection
# Trigger: PreToolUse (before AI uses tools)
# Function: Inject relevant rule reminders based on tool type

# Get tool name from arguments or environment
tool_name="${1:-$CLAUDE_TOOL_NAME}"

# Rules mapping
declare -a WRITE_EDIT_RULES=("01-code-quality.md:Code Quality" "02-code-style.md:Code Style" "03-security.md:Security Check" "04-performance.md:Performance")
declare -a READ_RULES=("07-refactoring.md:Refactoring Detection")

RULES_DIR=".claude/rules"

# Detect tool type
is_write_edit=false
is_read=false

case "$tool_name" in
    Write|Edit|write|edit)
        is_write_edit=true
        ;;
    Read|read)
        is_read=true
        ;;
esac

# Output rule reminders for Write/Edit tools
if [[ "$is_write_edit" == true ]]; then
    echo ""
    echo "## [PreToolUse] Code Writing Rule Reminders"
    echo ""
    echo "Before executing **$toolName** operation, ensure following these rules:"
    echo ""

    for rule_info in "${WRITE_EDIT_RULES[@]}"; do
        rule_file="${rule_info%%:*}"
        rule_name="${rule_info##*:}"
        rule_path="$RULES_DIR/$rule_file"
        if [[ -f "$rule_path" ]]; then
            echo "### $rule_name"
            echo ""
            echo "Reference: \`$rule_path\`"
            echo ""
        fi
    done

    echo "### Quick Checklist"
    echo ""
    echo "- [ ] No \`as any\`, \`@ts-ignore\` type bypassing"
    echo "- [ ] File length: Components ≤300 lines, Services ≤500 lines"
    echo "- [ ] Nesting depth ≤4 levels"
    echo "- [ ] No hardcoded keys or sensitive information"
    echo "- [ ] No N+1 query patterns"
    echo "- [ ] Use virtualization for large lists"
    echo ""
    echo "**After completing edits, must run \`lsp_diagnostics\` for verification.**"
    echo ""
fi

# Output reminders for Read tools
if [[ "$is_read" == true ]]; then
    echo ""
    echo "## [PreToolUse] Refactoring Detection Reminder"
    echo ""
    echo "After reading files, check for these potential issues:"
    echo ""
    echo "1. **File Length** - Exceeding limits?"
    echo "2. **Duplicate Code** - Similar code blocks?"
    echo "3. **Deep Nesting** - More than 4 levels?"
    echo "4. **Outdated Dependencies** - Using deprecated APIs?"
    echo ""
    echo "If issues found, refer to \`.claude/rules/07-refactoring.md\` for alerts."
    echo ""
fi

# Exit silently if not specific tools
if [[ "$is_write_edit" == false ]] && [[ "$is_read" == false ]]; then
    exit 0
fi
