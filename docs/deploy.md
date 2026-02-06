# Deploy

## Local

1. Install Python 3.11+
2. Install deps: `pip install -e .[dev]`
3. (Optional) apply DB schema:
- `SURREAL_URL=http://127.0.0.1:12470 ./scripts/db_init.sh`
4. Run API (in-memory backend): `make dev`
5. Run API (Surreal backend):
- `MEMORY_BACKEND=surreal SURREAL_URL=http://127.0.0.1:12470 make dev`

## Health check

`GET /healthz` should return `{\"status\": \"ok\"}`.
