#!/bin/bash
# stop.sh - Task completion feedback
# Trigger: After AI finishes answering
# Function: Analyze changes, recommend next steps

# Get git change statistics
get_change_stats() {
    local status=$(git status --porcelain 2>/dev/null)

    if [[ -z "$status" ]]; then
        echo "0|0|0|"
        return
    fi

    local added=0
    local modified=0
    local deleted=0
    local files=()

    while IFS= read -r line; do
        [[ -z "$line" ]] && continue
        local status_code="${line:0:2}"
        local file_path="${line:3}"

        case "$status_code" in
            "A "|"??")
                ((added++))
                ;;
            "M ")
                ((modified++))
                ;;
            "D ")
                ((deleted++))
                ;;
        esac

        if [[ ${#files[@]} -lt 10 ]]; then
            files+=("$status_code|$file_path")
        fi
    done <<< "$status"

    echo "$added|$modified|$deleted|${files[*]}"
}

# Cleanup temporary files
cleanup_temp_files() {
    local temp_patterns=("*.tmp" "*.log" "nul" "*~" "*.bak" "*.swp")
    local temp_locations=("." "temp" "tmp")

    for loc in "${temp_locations[@]}"; do
        if [[ -d "$loc" ]]; then
            for pattern in "${temp_patterns[@]}"; do
                find "$loc" -maxdepth 1 -name "$pattern" -delete 2>/dev/null
            done
        fi
    done
}

# Generate change summary
generate_change_summary() {
    local added="$1"
    local modified="$2"
    local deleted="$3"

    local parts=()
    [[ $added -gt 0 ]] && parts+=("$added added")
    [[ $modified -gt 0 ]] && parts+=("$modified modified")
    [[ $deleted -gt 0 ]] && parts+=("$deleted deleted")

    if [[ ${#parts[@]} -eq 0 ]]; then
        echo "No code changes detected"
    else
        echo "${parts[*]}"
    fi
}

# Generate suggestions based on changes
generate_suggestions() {
    local files_string="$1"
    local suggestions=()

    # Check for code changes
    if [[ -n "$files_string" ]]; then
        suggestions+=("Use \`@code-reviewer\` to review code")
        suggestions+=("Run \`/update-status\` to update project status")
    fi

    # Check for database changes
    if [[ "$files_string" == *".sql"* ]] || [[ "$files_string" == *"schema"* ]]; then
        suggestions+=("Database scripts changed, ensure sync across environments")
    fi

    # Check for service changes
    if [[ "$files_string" == *"Service"* ]] || [[ "$files_string" == *"service"* ]]; then
        suggestions+=("Service layer changed, remember to update docs")
    fi

    suggestions+=("Use \`git add . && git commit\` to commit code")

    printf '%s\n' "${suggestions[@]}"
}

# Main execution
stats=$(get_change_stats)
cleanup_temp_files

IFS='|' read -r added modified deleted files_string <<< "$stats"

change_summary=$(generate_change_summary "$added" "$modified" "$deleted")
suggestions=$(generate_suggestions "$files_string")

echo "---"
echo ""
echo "Task Completed | $change_summary"
echo ""

if [[ -n "$files_string" ]]; then
    echo "**Changed Files**:"
    IFS=' ' read -ra files_array <<< "$files_string"
    for file_info in "${files_array[@]}"; do
        status_code="${file_info%%|*}"
        file_path="${file_info##*|}"
        icon=""
        case "$status_code" in
            "M ") icon="[M]" ;;
            "A "|"??" ) icon="[+]" ;;
            "D ") icon="[-]" ;;
            *) icon="[*]" ;;
        esac
        echo "  $icon $file_path"
    done
    echo ""
fi

echo "**Suggested Actions**:"
echo "$suggestions" | while read -r suggestion; do
    echo "- $suggestion"
done
echo ""
echo "**Quick Commands**:"
echo "- \`/update-status\` - Update project status"
echo "- \`/progress\` - View development progress"
echo "- \`/next\` - Get next step suggestions"
