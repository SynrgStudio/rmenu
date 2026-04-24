# PROVIDER EXECUTION POLICY

Status: Frozen v1  
Date: 2026-04-24

---

## 1. Objective

Define how providers are executed while preserving low latency, stability, and deterministic results.

---

## 2. Principles

1. Providers must not block the UI.
2. A new query may invalidate older work.
3. The core controls budgets, timeouts, and final merge.
4. Results must be reproducible for the same input.
5. Provider errors do not break the global query.

---

## 3. Per-query lifecycle

1. Core receives query.
2. Core evaluates enabled providers.
3. Core applies the `providers` capability.
4. Core collects responses within budget.
5. Core discards invalid, late, or stale responses when applicable.
6. Core sanitizes items.
7. Core applies per-provider-host item caps.
8. Core merges provider items with base items.
9. Core applies dedupe.
10. Core applies final ranking.
11. Core allows decorations when applicable.
12. Core renders.

---

## 4. Configuration v1

Parameters in `[Modules]`:

```ini
provider_total_budget_ms = 35
provider_timeout_ms = 1500
max_items_per_provider_host = 24
dedupe_source_priority = core_first
host_restart_backoff_ms = 800
max_ipc_payload_bytes = 262144
```

### `provider_total_budget_ms`

Approximate global budget for collecting providers for one query.

If exceeded, the core stops querying remaining providers and continues with available results.

### `provider_timeout_ms`

Timeout per request to a provider/host.

If exceeded:

- the response is discarded,
- the timeout is recorded,
- the host may be restarted,
- it may contribute to degradation/disable policy.

### `max_items_per_provider_host`

Maximum accepted item count per provider host.

Extra items are truncated before entering the pipeline.

### `dedupe_source_priority`

Values:

- `core_first`
- `provider_first`

Defines priority when resolving duplicates between core and provider items.

### `max_ipc_payload_bytes`

Maximum request/response IPC payload size.

Larger payloads are rejected to protect stability.

---

## 5. Merge and dedupe

Dedupe must be deterministic.

Suggested key:

1. stable `id` when applicable,
2. normalized `target` when present,
3. core-controlled fallback.

Policies:

- `core_first`: when duplicated, core item wins.
- `provider_first`: when duplicated, provider item wins.

After dedupe, final ranking remains under core authority.

---

## 6. Execution order

Providers must be evaluated in stable order:

1. priority,
2. module/provider name,
3. normalized discovery order when applicable.

Goal: the same input should produce the same observable order.

---

## 7. Item sanitization

The core must validate:

- required `id`,
- required `title`,
- maximum lengths,
- multiline strings where disallowed,
- valid `quickSelectKey`,
- unknown fields as ignorable,
- total payload within IPC limit.

Invalid items are discarded or normalized. They must not crash the core.

---

## 8. Errors

On provider error:

- the global query continues,
- module/provider/latency/error are recorded,
- invalid partial results are not rendered,
- the host may be restarted,
- consecutive errors may degrade or disable the module.

See `ERROR_ISOLATION_POLICY.md`.

---

## 9. Minimum observability

`--modules-debug` must expose at least:

- loaded modules,
- running hosts,
- host state,
- request count,
- error count,
- timeout count,
- restart count,
- average/max latency,
- recent errors.

---

## 10. Author recommendations

- Respond quickly.
- Return few relevant items.
- Filter early by query.
- Cache when appropriate.
- Avoid blocking I/O.
- Do not depend on receiving every query.
- Do not depend on undocumented ordering.
- Handle own errors and log useful context.
