# CTX & ACTIONS SPEC V1

Estado: Frozen v1  
Fecha: 2026-04-24

---

## 1. Objetivo

Definir el comportamiento de `ctx` y de las actions permitidas para módulos.

`ctx` es una fachada controlada:

- lectura segura,
- utilidades de diagnóstico,
- mutaciones solo mediante actions validadas,
- sin referencias directas a estado interno mutable.

---

## 2. Principios

1. El core mantiene autoridad sobre estado.
2. Un módulo solicita cambios; no muta internals.
3. Toda action puede ser aceptada, normalizada o rechazada.
4. Errores de action no deben tumbar el launcher.
5. El resultado debe ser determinista para el mismo estado de entrada.

---

## 3. Lecturas

```ts
ctx.query() -> string
ctx.items() -> Item[]
ctx.selectedItem() -> Item | null
ctx.selectedIndex() -> number
ctx.mode() -> "launcher" | "stdin" | "command"
ctx.hasCapability(name: string) -> boolean
```

### Reglas

- Las lecturas son snapshot coherente del ciclo actual.
- El snapshot puede quedar obsoleto luego de aplicar actions.
- No se exponen punteros mutables a estructuras internas.
- `ctx.items()` puede estar normalizado/truncado respecto a fuentes originales.

---

## 4. Utilidades

```ts
ctx.log(message: string)
ctx.toast(message: string)
```

### Reglas

- `log` se integra a observabilidad del módulo.
- `toast` puede ser rate-limited, ignorado o degradado por el core.
- Mensajes excesivamente largos pueden truncarse.
- Ninguna utilidad debe bloquear UI.

---

## 5. Actions de query y selección

```ts
ctx.setQuery(text)
ctx.setSelection(index)
ctx.moveSelection(offset)
```

### Validaciones

- `text` puede ser truncado o normalizado por policy futura.
- `setSelection(index)` puede fallar si el índice está fuera de rango.
- `moveSelection(offset)` debe clamplear al rango visible válido.
- Cambios de selección no deben romper invariantes de scroll.

---

## 6. Actions de flujo

```ts
ctx.submit()
ctx.close()
```

### Reglas

- `submit()` solicita ejecutar el item seleccionado actual.
- `close()` solicita cerrar la UI actual.
- El core puede ignorar la solicitud si el estado no permite la operación.
- Estas actions no otorgan control directo sobre el proceso del launcher.

---

## 7. Actions de contenido

```ts
ctx.addItems(items)
ctx.replaceItems(items)
```

### Reglas

- Items se validan con el mismo modelo que providers.
- Items inválidos pueden descartarse.
- `addItems` agrega al dataset controlado por el ciclo actual.
- `replaceItems` es restrictivo y puede estar limitado por contexto/policy.
- El core conserva autoridad sobre merge, dedupe y ranking final.

---

## 8. Registro dinámico

```ts
ctx.registerProvider(def)
ctx.registerCommand(def)
```

### Capabilities requeridas

- `registerProvider` requiere `providers`.
- `registerCommand` requiere `commands`.

### Reglas

- Registros se asocian a la identidad del módulo.
- Colisiones de comandos se resuelven por namespacing.
- Registros inválidos se rechazan y registran.

---

## 9. UI primitive

```ts
ctx.setInputAccessory(accessory)
ctx.clearInputAccessory()
```

### Capability requerida

- `input-accessory`

### Reglas

- Solo un accessory visible a la vez.
- Mayor prioridad gana.
- `clearInputAccessory` elimina el accessory activo cuando corresponde.
- El core decide render, color, truncado y layout.

---

## 10. Errores de actions

Cada action debe terminar en uno de estos resultados conceptuales:

```ts
type ActionResult =
  | { ok: true }
  | { ok: false, error: ActionError }
```

Errores conceptuales:

- `invalid_state`
- `permission_denied`
- `invalid_payload`
- `invalid_selection`
- `timeout`
- `internal_error`

Reglas:

- El core debe loggear error con identidad del módulo.
- El módulo defectuoso no debe romper el runtime global.
- Errores repetidos pueden contribuir a degradación/desactivación.

---

## 11. Permission model

Cada operación sensible requiere capability declarada.

| Operación | Capability |
|---|---|
| `provideItems` / `registerProvider` | `providers` |
| `onCommand` / `registerCommand` | `commands` |
| `decorateItems` | `decorate-items` |
| `setInputAccessory` / `clearInputAccessory` | `input-accessory` |
| `onKey` | `keys` |

Sin capability:

- la operación se deniega,
- se registra `permission_denied`,
- el launcher continúa.

---

## 12. Determinismo y orden

- Actions se aplican en orden determinista por ciclo.
- Reentrancia no controlada debe bloquearse.
- Efectos secundarios ocultos entre módulos deben evitarse.
- Providers y decorators no deben depender de orden no documentado.
- Cuando exista prioridad, se ordena por prioridad y luego nombre estable.

---

## 13. Relación con specs

- Modelo de hooks e items: `MODULES_API_SPEC_V1.md`.
- Capabilities: `MODULES_CAPABILITIES_MATRIX.md`.
- Error isolation: `ERROR_ISOLATION_POLICY.md`.
- Arquitectura general: `MODULES_ARCHITECTURE.md`.
