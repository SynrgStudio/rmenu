# MODULES OPERATIONS GUIDE v1

Estado: Frozen v1

Guía de operación, diagnóstico y recuperación del sistema modular de `rmenu`.

---

## 1. Comandos runtime

- `/modules.reload` — recarga módulos externos.
- `/modules.list` — lista módulos cargados.
- `/modules.telemetry.reset` — limpia telemetría de hosts.

---

## 2. Flags CLI

```powershell
rmenu.exe --modules-debug
```

Imprime estado/telemetría de módulos y termina.

---

## 3. Estados de host

- `loaded` — host operativo.
- `degraded` — host con errores recientes.
- `disabled` — host deshabilitado por policy o configuración.
- `unloaded` — descriptor presente sin host activo.

---

## 4. Instalar módulos

### `.rmod`

```powershell
mkdir modules
copy .\my-module.rmod .\modules\my-module.rmod
rmenu.exe --modules-debug
```

### Carpeta

```text
modules/
  my-module/
    module.toml
    module.js
```

Validar:

```powershell
rmenu.exe --modules-debug
```

Ver también `MODULES_QUICKSTART.md`.

---

## 5. Límites operativos

En `[Modules]` de `config.ini`:

```ini
provider_total_budget_ms = 35
provider_timeout_ms = 1500
max_items_per_provider_host = 24
dedupe_source_priority = core_first
host_restart_backoff_ms = 800
max_ipc_payload_bytes = 262144
```

### `dedupe_source_priority`

Valores:

- `core_first`
- `provider_first`

---

## 6. Policy quick-select v1

### `quick_select_mode = select`

Tecla `1..0` mueve selección al item visible con `quickSelectKey`.

No ejecuta submit automáticamente.

### `quick_select_mode = submit`

Tecla `1..0` selecciona y ejecuta submit inmediato del item visible.

### Conflictos

Si varios items usan la misma key:

- primer item visible gana,
- duplicados posteriores pierden key/badge,
- se registra warning.

---

## 7. Policy de comandos

Formato namespaced:

```text
/modulo::comando
```

Reglas:

- si un alias pertenece a un solo módulo, puede enrutarse de forma determinista,
- si un alias pertenece a múltiples módulos, el alias sin namespace se rechaza,
- en colisión, usar namespace explícito.

---

## 8. Errores comunes

### `RMOD_E_INVALID_MAGIC`

El archivo `.rmod` no comienza con:

```text
#!rmod/v1
```

### `RMOD_E_MISSING_MODULE_JS`

Falta bloque:

```text
---module.js---
```

### `permission_denied`

El módulo no declaró la capability requerida.

Ejemplo:

```text
permission_denied module='x' operation='provide_items' capability='providers'
```

### `module-host timed out`

El host no respondió dentro del timeout configurado.

### `module '<name>' disabled after repeated failures`

El módulo superó umbral de errores/timeouts consecutivos.

---

## 9. Flujo recomendado de diagnóstico

1. Ejecutar:

```powershell
rmenu.exe --modules-debug
```

2. Revisar:

- estado del host,
- `request_count`,
- `error_count`,
- `timeout_count`,
- `restart_count`,
- `recent_errors`.

3. Corregir manifest/capabilities/script.

4. Recargar:

```text
/modules.reload
```

5. Confirmar que el estado vuelve a `loaded`.

---

## 10. Recuperación

Si un módulo queda roto:

1. Deshabilitarlo en manifest (`enabled = false`) o moverlo fuera de `modules/`.
2. Ejecutar `/modules.reload` o reiniciar `rmenu`.
3. Corregir errores.
4. Rehabilitar.
5. Validar con `--modules-debug`.

---

## 11. Señales de salud

Un sistema saludable debería mostrar:

- hosts en `loaded`,
- pocos o cero errores recientes,
- timeouts en cero,
- restarts bajos,
- latencias estables,
- descriptors externos iguales a módulos esperados.
