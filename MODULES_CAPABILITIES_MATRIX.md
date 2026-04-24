# MODULES CAPABILITIES MATRIX v1

Status: Frozen v1

| Capability | Allows | Required | If missing |
|---|---|---|---|
| `providers` | `provideItems`, `ctx.registerProvider` | Yes | `permission_denied`, operation ignored |
| `commands` | `onCommand`, `ctx.registerCommand` | Yes | `permission_denied`, operation ignored |
| `decorate-items` | `decorateItems` | Yes | `permission_denied`, decoration not applied |
| `input-accessory` | `ctx.setInputAccessory`, `ctx.clearInputAccessory` | Yes | `permission_denied`, accessory unchanged |
| `keys` | `onKey` | Yes | `permission_denied`, event not routed |

---

## 1. Enforcement rules

1. The manifest declares intent.
2. The runtime validates capability before sensitive operations.
3. Without the capability, the operation is rejected.
4. The module remains isolated.
5. Rejection records module + operation + required capability.

---

## 2. Recommended minimum declaration

Declare the smallest set that supports the module behavior.

Examples:

### Pure provider

```toml
capabilities = ["providers"]
```

### Module with commands

```toml
capabilities = ["commands"]
```

### Module with input accessory

```toml
capabilities = ["input-accessory"]
```

### Module with quick keys/key hooks

```toml
capabilities = ["keys"]
```

### Combined module

```toml
capabilities = ["providers", "commands", "input-accessory"]
```

---

## 3. Anti-patterns

Avoid:

- declaring all capabilities “just in case”,
- depending on unauthorized operations failing silently,
- mixing too many responsibilities into one module without need.

---

## 4. Related documents

- Public API: `MODULES_API_SPEC_V1.md`.
- Action semantics: `CTX_ACTIONS_SPEC_V1.md`.
- Operations/debugging: `MODULES_OPERATIONS_GUIDE.md`.
