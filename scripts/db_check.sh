#!/usr/bin/env bash
set -euo pipefail

SURREAL_URL="${SURREAL_URL:-http://127.0.0.1:12470}"
SURREAL_NAMESPACE="${SURREAL_NAMESPACE:-mem_ns}"
SURREAL_DATABASE="${SURREAL_DATABASE:-mem_db}"

AUTH_ARGS=()
if [ -n "${SURREAL_TOKEN:-}" ]; then
  AUTH_ARGS+=( -H "Authorization: Bearer ${SURREAL_TOKEN}" )
elif [ -n "${SURREAL_USERNAME:-}" ] && [ -n "${SURREAL_PASSWORD:-}" ]; then
  AUTH_ARGS+=( -u "${SURREAL_USERNAME}:${SURREAL_PASSWORD}" )
fi

echo "Checking SurrealDB health: ${SURREAL_URL}/health"
curl -sS -f "${SURREAL_URL}/health" >/dev/null
echo "Health: OK"

echo "Checking schema in NS=${SURREAL_NAMESPACE}, DB=${SURREAL_DATABASE}"
QUERY='INFO FOR DB;'

if [ ${#AUTH_ARGS[@]} -gt 0 ]; then
  curl -sS -f \
    -X POST "${SURREAL_URL}/sql" \
    -H "Accept: application/json" \
    -H "Content-Type: text/plain" \
    -H "NS: ${SURREAL_NAMESPACE}" \
    -H "DB: ${SURREAL_DATABASE}" \
    "${AUTH_ARGS[@]}" \
    --data "${QUERY}" > /tmp/surreal_db_info.json
else
  curl -sS -f \
    -X POST "${SURREAL_URL}/sql" \
    -H "Accept: application/json" \
    -H "Content-Type: text/plain" \
    -H "NS: ${SURREAL_NAMESPACE}" \
    -H "DB: ${SURREAL_DATABASE}" \
    --data "${QUERY}" > /tmp/surreal_db_info.json
fi

echo "DB info saved to /tmp/surreal_db_info.json"
