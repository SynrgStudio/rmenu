# ERROR ISOLATION POLICY

Status: Frozen v1  
Date: 2026-04-24

---

## 1. Objective

Prevent module errors from compromising launcher stability.

Rule:

> A module failure must degrade optional functionality, not break the core.

---

## 2. Scope

This policy applies to:

- external module host startup,
- IPC requests/responses,
- hook execution,
- provider execution,
- command dispatch,
- decorations,
- key hooks,
- actions requested by modules,
- payload validation.

---

## 3. Module states

States v1:

- `loaded` — module is operational.
- `degraded` — module has recent errors but can still recover.
- `disabled` — module is disabled by policy or configuration.
- `unloaded` — descriptor exists but no active host is running.

Transitions must be observable through debug/log output.

---

## 4. General error handling

On module error:

1. Catch error at the module boundary.
2. Record:
   - module,
   - operation/hook,
   - latency,
   - error kind,
   - message.
3. Do not panic the launcher.
4. Continue global operation with remaining modules/results.
5. Update telemetry.
6. Apply restart/degrade/disable policy when thresholds are reached.

---

## 5. Hook errors

If a hook throws or returns invalid data:

- the hook result is discarded,
- the error is recorded,
- other modules continue,
- the launcher keeps the previous safe state when possible.

Repeated errors may degrade or disable the module.

---

## 6. Timeout policy

If a module host times out:

- the response is discarded,
- timeout telemetry is incremented,
- the host may be restarted,
- after the threshold is exceeded, the module becomes `disabled`.

Timeouts must not block the UI indefinitely.

---

## 7. Invalid payloads

Invalid payloads are module/host errors, not core errors.

Examples:

- missing required item fields,
- oversized IPC payload,
- invalid quick-select key,
- malformed action,
- invalid JSON/config.

The core may discard, normalize, or reject the payload. It must never crash because of invalid payload.

---

## 8. Host restart and backoff

When a host fails:

1. Stop the broken host if still running.
2. Wait for configured backoff.
3. Restart if the module is not disabled.
4. Reset consecutive error counters after successful reload when appropriate.
5. Recurrent restarts end in degradation or disable.

---

## 9. Permission violations

If a module uses an operation without the required capability:

- the operation is rejected,
- `permission_denied` is recorded,
- the module remains isolated,
- the launcher continues.

Repeated violations may be treated as module errors.

---

## 10. Global operations

A global runtime operation, such as module reload, must fail partially when possible:

- the launcher continues,
- the affected module becomes `degraded` or `disabled`,
- unrelated modules remain loaded,
- other modules are not unloaded unless they explicitly depend on that global operation.

---

## 11. Observability

`--modules-debug` must expose at least:

- builtin module count,
- external descriptors,
- running hosts,
- loaded modules,
- host state,
- request count,
- error count,
- timeout count,
- restart count,
- average/max latency,
- recent errors.

`--silent` suppresses non-critical diagnostics but must not suppress internal telemetry.

---

## 12. Recovery

A module can recover through:

- `/modules.reload`,
- file hot reload,
- process restart,
- manifest/script fix,
- disabling and re-enabling the module.

---

## 13. Minimum test cases

Tests should cover:

- module throws in hook,
- provider timeout,
- provider returns invalid payload,
- invalid command,
- missing capability,
- IPC payload exceeds limit,
- host crashes or exits,
- repeated failures trigger disable.

Expected result:

- launcher remains operational,
- module enters expected state,
- other modules keep working.
