# Modules Release-Ready Checklist (v1)

Checklist final para declarar el sistema modular listo para release.

## A) Runtime y loader

- [ ] Loader mixto estable (`directory + .rmod`).
- [ ] Hot reload por módulo funcionando sin loops.
- [ ] Errores `.rmod` claros (`RMOD_E_*`).
- [ ] Runtime commands operativos (`/modules.reload`, `/modules.list`, `/modules.telemetry.reset`).

## B) Aislamiento y resiliencia

- [ ] Timeout por request IPC activo.
- [ ] Restart automático de hosts fallidos.
- [ ] Auto-disable por umbral de errores/timeouts.
- [ ] Backoff de restart activo.
- [ ] Telemetría por host + `recent_errors`.

## C) Seguridad y permisos

- [ ] Límite de payload IPC request/response.
- [ ] Sanitización/validación estricta de `IpcItem`.
- [ ] Enforce de capabilities (`providers`, `commands`, `decorate-items`, `input-accessory`, `keys`).
- [ ] Logs `permission_denied` con módulo y operación.

## D) UX/UI primitives

- [ ] Quick-select `1..0` estable y conflict policy aplicada.
- [ ] `quick_select_mode` (`select|submit`) documentado.
- [ ] Layout trailing zones robusto en anchos extremos.
- [ ] Input accessory por `kind` visible y estable.

## E) Tests

- [ ] Parser `.rmod` (válidos + inválidos críticos).
- [ ] Loader mixto (`directory + .rmod`).
- [ ] Resiliencia host (timeout/restart/disable).
- [ ] Runtime commands.
- [ ] Quick-select/chip/hint/accessory.

## F) Documentación

- [ ] Guía de authoring oficial.
- [ ] Tabla oficial de capabilities/permisos.
- [ ] Guía operativa de diagnóstico.
- [ ] Snapshot de implementación actualizado.

---

## Criterio de salida

Marcar release-ready solo cuando todas las secciones A..F estén en verde.
