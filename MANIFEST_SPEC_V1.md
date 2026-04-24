# MANIFEST SPEC V1 (`module.toml`)

Estado: Frozen v1  
Fecha: 2026-04-24

---

## 1. Objetivo

Definir el formato declarativo mínimo para módulos en modo carpeta.

El manifest declara identidad, versión, API, entrypoint y capabilities. El enforcement real siempre ocurre en el runtime del core.

---

## 2. Ubicación

Ubicación por defecto:

```text
modules/<module-folder>/module.toml
```

Convenciones:

- un módulo por carpeta,
- `module.toml` en la raíz de la carpeta,
- entry JS relativo a esa carpeta,
- assets/config/readme junto al manifest,
- el nombre de carpeta no define identidad; la define `name`.

---

## 3. Ejemplo mínimo

```toml
name = "hello-module"
version = "0.1.0"
api_version = 1
kind = "script"
entry = "module.js"
capabilities = ["providers"]
enabled = true
priority = 0
```

---

## 4. Campos obligatorios

- `name` — string no vacío, único entre módulos cargados.
- `version` — string semver recomendado.
- `api_version` — integer, debe ser soportado por el core.
- `kind` — en v1 solo `script`.
- `entry` — archivo JS de entrada, requerido para `kind = "script"`.

---

## 5. Campos opcionales

- `capabilities` — lista de permisos declarados, default `[]`.
- `enabled` — bool, default `true`.
- `priority` — int, default `0`.
- `description` — string.
- `author` — string.
- `homepage` — URL string.

---

## 6. Capabilities v1

Capabilities oficiales:

- `providers`
- `commands`
- `decorate-items`
- `input-accessory`
- `keys`

Reglas:

- declarar solo lo necesario,
- capabilities desconocidas pueden generar warning o rechazo según policy del loader,
- una capability no otorga acceso directo a internals,
- el runtime debe denegar operaciones no cubiertas por capabilities declaradas.

Ver `MODULES_CAPABILITIES_MATRIX.md`.

---

## 7. Validaciones mínimas

El loader debe validar:

1. `name` presente y no vacío.
2. `version` presente y no vacío.
3. `api_version` presente, numérico y soportado.
4. `kind` presente y compatible con v1.
5. `entry` presente para `kind = "script"`.
6. Archivo `entry` existente y legible.
7. `capabilities` parseables como lista.
8. `enabled`, si existe, parseable como bool.
9. `priority`, si existe, parseable como int.

---

## 8. Hot reload v1

Cuando cambia un módulo en modo carpeta:

1. El core detecta cambio.
2. Descarga el módulo afectado.
3. Relee manifest y entry.
4. Reinicia host si corresponde.
5. Re-registra contribuciones.
6. Si falla, el módulo queda degradado/deshabilitado y el error se registra.

No hay migración compleja de estado en v1.

---

## 9. Seguridad operativa

- Manifest no da acceso directo a internals.
- Manifest no ejecuta código por sí mismo.
- Manifest declara intención/capabilities.
- Runtime aplica enforcement.
- Errores de manifest no deben tumbar el launcher.

---

## 10. Compatibilidad futura

- `kind = "script"` es el único kind oficial v1.
- Otros kinds como `theme` o `declarative` quedan reservados para futuras APIs.
- Campos opcionales nuevos pueden agregarse si son ignorables por runtimes v1.
- Cambios incompatibles requieren nueva versión de API o spec.
