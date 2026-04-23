# MANIFEST SPEC V1 (`module.toml`)

Estado: Draft v1  
Fecha: 2026-04-22

---

## 1) Objetivo

Definir formato declarativo mínimo de módulo para carga/validación.

---

## 2) Ejemplo

```toml
name = "numeric-shortcuts"
version = "0.1.0"
api_version = 1
kind = "script"
entry = "index.js"

capabilities = ["keys", "decorate-items", "providers", "commands"]

enabled = true
priority = 0
```

---

## 3) Campos obligatorios

- `name` (string, único)
- `version` (semver string)
- `api_version` (int)
- `kind` (`script` | `theme` | `declarative`)

---

## 4) Campos opcionales

- `entry` (archivo de entrada para módulos script)
- `capabilities` (lista de permisos declarados)
- `enabled` (bool, default `true`)
- `priority` (int, default `0`)
- `description` (string)
- `author` (string)
- `homepage` (url string)

---

## 5) Validaciones mínimas

1. `name` no vacío y sin colisiones.
2. `api_version` soportada por core.
3. `kind=script` requiere `entry` existente.
4. `capabilities` deben ser conocidas por core.
5. `priority` dentro de rango permitido.

---

## 6) Capabilities sugeridas v1

- `keys`
- `decorate-items`
- `providers`
- `commands`
- `input-accessory`

Core debe denegar uso de APIs no habilitadas por capability.

---

## 7) Descubrimiento de módulos

Ubicación por defecto:

- `modules/<module-name>/module.toml`

Convención:

- un módulo por carpeta,
- assets y entry junto al manifest,
- nombre de carpeta no define identidad (la define `name`).

---

## 8) Hot reload v1

- Al detectar cambio: unload + reload + re-register.
- Si falla reload: módulo queda deshabilitado y se loggea error.
- No hay migración de estado compleja en v1.

---

## 9) Seguridad operativa

- Manifest no da acceso directo a internals.
- Manifest solo declara intención/capabilities.
- Enforcement siempre en runtime core.
