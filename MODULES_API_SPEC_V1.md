# MODULES API SPEC V1

Status: Frozen v1  
Date: 2026-04-24

---

## 1. Objective

Define the minimum public contract for `rmenu` modules.

Rule:

> Small, stable API with no exposure of internals.

The intent of v1 is to allow useful modules without letting a module control the core, renderer, or event loop.

---

## 2. Versioning

- Current version: `api_version = 1`.
- Every module must declare `api_version = 1` in `.rmod` or `module.toml`.
- Breaking changes require a new API version.
- Additive extensions may be added to v1 only when optional and ignorable by existing modules.

---

## 3. Distribution formats

A module can be distributed as:

1. development directory with `module.toml` + entry JS,
2. single-file `.rmod`.

Both formats normalize to the same internal descriptor.

---

## 4. Hooks v1

Public hooks:

```ts
onLoad(ctx)
onUnload(ctx)
onQueryChange(query, ctx)
onSelectionChange(item, index, ctx)
onKey(event, ctx)
onSubmit(item, ctx)
onCommand(command, args, ctx)
provideItems(query, ctx) -> Item[]
decorateItems(items, ctx) -> Item[]
```

Rules:

- hooks must be fast and deterministic,
- hooks must not block the UI loop,
- errors are isolated per module,
- the core may skip hooks when the corresponding capability is missing,
- the core may discard stale, slow, or invalid responses.

---

## 5. Context (`ctx`) v1

`ctx` is a controlled facade. It does not expose mutable references to internal state.

### Reads

```ts
ctx.query() -> string
ctx.items() -> Item[]
ctx.selectedItem() -> Item | null
ctx.selectedIndex() -> number
ctx.mode() -> "launcher" | "stdin" | "command"
ctx.hasCapability(name: string) -> boolean
```

### Utilities

```ts
ctx.log(message: string)
ctx.toast(message: string)
```

### Mutations through actions

```ts
ctx.setQuery(text)
ctx.setSelection(index)
ctx.moveSelection(offset)
ctx.submit()
ctx.close()
ctx.addItems(items)
ctx.replaceItems(items)
ctx.registerCommand(def)
ctx.registerProvider(def)
ctx.setInputAccessory(accessory)
ctx.clearInputAccessory()
```

Rules:

- every mutation is a request, not direct mutation,
- the core validates state, payload, and permissions,
- the core may reject an action and record an error,
- detailed semantics are defined in `CTX_ACTIONS_SPEC_V1.md`.

---

## 6. Item v1

```ts
type Item = {
  id: string
  title: string
  subtitle?: string
  source?: string
  target?: string
  quickSelectKey?: string
  badge?: string
  hint?: string
}
```

Fields:

- `id`: stable identifier within its source.
- `title`: main visible text.
- `subtitle`: optional detail.
- `source`: visible or logical source.
- `target`: destination to launch when the item represents a direct launch.
- `quickSelectKey`: visible quick key (`"1".."9"|"0"`).
- `badge`: short trailing text.
- `hint`: contextual help.

Rules:

- `id` and `title` are required,
- invalid fields may be trimmed, normalized, or discarded,
- items without executable action may behave as `noop`,
- the core decides rendering, final ranking, and dedupe.

---

## 7. Input Accessory v1

```ts
type InputAccessory = {
  text: string
  kind?: "info" | "success" | "warning" | "error" | "hint"
  priority?: number
}
```

Rules:

- requires `input-accessory`,
- only one accessory is visible at a time,
- higher priority wins,
- the core decides colors, position, truncation, and layout.

---

## 8. Commands v1

```ts
type CommandDef = {
  name: string
  description?: string
}
```

Rules:

- requires `commands`,
- recommended name: `/module::command`,
- alias without namespace is allowed only when there is no collision,
- collisions are rejected deterministically.

---

## 9. Providers v1

```ts
type ProviderDef = {
  name: string
  priority?: number
}
```

Rules:

- requires `providers`,
- responses are subject to global per-query budget,
- responses are subject to host/provider timeout,
- responses are subject to item caps,
- the core performs merge, dedupe, and final ranking.

---

## 10. Key Event v1

```ts
type KeyEvent = {
  key: string
  ctrl: boolean
  alt: boolean
  shift: boolean
  meta: boolean
}
```

Rules:

- requires `keys`,
- does not replace the core keybinding system,
- does not allow interception of the full event loop.

---

## 11. Capabilities v1

Official capabilities:

- `providers`
- `commands`
- `decorate-items`
- `input-accessory`
- `keys`

The runtime must deny undeclared operations.

See `MODULES_CAPABILITIES_MATRIX.md`.

---

## 12. Explicit restrictions

Modules cannot:

- draw UI directly,
- access Win32/GDI,
- change global layout,
- mutate arbitrary internal state,
- modify ranking internally,
- intercept the core event loop,
- bypass capabilities,
- depend on core internals.

---

## 13. Errors and isolation

- Module error must not crash the launcher.
- Faulty hooks/providers may be degraded or disabled.
- Logs must include module identity.
- Timeouts, errors, and restarts are exposed through `--modules-debug`.

See `ERROR_ISOLATION_POLICY.md`.

---

## 14. Future compatibility

- API extensions should be additive when possible.
- New primitives require evidence from real modules.
- Breaking changes require a new API and documented decision.
- General policy is defined in `MODULES_ARCHITECTURE.md`.
