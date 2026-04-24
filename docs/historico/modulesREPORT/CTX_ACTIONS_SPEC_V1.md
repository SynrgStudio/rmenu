# CTX & ACTIONS SPEC V1

Estado: Draft v1  
Fecha: 2026-04-22

---

## 1) Objetivo

Definir comportamiento exacto de `ctx` y actions permitidas para módulos.

---

## 2) Diseño

`ctx` es una fachada controlada:

- lectura segura,
- utilidades de diagnóstico,
- mutaciones solo vía actions validadas.

Sin referencias directas a estado interno mutable.

---

## 3) Lecturas

- `ctx.query() -> string`
- `ctx.items() -> Item[]` (vista actual)
- `ctx.selectedItem() -> Item | null`
- `ctx.selectedIndex() -> number`
- `ctx.mode() -> "launcher" | "stdin" | "command" | ...`
- `ctx.hasCapability(name: string) -> boolean`

### Reglas

- Lecturas son snapshot coherente del ciclo actual.
- No exponer punteros mutables a estructuras internas.

---

## 4) Utilidades

- `ctx.log(message: string)`
- `ctx.toast(message: string)`

### Reglas

- `log` se integra a observabilidad de módulos.
- `toast` puede ser rate-limited por core.

---

## 5) Actions (mutación)

### Query/selección

- `ctx.setQuery(text)`
- `ctx.setSelection(index)`
- `ctx.moveSelection(offset)`

Validaciones:

- índices fuera de rango deben clamp/ignorar según policy.
- no romper invariantes de scroll/selección visible.

### Flujo

- `ctx.submit()`
- `ctx.close()`

### Contenido

- `ctx.addItems(items)`
- `ctx.replaceItems(items)` (solo contextos habilitados)

### Registro dinámico

- `ctx.registerProvider(def)`
- `ctx.registerCommand(def)`

### UI primitive

- `ctx.setInputAccessory(accessory)`
- `ctx.clearInputAccessory()`

---

## 6) Errores de actions

Cada action devuelve:

- éxito, o
- error tipado (invalid state, permission denied, invalid payload, etc.)

Core debe loggear error con identidad de módulo.

---

## 7) Permission model

Cada action/hook requiere capability declarada.

Ejemplos:

- `registerProvider` requiere `providers`
- `registerCommand` requiere `commands`
- `setInputAccessory` requiere `input-accessory`
- `onKey` requiere `keys`

---

## 8) Determinismo

- Actions deben aplicarse en orden determinista por ciclo.
- Reentrancia no controlada debe bloquearse.
- Evitar efectos secundarios ocultos entre módulos.
