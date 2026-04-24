# CTX ACTIONS SPEC V1

Status: Frozen v1  
Date: 2026-04-24

---

## 1. Objective

Define the behavior of `ctx` and the actions modules are allowed to request.

`ctx` is a controlled facade. It provides:

- read-only state snapshots,
- diagnostic utilities,
- action requests validated by the core.

---

## 2. Core rules

1. A module never mutates internal core state directly.
2. A module requests changes through actions.
3. The core validates state, payload, and capabilities.
4. The core may reject an action.
5. Rejected actions must not break the launcher.

---

## 3. Read methods

```ts
ctx.query() -> string
ctx.items() -> Item[]
ctx.selectedItem() -> Item | null
ctx.selectedIndex() -> number
ctx.mode() -> "launcher" | "stdin" | "command"
ctx.hasCapability(name: string) -> boolean
```

Rules:

- values are snapshots,
- returned values are not mutable references to core internals,
- snapshots may be stale by the next event tick.

---

## 4. Utilities

```ts
ctx.log(message)
ctx.toast(message)
```

Rules:

- `log` integrates with module observability.
- `toast` is a request for user feedback; the core may ignore or coalesce it.
- Utilities must not expose UI internals.

---

## 5. Query and selection actions

```ts
ctx.setQuery(text)
ctx.setSelection(index)
ctx.moveSelection(offset)
```

Rules:

- `setQuery` requests replacement of current input.
- `setSelection(index)` may fail when the index is out of range.
- `moveSelection(offset)` clamps to the valid visible range.
- Selection changes must preserve scroll invariants.

---

## 6. Submit and close

```ts
ctx.submit()
ctx.close()
```

Rules:

- `submit` requests normal submit behavior for the current state.
- `close` requests launcher close.
- The core may ignore the request if the state does not allow the operation.

---

## 7. Item actions

```ts
ctx.addItems(items)
ctx.replaceItems(items)
```

Rules:

- items are validated and sanitized before use.
- invalid items may be discarded.
- `addItems` appends to the module-visible item list.
- `replaceItems` replaces the visible item list for the current cycle.
- `replaceItems` can be used by scoped intent modules to avoid global fuzzy competition.

---

## 8. Dynamic registration

```ts
ctx.registerCommand(def)
ctx.registerProvider(def)
```

Rules:

- `registerCommand` requires `commands`.
- `registerProvider` requires `providers`.
- registrations are associated with module identity.
- command collisions are resolved by namespacing.
- invalid registrations are rejected and recorded.

---

## 9. Input accessory

```ts
ctx.setInputAccessory(accessory)
ctx.clearInputAccessory()
```

Rules:

- requires `input-accessory`.
- only one input accessory is visible at a time.
- higher priority wins.
- the core decides rendering, color, truncation, and layout.

---

## 10. Errors

If an action fails validation:

- the action is ignored,
- the error is recorded with module identity,
- the faulty module must not break the global runtime,
- repeated errors may contribute to degradation/disable.

---

## 11. Capability enforcement

Each sensitive operation requires its declared capability.

| Operation | Capability |
|---|---|
| `registerProvider` | `providers` |
| `registerCommand` | `commands` |
| `setInputAccessory`, `clearInputAccessory` | `input-accessory` |
| `onKey` routing | `keys` |
| `decorateItems` routing | `decorate-items` |

If missing:

- the operation is denied,
- `permission_denied` is recorded,
- the launcher continues.

---

## 12. Determinism

- Action application order must be stable.
- Hidden side effects between modules should be avoided.
- The core remains the authority over final UI state.

---

## 13. Related specs

- `MODULES_API_SPEC_V1.md`
- `MODULES_CAPABILITIES_MATRIX.md`
- `PROVIDER_EXECUTION_POLICY.md`
- `ERROR_ISOLATION_POLICY.md`
