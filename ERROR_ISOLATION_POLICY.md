# ERROR ISOLATION POLICY — Modules

Estado: Frozen v1  
Fecha: 2026-04-24

---

## 1. Objetivo

Evitar que errores de un módulo comprometan la estabilidad del launcher.

Principio:

> Fail module, not launcher.

El fallo de un módulo debe degradar funcionalidad opcional, no romper el core.

---

## 2. Alcance

La política aplica a:

- load,
- reload,
- unload,
- hooks,
- providers,
- decorators,
- commands,
- key hooks,
- actions solicitadas por módulos,
- IPC,
- host externo.

---

## 3. Estados de módulo

Estados v1:

- `loaded` — módulo operativo.
- `degraded` — módulo con errores recientes, aún recuperable.
- `disabled` — módulo desactivado por policy o configuración.
- `unloaded` — descriptor presente sin host activo.

Transiciones deben ser observables vía debug/log.

---

## 4. Estrategia general

Ante error:

1. Capturar error en boundary de módulo.
2. Registrar contexto:
   - módulo,
   - operación/hook,
   - latencia,
   - timeout si aplica,
   - mensaje de error.
3. Devolver estado seguro al core.
4. Continuar pipeline sin ese resultado.
5. Actualizar telemetría.
6. Reiniciar/degradar/deshabilitar si policy lo requiere.

---

## 5. Timeouts

Un timeout se trata como fallo recuperable al principio.

Si hay timeouts consecutivos:

- se incrementa contador del host,
- se registra en telemetría,
- puede reiniciarse el host,
- al superar umbral, el módulo pasa a `disabled`.

Timeouts no deben congelar la UI.

---

## 6. Errores consecutivos

Errores consecutivos pueden llevar a degradación/desactivación.

Umbrales conceptuales v1:

- `max_consecutive_errors_per_module`,
- `max_consecutive_timeouts_per_module`.

Cuando se supera un umbral:

- el módulo pasa a `disabled`,
- el host se detiene o deja de recibir requests,
- el error queda visible en debug.

---

## 7. Reinicio de host

Ante fallo recuperable:

1. Core marca error.
2. Core intenta reiniciar host si policy lo permite.
3. Reinicios respetan `host_restart_backoff_ms`.
4. Un reinicio exitoso limpia contadores relevantes.
5. Reinicios recurrentes terminan en degradación o disable.

---

## 8. Load/reload fallido

Si falla carga o reload:

- el launcher continúa,
- el módulo afectado queda `degraded` o `disabled`,
- se registra error,
- otros módulos no se descargan salvo que dependan explícitamente de esa operación global.

Hot reload no debe entrar en loop continuo; debe tener debounce/backoff.

---

## 9. Permission denied

Si un módulo usa una operación sin capability:

- operación se rechaza,
- se registra `permission_denied`,
- no se aplica efecto parcial,
- el launcher continúa.

Violaciones repetidas pueden considerarse error del módulo.

---

## 10. Payload inválido

Payload inválido debe tratarse como error de módulo/host, no del core.

El core puede:

- descartar item,
- truncar campo,
- normalizar campo,
- rechazar response completa,
- registrar error.

Nunca debe crashear por payload inválido.

---

## 11. UX ante fallo

Por defecto:

- launcher sigue operativo,
- no hay crash visible,
- errores van a debug/stderr/log,
- `--silent` suprime diagnósticos no críticos,
- `--modules-debug` expone detalle operacional.

Toasts de error son opcionales y deben ser no intrusivos.

---

## 12. Recuperación

Un módulo puede recuperarse mediante:

- hot reload exitoso,
- `/modules.reload`,
- corrección de manifest/script,
- restart del launcher.

Reload exitoso debe resetear contadores relevantes.

---

## 13. Observabilidad mínima

`--modules-debug` debe mostrar:

- `api_version`,
- cantidad de módulos builtin,
- descriptors externos,
- hosts activos,
- módulos cargados,
- estado por host,
- requests,
- errores,
- timeouts,
- restarts,
- latencia promedio/máxima,
- últimos errores.

---

## 14. Casos mínimos de test

- Hook lanza error.
- Provider timeout.
- Provider devuelve payload inválido.
- Command inválido.
- Action denegada por permisos.
- Reload fallido.
- Host muere inesperadamente.
- Payload IPC excede límite.

Cada caso debe verificar:

- core sigue estable,
- error registrado,
- módulo entra al estado esperado,
- otros módulos siguen funcionando.
