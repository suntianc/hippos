#!/usr/bin/env bash
set -euo pipefail

SURREAL_URL="${SURREAL_URL:-http://127.0.0.1:12470}"
DDL_FILE="${1:-surrealdb_ddl.surql}"

if [ ! -f "$DDL_FILE" ]; then
  echo "DDL file not found: $DDL_FILE" >&2
  exit 1
fi

AUTH_ARGS=()
if [ -n "${SURREAL_TOKEN:-}" ]; then
  AUTH_ARGS+=( -H "Authorization: Bearer ${SURREAL_TOKEN}" )
elif [ -n "${SURREAL_USERNAME:-}" ] && [ -n "${SURREAL_PASSWORD:-}" ]; then
  AUTH_ARGS+=( -u "${SURREAL_USERNAME}:${SURREAL_PASSWORD}" )
fi

echo "Applying DDL to ${SURREAL_URL} using ${DDL_FILE}"

if [ ${#AUTH_ARGS[@]} -gt 0 ]; then
  curl -sS -f \
    -X POST "${SURREAL_URL}/sql" \
    -H "Accept: application/json" \
    -H "Content-Type: text/plain" \
    "${AUTH_ARGS[@]}" \
    --data-binary "@${DDL_FILE}" > /tmp/surreal_ddl_result.json
else
  curl -sS -f \
    -X POST "${SURREAL_URL}/sql" \
    -H "Accept: application/json" \
    -H "Content-Type: text/plain" \
    --data-binary "@${DDL_FILE}" > /tmp/surreal_ddl_result.json
fi

echo "DDL applied. Response saved to /tmp/surreal_ddl_result.json"
