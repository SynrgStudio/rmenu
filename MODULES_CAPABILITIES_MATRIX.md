# MODULES CAPABILITIES MATRIX v1

Estado: Frozen v1

Tabla oficial capability -> operaciones permitidas/enforced.

| Capability | Operaciones habilitadas | Enforcement | Sin permiso |
|---|---|---|---|
| `providers` | `provideItems`, `ctx.registerProvider` | Sí | `permission_denied`, se ignora operación |
| `commands` | `onCommand`, `ctx.registerCommand` | Sí | `permission_denied`, se ignora operación |
| `decorate-items` | `decorateItems` | Sí | `permission_denied`, no se aplica decoración |
| `input-accessory` | `ctx.setInputAccessory`, `ctx.clearInputAccessory` | Sí | `permission_denied`, no cambia accessory |
| `keys` | `onKey` | Sí | `permission_denied`, no se enruta evento |

---

## 1. Reglas de enforcement

1. El manifest declara intención.
2. El runtime valida capability antes de operación sensible.
3. En ausencia de capability, la operación se rechaza.
4. El rechazo no rompe el runtime global.
5. El rechazo registra módulo + operación + capability requerida.

---

## 2. Declaración mínima recomendada

Declarar solo lo necesario.

Ejemplos:

### Provider simple

```toml
capabilities = ["providers"]
```

### Provider que decora items

```toml
capabilities = ["providers", "decorate-items"]
```

### Módulo con comandos

```toml
capabilities = ["commands"]
```

### Módulo con quick keys/key hooks

```toml
capabilities = ["keys"]
```

---

## 3. Antipatrones

Evitar:

- declarar todas las capabilities por defecto,
- usar `keys` para reemplazar keybindings globales,
- usar `decorate-items` para simular renderer custom,
- depender de que una operación sin permiso falle silenciosamente,
- mezclar muchas responsabilidades en un único módulo sin necesidad.

---

## 4. Relación con otros documentos

- Arquitectura general: `MODULES_ARCHITECTURE.md`.
- API pública: `MODULES_API_SPEC_V1.md`.
- Actions: `CTX_ACTIONS_SPEC_V1.md`.
- Operación/debug: `MODULES_OPERATIONS_GUIDE.md`.
