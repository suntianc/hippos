#!/bin/bash
# skill-forced-eval.sh - Mandatory skill evaluation hook
# Trigger: UserPromptSubmit (on every user submission)
# Function: Evaluate and activate relevant skills, check if external docs need querying

# Skill definitions
declare -A SKILL_keywords
SKILL_keywords["electron-main"]="electron,main process,ipc,database,sqlite,lancedb,mcp,native module"
SKILL_keywords["react-frontend"]="react,frontend,component,hooks,typescript,jsx,tsx,ui"
SKILL_keywords["rag-vectordb"]="rag,vector,retrieval,knowledge,embedding,lancedb,chunk"
SKILL_keywords["ai-integration"]="ai,llm,gemini,ollama,openai,api,chat,generate"
SKILL_keywords["mcp-tools"]="mcp,tool,protocol,server,browser,filesystem"
SKILL_keywords["platform-build"]="package,build,electron-builder,installer,dmg,exe,deb"
SKILL_keywords["bug-debug"]="bug,error,exception,debug,troubleshoot,problem"
SKILL_keywords["planning-with-files"]="plan,planning,task,project,scope,requirements,manus,structure"
SKILL_keywords["ui-ux-pro-max"]="ui,ux,design,style,color,typography,font,landing,dashboard,glassmorphism,minimalism,dark mode,tailwind,css,responsive,animation,hover,layout"
SKILL_keywords["spec-interview"]="spec,specification,interview,requirements,clarify,scope,define"

# Skill descriptions
declare -A SKILL_descriptions
SKILL_descriptions["electron-main"]="Electron Main Process Development"
SKILL_descriptions["react-frontend"]="React Frontend Development"
SKILL_descriptions["rag-vectordb"]="RAG Vector Database"
SKILL_descriptions["ai-integration"]="AI Service Integration"
SKILL_descriptions["mcp-tools"]="MCP Tool Protocol"
SKILL_descriptions["platform-build"]="Platform Build & Package"
SKILL_descriptions["bug-debug"]="Bug Debugging & Troubleshooting"
SKILL_descriptions["planning-with-files"]="Manus-style File Planning & Task Management"
SKILL_descriptions["ui-ux-pro-max"]="UI/UX Design Intelligence"
SKILL_descriptions["spec-interview"]="Requirements Specification Interview"

# External library keywords - trigger Context7 query
LIBRARY_KEYWORDS=(
    "react-query" "tanstack" "zustand" "jotai" "recoil" "redux"
    "next.js" "nextjs" "nuxt" "svelte" "vue" "angular"
    "tailwindcss" "shadcn" "radix" "chakra" "antd" "material-ui" "mui"
    "zod" "yup" "formik" "react-hook-form"
    "axios" "swr" "trpc" "graphql"
    "vite" "webpack" "esbuild" "rollup" "turbopack"
    "express" "fastify" "nest" "koa"
    "prisma" "drizzle" "typeorm" "sequelize"
    "vitest" "jest" "playwright" "cypress"
)

# Get user prompt from arguments or stdin
user_prompt="$1"
if [[ -z "$user_prompt" ]] && [[ ! -t 0 ]]; then
    user_prompt=$(cat)
fi

# Skip slash commands
if [[ "$user_prompt" =~ ^/[a-zA-Z]+ ]]; then
    echo "[Hook] Detected slash command, skipping skill evaluation: ${user_prompt%% *}"
    exit 0
fi

if [[ -n "$user_prompt" ]]; then
    user_prompt_lower=$(echo "$user_prompt" | tr '[:upper:]' '[:lower:]')

    echo "## Command: Mandatory Skill Activation (Must Execute)"
    echo ""

    # ============================================
    # Context7 Query Check (from 06-context7-query.md)
    # ============================================
    mentioned_libraries=()
    for lib in "${LIBRARY_KEYWORDS[@]}"; do
        if [[ "$user_prompt_lower" == *"$lib"* ]]; then
            mentioned_libraries+=("$lib")
        fi
    done

    if [[ ${#mentioned_libraries[@]} -gt 0 ]]; then
        echo "### External Library Detection (Context7 Query)"
        echo ""
        echo "Detected the following external libraries/frameworks:"
        echo ""
        for lib in "${mentioned_libraries[@]}"; do
            echo "- **$lib**"
        done
        echo ""
        echo "According to \`.claude/rules/06-context7-query.md\` rules:"
        echo ""
        echo "1. If you are **unfamiliar** with these libraries, you **must** query official docs first"
        echo "2. Query priority: Context7 → deepwiki.com → GitHub"
        echo "3. Inform user before querying, cite sources after querying"
        echo ""
    fi

    # ============================================
    # Skill Evaluation
    # ============================================
    echo "### Step 1 - Evaluate Skills"
    echo "For each skill, explain: [skill_name] - Yes/No - [reason]"
    echo ""
    echo "Available Skills:"
    for skill in "${!SKILL_descriptions[@]}"; do
        echo "  - $skill: ${SKILL_descriptions[$skill]}"
    done
    echo ""
    echo "User input: $user_prompt"
    echo ""
    echo "### Step 2 - Activate Skills"

    relevant_skills=()
    for skill in "${!SKILL_keywords[@]}"; do
        keywords="${SKILL_keywords[$skill]}"
        IFS=',' read -ra keyword_list <<< "$keywords"
        match_count=0
        matched_keywords=()
        for keyword in "${keyword_list[@]}"; do
            keyword=$(echo "$keyword" | xargs)
            if [[ "$user_prompt_lower" == *"$keyword"* ]]; then
                ((match_count++))
                matched_keywords+=("$keyword")
            fi
        done
        if [[ $match_count -gt 0 ]]; then
            relevant_skills+=("$skill|${matched_keywords[*]}")
        fi
    done

    if [[ ${#relevant_skills[@]} -gt 0 ]]; then
        echo "Detected relevant skills:"
        for item in "${relevant_skills[@]}"; do
            skill_name="${item%%|*}"
            matched="${item##*|}"
            echo "- $skill_name: Yes - Matched keywords: $matched"
        done
        echo ""
        echo "Activation commands:"
        for item in "${relevant_skills[@]}"; do
            skill_name="${item%%|*}"
            echo "> Skill($skill_name)"
        done
    else
        echo "All skills evaluated as 'No', explain 'no skills needed' and continue"
    fi

    echo ""
    echo "### Step 3 - Implementation"
    echo "Only start implementing user request after completing Steps 1 and 2."
    echo ""
    echo "### Important Notes"
    echo "- Must complete Steps 1 and 2 before Step 3"
    echo "- Use \`Skill()\` tool to activate relevant skills"
    echo "- If no relevant skills, explain reason and answer directly"
fi
