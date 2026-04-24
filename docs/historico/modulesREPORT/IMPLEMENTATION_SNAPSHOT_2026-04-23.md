# Modules Implementation Snapshot — 2026-04-23

Snapshot de cierre actualizado del sistema modular (`rmenu`) para reinicio de chat.

---

## 1) Estado general

Estado: **Batch 0..9 completados**.

Resumen:
- Runtime modular + host externo IPC operativo.
- Capabilities enforceadas (incluye `keys`).
- Hardening IPC y resiliencia host activos.
- UI primitives v1 cerradas (quick-select, trailing zones, input accessory).
- Tests críticos cubiertos.
- Documentación de operación/authoring/release completa.

---

## 2) Qué quedó implementado (cierre)

### Core/runtime/host
- Runtime modular conectado a pipeline real de UI/ranking.
- Host externo por módulo con bridge Node persistente.
- IPC lifecycle: initialize/load/query/key/provide/decorate/command/unload/shutdown.
- Timeout por request, restart automático, auto-disable por umbral, backoff de restart.
- Estados de host: `loaded | degraded | disabled | unloaded`.
- Telemetría por host + `recent_errors`.

### Capabilities enforce (v1)
- `providers` en `provideItems` + `registerProvider`.
- `commands` en `onCommand` + `registerCommand`.
- `decorate-items` en `decorateItems`.
- `input-accessory` en `set/clearInputAccessory`.
- `keys` en `onKey`.
- Registro de `permission_denied` con módulo y operación.

### Providers/commands policy
- Budget global por query.
- Timeout por provider configurable.
- Cap por host/provider.
- Dedupe determinista con prioridad configurable (`dedupe_source_priority = core_first|provider_first`).
- Namespacing y colisión de comandos:
  - soporte `/modulo::comando`,
  - alias sin namespace solo si owner único,
  - colisión exige namespace explícito.

### Seguridad IPC
- Límite payload request/response.
- Sanitización y validación estricta de `IpcItem` antes del core:
  - obligatorios,
  - longitudes máximas,
  - normalización/trim,
  - descarte seguro de inválidos.

### UI primitives v1
- Quick-select `1..0` estable.
- `quick_select_mode` (`select|submit`) implementado y documentado.
- Conflicto de quick keys: primer visible gana + warning.
- Trailing zones (`label + hint + chip`) robustas en anchos extremos.
- Input accessory con `kind` y prioridad.

### Runtime commands
- `/modules.reload`
- `/modules.list`
- `/modules.telemetry.reset`

---

## 3) Testing actual

Suite verde:
- `cargo test` ✅ (42 tests)

Cobertura clave:
- Parser `.rmod` válido/errores críticos.
- Loader mixto (`directory + .rmod`).
- Resiliencia host (timeout/disable/recovery).
- Runtime commands (`reload/list/telemetry.reset`).
- UI quick-select + chip/hint/accessory.

---

## 4) Config relevante

Sección `[Modules]`:
- `provider_total_budget_ms`
- `provider_timeout_ms`
- `max_items_per_provider_host`
- `dedupe_source_priority`
- `host_restart_backoff_ms`
- `max_ipc_payload_bytes`

Sección `[Behavior]`:
- `quick_select_mode = select|submit`

---

## 5) Documentación final disponible

- `MODULES_EXECUTION_CHECKLIST.md`
- `MODULES_OPERATIONS_GUIDE.md`
- `MODULES_AUTHORING_GUIDE.md`
- `MODULES_CAPABILITIES_MATRIX.md`
- `MODULES_RELEASE_CHECKLIST.md`
- `RMOD_SPEC_V1.md`
- `MANIFEST_SPEC_V1.md`
- `MODULES_API_SPEC_V1.md`
- `CTX_ACTIONS_SPEC_V1.md`
- `PROVIDER_EXECUTION_POLICY.md`
- `ERROR_ISOLATION_POLICY.md`

---

## 6) Criterio de cierre

`Definition of Done v1` marcado en verde en `MODULES_EXECUTION_CHECKLIST.md`.

No quedan pendientes funcionales de los batches 0..9 en este snapshot.

---

## 7) Comandos para retomar en chat nuevo

1. `cargo check`
2. `cargo test`
3. `rmenu --modules-debug`
4. Revisar:
   - `modulesREPORT/IMPLEMENTATION_SNAPSHOT_2026-04-23.md`
   - `modulesREPORT/MODULES_EXECUTION_CHECKLIST.md`
