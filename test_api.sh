#!/bin/bash

# Hippos API 测试脚本
# 使用方法: ./test_api.sh [server_url] [api_key]

set -e

# 默认配置
SERVER_URL="${1:-http://localhost:8080}"
API_KEY="${2:-dev-api-key}"
TENANT_A="tenant_a"
TENANT_B="tenant_b"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 测试结果文件
RESULTS_FILE="test_results.json"
mkdir -p test_data/requests test_data/responses test_data/tenant_isolation

# 初始化结果
echo '{"test_run": {' > "$RESULTS_FILE"
echo "  \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"," >> "$RESULTS_FILE"
echo "  \"server_url\": \"$SERVER_URL\"," >> "$RESULTS_FILE"
echo "  \"api_key\": \"${API_KEY:0:8}...\"," >> "$RESULTS_FILE"
echo "  \"tests\": [" >> "$RESULTS_FILE"

passed=0
failed=0
total=0

# 测试函数
test_endpoint() {
    local name="$1"
    local method="$2"
    local endpoint="$3"
    local data="$4"
    local expected_status="$5"
    
    total=$((total + 1))
    echo -e "${BLUE}Testing: $name${NC}"
    
    # 构建 curl 命令
    local curl_cmd="curl -s -w '\n%{http_code}' -X $method"
    curl_cmd="$curl_cmd -H 'Authorization: ApiKey $API_KEY'"
    curl_cmd="$curl_cmd -H 'Content-Type: application/json'"
    
    if [ -n "$data" ]; then
        curl_cmd="$curl_cmd -d '$data'"
    fi
    
    curl_cmd="$curl_cmd '$SERVER_URL$endpoint'"
    
    # 执行请求
    local response=$(eval $curl_cmd 2>/dev/null)
    local http_code=$(echo "$response" | tail -1)
    local body=$(echo "$response" | sed '$d')
    
    # 保存请求和响应
    local test_name=$(echo "$name" | tr ' ' '_' | tr '[:upper:]' '[:lower:]')
    echo "{\"test\": \"$name\", \"request\": {\"method\": \"$method\", \"endpoint\": \"$endpoint\", \"body\": $data}, \"response\": $body, \"status\": $http_code}" > "test_data/requests/${test_name}.json"
    
    # 检查结果
    if [ "$http_code" == "$expected_status" ]; then
        echo -e "  ${GREEN}✓ PASSED${NC} (Status: $http_code)"
        passed=$((passed + 1))
        echo "    {\"name\": \"$name\", \"status\": \"passed\", \"http_code\": $http_code}," >> "$RESULTS_FILE"
    else
        echo -e "  ${RED}✗ FAILED${NC} (Expected: $expected_status, Got: $http_code)"
        failed=$((failed + 1))
        echo "    {\"name\": \"$name\", \"status\": \"failed\", \"expected\": $expected_status, \"got\": $http_code}," >> "$RESULTS_FILE"
    fi
}

# 测试健康检查
echo -e "${YELLOW}=== 健康检查测试 ===${NC}"
test_endpoint "Health Check" "GET" "/health" "" "200"
test_endpoint "Liveness Check" "GET" "/health/live" "" "200"
test_endpoint "Version Info" "GET" "/version" "" "200"

# 测试认证
echo -e "${YELLOW}=== 认证测试 ===${NC}"
test_endpoint "Valid API Key" "GET" "/api/v1/sessions" "" "200"
test_endpoint "Invalid API Key" "GET" "/api/v1/sessions" "" "401"
test_endpoint "No Auth Header" "GET" "/api/v1/sessions" "" "401"

# 测试 Session API
echo -e "${YELLOW}=== Session API 测试 ===${NC}"

# 创建会话
echo -e "${BLUE}Creating test session...${NC}"
SESSION_RESPONSE=$(curl -s -X POST "$SERVER_URL/api/v1/sessions" \
    -H "Authorization: ApiKey $API_KEY" \
    -H "Content-Type: application/json" \
    -d '{"name": "Test Session", "description": "API Test Session"}')
echo "$SESSION_RESPONSE" > "test_data/responses/create_session.json"
SESSION_ID=$(echo "$SESSION_RESPONSE" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)

if [ -z "$SESSION_ID" ]; then
    echo -e "${RED}Failed to create session, skipping session tests${NC}"
