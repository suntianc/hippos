#!/bin/bash
# pre-tool-use.sh - Security protection layer
# Trigger: Before AI executes Bash commands or writes files
# Function: Intercept dangerous commands, warn about sensitive operations

# Get command from arguments
args=("$@")
command=""
target_file=""

# Parse arguments (handle -- separator)
for i in "${!args[@]}"; do
    if [[ "${args[$i]}" == "--" ]]; then
        # Collect command before --
        command="${args[@]:0:$i}"
        # Collect file paths after --
        for j in "${args[@]:$((i+1))}"; do
            if [[ "$j" =~ ^/ ]] || [[ "$j" =~ ^[a-z]: ]]; then
                target_file="$j"
                break
            fi
        done
        break
    fi
done

# If no -- found, use all args as command
if [[ -z "$command" ]]; then
    command="${args[*]}"
fi

# Exit if no command
if [[ -z "$command" ]]; then
    exit 0
fi

# Check for dangerous patterns
check_dangerous_command() {
    local cmd="$1"
    local result

    # Block: rm -rf with wildcards
    if echo "$cmd" | grep -Eq 'rm\s+(-rf?|--recursive).*[/.*]'; then
        echo "block|Dangerous command detected: rm -rf may delete important files|$cmd"
        return
    fi

    # Warn: rm -rf without wildcards
    if echo "$cmd" | grep -Eq 'rm\s+(-rf?|--recursive)'; then
        echo "warn|Warning: rm -rf command is very dangerous, please confirm|$cmd"
        return
    fi

    # Block: drop database objects
    if echo "$cmd" | grep -Eq 'drop\s+(database|table|index)'; then
        echo "block|Dangerous operation detected: deleting database object|$cmd"
        return
    fi

    # Warn: truncate
    if echo "$cmd" | grep -Eq 'truncate\s+'; then
        echo "warn|Warning: truncate operation is irreversible|$cmd"
        return
    fi

    # Warn: delete from
    if echo "$cmd" | grep -Eq 'delete\s+.*from'; then
        echo "warn|Warning: DELETE operation may delete data, please confirm|$cmd"
        return
    fi

    # Block: disk formatting (Windows)
    if echo "$cmd" | grep -Eq 'format\s+[a-z]:'; then
        echo "block|Dangerous command detected: disk formatting|$cmd"
        return
    }

    # Block: recursive force delete (PowerShell/cmd equivalent)
    if echo "$cmd" | grep -Eq 'rm\s+-rf.*[/\\]' || echo "$cmd" | grep -Eq 'Remove-Item.*-Recurse.*-Force.*[/\\]'; then
        echo "block|Dangerous command detected: recursive force delete|$cmd"
        return
    }

    # Warn: batch file deletion (Windows)
    if echo "$cmd" | grep -Eq 'del\s+/[sS].*\*'; then
        echo "warn|Dangerous command detected: batch file deletion|$cmd"
        return
    }

    # Warn: chmod 777
    if echo "$cmd" | grep -Eq 'chmod\s+777'; then
        echo "warn|Warning: chmod 777 may cause security risks|$cmd"
        return
    }

    # Warn: force reinstall
    if echo "$cmd" | grep -Eq 'npm\s+run\s+(reinstall|rebuild)\s+--force'; then
        echo "warn|Warning: force reinstall may affect project stability|$cmd"
        return
    fi

    echo "allow|Command is safe|$cmd"
}

# Check for sensitive files
check_sensitive_file() {
    local file="$1"
    local result

    if [[ -z "$file" ]]; then
        echo "allow|No target file|$file"
        return
    fi

    # .env files
    if echo "$file" | grep -Eq '\.env(\.local)?$'; then
        echo "warn|Target file contains sensitive config: .env|$file"
        return
    fi

    # package.json
    if echo "$file" | grep -Eq 'package\.json$'; then
        echo "warn|Target file is package.json, please confirm modification|$file"
        return
    fi

    # electron main.ts
    if echo "$file" | grep -Eq 'electron[\\/]main\.ts$'; then
        echo "warn|Target file is main process entry, modification may cause app issues|$file"
        return
    fi

    echo "allow|File is safe|$file"
}

# Execute checks
danger_result=$(check_dangerous_command "$command")
decision=$(echo "$danger_result" | cut -d'|' -f1)
reason=$(echo "$danger_result" | cut -d'|' -f2)
severity=$(echo "$danger_result" | cut -d'|' -f3 | head -c 100)

if [[ "$decision" == "block" ]]; then
    echo "{\"decision\": \"block\", \"reason\": \"$reason\", \"command\": \"$severity\", \"severity\": \"$severity\"}"
    exit 1
elif [[ "$decision" == "warn" ]]; then
    echo "{\"decision\": \"warn\", \"reason\": \"$reason\", \"command\": \"$severity\", \"severity\": \"$severity\"}"
    exit 0
fi

# Check sensitive file
file_result=$(check_sensitive_file "$target_file")
file_decision=$(echo "$file_result" | cut -d'|' -f1)
file_reason=$(echo "$file_result" | cut -d'|' -f2)
file_severity=$(echo "$file_result" | cut -d'|' -f3 | head -c 100)

if [[ "$file_decision" == "warn" ]]; then
    echo "{\"decision\": \"warn\", \"reason\": \"$file_reason\", \"targetFile\": \"$file_severity\", \"severity\": \"warning\"}"
elif [[ "$file_decision" == "allow" ]]; then
    echo "{\"decision\": \"allow\", \"reason\": \"Command is safe\", \"command\": \"$severity\"}"
fi

exit 0
