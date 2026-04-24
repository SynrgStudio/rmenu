# Modules Operations Guide (v1)

## Comandos runtime

- `/modules.reload` — recarga módulos externos
- `/modules.list` — muestra módulos cargados
- `/modules.telemetry.reset` — limpia telemetría de hosts

## Flags CLI

- `--modules-debug` — imprime estado/telemetría de módulos y termina

## Señales de estado host

- `loaded` — host operativo
- `degraded` — host con errores recientes
- `disabled` — deshabilitado por umbrales de errores/timeouts
- `unloaded` — descriptor presente sin host activo

## Policy quick-select (final v1)

- `quick_select_mode = select`
  - Tecla `1..0` mueve selección al item visible con `quick_select_key`.
  - No ejecuta submit automáticamente.
- `quick_select_mode = submit`
  - Tecla `1..0` selecciona y ejecuta submit inmediato del item visible.
- Resolución de conflictos de `quick_select_key` duplicada:
  - primer item visible gana,
  - duplicados posteriores limpian key/badge,
  - se registra warning.

## Límites operativos (configurables)

En sección `[Modules]` de `config.ini`:

- `provider_total_budget_ms`
- `provider_timeout_ms`
- `max_items_per_provider_host`
- `dedupe_source_priority` (`core_first` | `provider_first`)
- `host_restart_backoff_ms`
- `max_ipc_payload_bytes`

## Policy de comandos (namespacing y colisiones)

- Formato namespaced: `/modulo::comando`.
- Si un alias de comando está registrado por múltiples módulos, el alias sin namespace se rechaza.
- En colisión, usar siempre namespace explícito.
- Si no hay colisión y el alias está registrado por un único módulo, se enruta de forma determinista a ese módulo.

## Errores comunes

1. `RMOD_E_INVALID_MAGIC`
   - El archivo `.rmod` no comienza con `#!rmod/v1`.

2. `permission_denied ... capability='providers|commands|decorate-items|keys'`
   - El módulo no declaró la capability requerida.

3. `module-host timed out ...`
   - Host no respondió dentro del timeout configurado.

4. `module '<name>' disabled after repeated failures`
   - Host superó umbral de fallos/timeouts consecutivos.

## Flujo recomendado de diagnóstico

1. Ejecutar `rmenu --modules-debug`
2. Revisar `host_telemetry` y `recent_errors`
3. Corregir manifiesto/capabilities o script JS
4. Ejecutar `/modules.reload`
5. Verificar estado vuelva a `loaded`