else
    echo -e "${GREEN}Created session: $SESSION_ID${NC}"
    
    test_endpoint "Get Session" "GET" "/api/v1/sessions/$SESSION_ID" "" "200"
    test_endpoint "List Sessions" "GET" "/api/v1/sessions" "" "200"
    test_endpoint "Update Session" "PUT" "/api/v1/sessions/$SESSION_ID" '{"name": "Updated Session"}' "200"
    
    # 测试 Turn API
    echo -e "${YELLOW}=== Turn API 测试 ===${NC}"
    
    TURN_RESPONSE=$(curl -s -X POST "$SERVER_URL/api/v1/sessions/$SESSION_ID/turns" \
        -H "Authorization: ApiKey $API_KEY" \
        -H "Content-Type: application/json" \
        -d '{"role": "user", "content": "Hello, this is a test message"}')
    echo "$TURN_RESPONSE" > "test_data/responses/create_turn.json"
    TURN_ID=$(echo "$TURN_RESPONSE" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
    
    if [ -z "$TURN_ID" ]; then
        echo -e "${RED}Failed to create turn${NC}"
    else
        echo -e "${GREEN}Created turn: $TURN_ID${NC}"
        test_endpoint "Get Turn" "GET" "/api/v1/sessions/$SESSION_ID/turns/$TURN_ID" "" "200"
        test_endpoint "List Turns" "GET" "/api/v1/sessions/$SESSION_ID/turns" "" "200"
        test_endpoint "Update Turn" "PUT" "/api/v1/sessions/$SESSION_ID/turns/$TURN_ID" '{"content": "Updated content"}' "200"
        test_endpoint "Delete Turn" "DELETE" "/api/v1/sessions/$SESSION_ID/turns/$TURN_ID" "" "200"
    fi
    
    # 测试边界条件
    echo -e "${YELLOW}=== 边界条件测试 ===${NC}"
    test_endpoint "Empty Content" "POST" "/api/v1/sessions/$SESSION_ID/turns" '{"role": "user", "content": ""}' "400"
    test_endpoint "Non-existent Session" "GET" "/api/v1/sessions/non_existent_session" "" "404"
    test_endpoint "Empty Query" "GET" "/api/v1/sessions/$SESSION_ID/search?q=" "" "400"
    
    # 测试状态机
    echo -e "${YELLOW}=== 状态机测试 ===${NC}"
    test_endpoint "Archive Session" "POST" "/api/v1/sessions/$SESSION_ID/archive" '{"reason": "Test archive"}' "200"
    
    # 恢复并删除
    test_endpoint "Restore Session" "POST" "/api/v1/sessions/$SESSION_ID/restore" '{"new_name": null}' "200"
    test_endpoint "Delete Session" "DELETE" "/api/v1/sessions/$SESSION_ID" "" "200"
fi

# 租户隔离测试
echo -e "${YELLOW}=== 租户隔离测试 ===${NC}"
echo '{"tenant_tests": [' > "test_data/tenant_isolation/results.json"

# 创建租户 A 的会话
TENANT_A_SESSION=$(curl -s -X POST "$SERVER_URL/api/v1/sessions" \
    -H "Authorization: ApiKey $API_KEY" \
    -H "X-Tenant-Id: $TENANT_A" \
    -H "Content-Type: application/json" \
    -d '{"name": "Tenant A Session"}')
TENANT_A_SESSION_ID=$(echo "$TENANT_A_SESSION" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)

if [ -n "$TENANT_A_SESSION_ID" ]; then
    echo "  {\"tenant\": \"$TENANT_A\", \"session_id\": \"$TENANT_A_SESSION_ID\", \"action\": \"create\", \"status\": \"success\"}," >> "test_data/tenant_isolation/results.json"
    
    # 尝试使用租户 B 的凭据访问
    CROSS_TENANT_RESPONSE=$(curl -s -w '\n%{http_code}' -X GET "$SERVER_URL/api/v1/sessions/$TENANT_A_SESSION_ID" \
        -H "Authorization: ApiKey $API_KEY" \
        -H "X-Tenant-Id: $TENANT_B")
    
    CROSS_TENANT_CODE=$(echo "$CROSS_TENANT_RESPONSE" | tail -1)
    
    if [ "$CROSS_TENANT_CODE" == "403" ] || [ "$CROSS_TENANT_CODE" == "404" ]; then
        echo -e "  ${GREEN}✓ Tenant Isolation Working${NC}"
        echo "    {\"tenant\": \"$TENANT_B\", \"action\": \"access_tenant_a_session\", \"status\": \"blocked\", \"http_code\": $CROSS_TENANT_CODE}," >> "test_data/tenant_isolation/results.json"
    else
        echo -e "  ${RED}✗ Tenant Isolation NOT Working${NC}"
        echo "    {\"tenant\": \"$TENANT_B\", \"action\": \"access_tenant_a_session\", \"status\": \"bypassed\", \"http_code\": $CROSS_TENANT_CODE}," >> "test_data/tenant_isolation/results.json"
    fi
    
    # 清理
    curl -s -X DELETE "$SERVER_URL/api/v1/sessions/$TENANT_A_SESSION_ID" \
        -H "Authorization: ApiKey $API_KEY" \
        -H "X-Tenant-Id: $TENANT_A" > /dev/null
fi

# 完成租户测试
echo "    {\"tenant\": \"cleanup\", \"action\": \"delete_tenant_a_session\", \"status\": \"completed\"}" >> "test_data/tenant_isolation/results.json"
echo "  ]}" >> "test_data/tenant_isolation/results.json"

# 写入最终结果
echo "  ]," >> "$RESULTS_FILE"
echo "  \"summary\": {" >> "$RESULTS_FILE"
echo "    \"total\": $total," >> "$RESULTS_FILE"
echo "    \"passed\": $passed," >> "$RESULTS_FILE"
echo "    \"failed\": $failed" >> "$RESULTS_FILE"
echo "  }" >> "$RESULTS_FILE"
echo "}}" >> "$RESULTS_FILE"

# 打印总结
echo ""
echo -e "${YELLOW}=== 测试总结 ===${NC}"
echo -e "总计: ${total} 个测试"
echo -e "${GREEN}通过: ${passed}${NC}"
echo -e "${RED}失败: ${failed}${NC}"

if [ $failed -eq 0 ]; then
    echo -e "${GREEN}所有测试通过!${NC}"
    exit 0
else
    echo -e "${RED}存在失败的测试${NC}"
    exit 1
fi
