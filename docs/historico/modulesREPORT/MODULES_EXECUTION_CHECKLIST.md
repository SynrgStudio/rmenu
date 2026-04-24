# MODULES EXECUTION CHECKLIST (Batches progresivos)

Estado: Activo  
Fecha: 2026-04-22

## Batch 0 — Planificación y freeze de contrato
- [x] Congelar dirección `.rmod` texto plano + carpeta dev.
- [x] Definir `RMOD_SPEC_V1.md`.
- [x] Definir templates (`.rmod` y `directory`).
- [x] Crear parser/loader base para `.rmod` y `module.toml`.

## Batch 1 — Hardening inicial runtime/host
- [x] Host externo por módulo con IPC base.
- [x] Runtime JS persistente (sin spawn por hook).
- [x] Timeout por request IPC + kill de host colgado.
- [x] Auto-restart de host por error/timeout.
- [x] `--modules-debug` con telemetría base.
- [x] Ring buffer de últimos errores por host.
- [x] Comando runtime `/modules.telemetry.reset`.
- [x] Códigos de error `.rmod` (`RMOD_E_*`) en loader.

## Batch 2 — Permisos/capabilities enforce (crítico)
- [x] Matriz capability -> acción/hook permitidos en código (base v1: providers/commands/input-accessory).
- [x] Denegar `registerProvider` si no declara `providers`.
- [x] Denegar `registerCommand` si no declara `commands`.
- [x] Denegar `setInputAccessory` si no declara `input-accessory`.
- [x] Denegar `onKey`/atajos si no declara `keys`.
- [x] Registrar `permission_denied` con módulo + operación.

## Batch 3 — Estados formales de módulo + políticas de degradación
- [x] Introducir estados: `loaded | degraded | disabled | unloaded`.
- [x] Umbral por código (v1): fallos consecutivos por módulo.
- [x] Umbral por código (v1): timeouts consecutivos por módulo.
- [x] Auto-disable al superar umbrales.
- [x] Reset de contadores en reload/restart exitoso.

## Batch 4 — Loader/hot reload granular
- [x] Watcher por polling en runtime de `modules/`.
- [x] Reload por módulo (no global) al cambiar descriptor (`.rmod` o directorio).
- [x] Unload/reload por módulo afectado.
- [x] Protección contra loops de reload continuos (debounce).

## Batch 5 — UI primitives v1 cierre de calidad
- [x] Formalizar conflicto quick-select duplicadas (primer visible gana + warning).
- [x] Consolidar layout zones en anchos extremos.
- [x] Integrar colores por `InputAccessory.kind`.
- [x] Documentar policy final `quick_select_mode`.

## Batch 6 — Providers/commands policy completa
- [x] Budget global por query para providers.
- [x] Cap por provider host.
- [x] Timeout por provider configurable (`[Modules].provider_timeout_ms`).
- [x] Dedupe determinista final con prioridad de fuente.
- [x] Namespacing y colisión de comandos.

## Batch 7 — Seguridad operacional IPC
- [x] Límite de tamaño de payload IPC (request/response).
- [x] Validación estricta de `IpcItem` antes de entrar al core.
- [x] Sanitización de strings (longitud máxima, campos obligatorios).
- [x] Backoff para restarts recurrentes de un host.

## Batch 8 — Testing completo
- [x] Tests parser `.rmod` inválidos (subset principal `RMOD_E_*`).
- [x] Tests loader mixto (`directory + .rmod`).
- [x] Tests timeout/restart/disable host.
- [x] Tests comando runtime (`/modules.reload`, `/modules.list`, `/modules.telemetry.reset`).
- [x] Tests quick-select/chip/hint/accessory.

## Batch 9 — Documentación final de operación
- [x] Guía authoring oficial (`.rmod` y carpeta).
- [x] Guía de debugging (`--modules-debug`, errores frecuentes).
- [x] Tabla de capacidades y permisos.
- [x] Checklist release-ready de módulos.

## Definition of Done v1 (resumen)
- [x] Loader `.rmod` + carpeta estable.
- [x] Runtime + host con aislamiento real y recuperación.
- [x] Capabilities enforceadas.
- [x] UI primitives v1 cerradas.
- [x] Tests críticos verdes.
- [x] Docs de authoring y operación cerradas.
