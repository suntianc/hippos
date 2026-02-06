# Runbook

## API not responding

1. Check process logs.
2. Call `/healthz`.
3. Restart API process.

## Data inconsistency

1. Query audit by `trace_id`.
2. Revoke wrong fact by governance API.
