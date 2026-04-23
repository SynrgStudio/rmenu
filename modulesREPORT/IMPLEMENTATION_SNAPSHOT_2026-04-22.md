# Modules Implementation Snapshot — 2026-04-22

Este snapshot resume estado actual de implementación modular (`rmenu`) para reinicio de chat.

> Nota: este snapshot quedó histórico. Usar como fuente principal:
> `modulesREPORT/IMPLEMENTATION_SNAPSHOT_2026-04-23.md` (estado de cierre actualizado).

---

## 1) Qué quedó implementado

### Runtime modular core
- `src/modules/*` creado y conectado al core (`ModuleRuntime`, hooks, ctx, actions, state).
- Hooks internos funcionando: load/unload/query/decorate + commands/providers base.
- Integración real con UI/ranking pipeline.

### M3 (UI primitives) — mayormente completo
- Quick select `1..0` core-resolved.
- `quick_select_mode` configurable (`select|submit`) en config behavior.
- Layout trailing por zonas (`label + hint + chip`).
- Input Accessory renderizado en input bar.
- Conflicto de quick keys duplicadas: primer visible gana + warning.

### Loader M4
- Parser `.rmod` (`src/modules/rmod.rs`) + validaciones.
- Parser `module.toml` (`src/modules/manifest.rs`).
- Discovery mixto (`src/modules/loader.rs`): carpeta + `.rmod`.
- Normalización a `ModuleDescriptor`.

### Host externo + IPC
- Bin host: `rmenu-module-host` (`src/module_host_main.rs`).
- IPC request/response (`src/modules/ipc.rs`).
- Cliente host (`src/modules/host_client.rs`).
- Runtime JS persistente por host (Node bridge, sin spawn por hook).
- Lifecycle: initialize/load/query/provide/decorate/command/unload/shutdown.

### Hardening
- Timeout por request IPC.
- Auto-restart de host en errores/timeouts.
- Estados host: `loaded | degraded | disabled | unloaded`.
- Auto-disable por umbrales de errores/timeouts consecutivos.
- Backoff de restart (`host_restart_backoff_ms`).
- Telemetría por host + `recent_errors` ring buffer.
- Flag `--modules-debug` + runtime commands:
  - `/modules.reload`
  - `/modules.list`
  - `/modules.telemetry.reset`

### Capabilities (enforce parcial útil)
- Enforce para hosts externos:
  - `providers` en provide_items
  - `commands` en on_command
  - `decorate-items` en decorate_items
- Enforce de actions vía `apply_ctx_requests` para:
  - `registerProvider`
  - `registerCommand`
  - `set/clearInputAccessory`

### Seguridad IPC (parcial)
- Límite payload request/response en host client y module host.
- Configurable por `max_ipc_payload_bytes`.

### Config modular
- Nueva sección `[Modules]` en `config_example.ini` y parser en `settings.rs`:
  - `provider_total_budget_ms`
  - `provider_timeout_ms`
  - `max_items_per_provider_host`
  - `host_restart_backoff_ms`
  - `max_ipc_payload_bytes`

### Docs creadas/actualizadas
- `modulesREPORT/MODULES_EXECUTION_CHECKLIST.md`
- `modulesREPORT/MODULES_OPERATIONS_GUIDE.md`
- `modulesREPORT/RMOD_SPEC_V1.md`
- templates y examples (`templates/rmod`, `templates/directory`, `example-modules/...`)
- `modulesREPORT/README.md` y `CHANGELOG.md` actualizados

### Tests
- Parser `.rmod` válido + inválidos principales.
- Loader mixto (`directory + .rmod`).
- Suite total actual: `cargo test` verde (23 tests).

---

## 2) Qué falta para cerrar (pendiente)

### Batch 6
- Dedupe determinista final con prioridad de fuente configurable.
- Namespacing y colisión de comandos (política formal + enforce).

### Batch 7
- Validación/sanitización estricta de `IpcItem` antes de entrar al core:
  - campos obligatorios,
  - longitudes máximas,
  - trimming/normalización,
  - descarte seguro de items inválidos.

### Batch 8
- Tests timeout/restart/disable host (regresión de resiliencia).
- Tests E2E de runtime commands (`/modules.reload`, `/modules.list`, `/modules.telemetry.reset`).
- Tests UI/UX quick-select + chip/hint/accessory.

### Batch 9
- Guía authoring final completa (`.rmod` + carpeta) en formato release-ready.
- Tabla final de capabilities/permisos.
- Checklist release-ready final de módulos.

---

## 3) Archivos clave tocados recientemente

- `src/modules/mod.rs`
- `src/modules/host_client.rs`
- `src/modules/ipc.rs`
- `src/modules/rmod.rs`
- `src/modules/loader.rs`
- `src/modules/manifest.rs`
- `src/module_host_main.rs`
- `src/ui_win32.rs`
- `src/settings.rs`
- `src/main.rs`
- `config_example.ini`
- `modulesREPORT/MODULES_EXECUTION_CHECKLIST.md`
- `modulesREPORT/MODULES_OPERATIONS_GUIDE.md`

---

## 4) Comandos para retomar en chat nuevo

1. `cargo check`
2. `cargo test`
3. `rmenu --modules-debug`
4. Revisar:
   - `modulesREPORT/MODULES_EXECUTION_CHECKLIST.md`
   - `modulesREPORT/IMPLEMENTATION_SNAPSHOT_2026-04-22.md`

---

## 5) Siguiente bloque recomendado (orden)

1. Sanitización estricta `IpcItem` (Batch 7)
2. Namespacing/colisión de comandos (Batch 6)
3. Tests de resiliencia host (Batch 8)
4. Cierre docs authoring/permissions/release (Batch 9)
