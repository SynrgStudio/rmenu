# Modules Capabilities Matrix (v1)

Tabla oficial capability -> operaciones permitidas/enforced.

| Capability | Hook/Action habilitado | Enforce actual | Comportamiento sin permiso |
|---|---|---|---|
| `providers` | `provideItems`, `ctx.registerProvider` | Sí | `permission_denied`, se ignora operación |
| `commands` | `onCommand`, `ctx.registerCommand` | Sí | `permission_denied`, se ignora operación |
| `decorate-items` | `decorateItems` | Sí | `permission_denied`, no se aplica decorate |
| `input-accessory` | `ctx.setInputAccessory`, `ctx.clearInputAccessory` | Sí | `permission_denied`, no cambia accessory |
| `keys` | `onKey` | Sí | `permission_denied`, no se enruta evento |

---

## Notas de enforcement

1. En módulos externos (host IPC), el runtime valida capability antes de ejecutar hook/acción sensible.
2. En ausencia de capability requerida, la operación no rompe el runtime global.
3. El bloqueo por capability registra contexto de módulo + operación.
4. El estado de host y telemetría se mantiene aislado por módulo.

---

## Convención recomendada para autores

- Declarar capacidades mínimas necesarias.
- Evitar declarar capacidades "por si acaso".
- Mantener manifiesto alineado con hooks realmente implementados.
