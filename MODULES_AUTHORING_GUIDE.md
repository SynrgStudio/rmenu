# MODULES AUTHORING GUIDE v1

Status: Frozen v1

Official guide for creating `rmenu` modules in `.rmod` and directory formats.

---

## 1. Read first

Recommended order:

1. `MODULES_ARCHITECTURE.md`
2. `MODULES_QUICKSTART.md`
3. `MODULES_API_SPEC_V1.md`
4. `RMOD_SPEC_V1.md` or `MANIFEST_SPEC_V1.md`
5. `MODULES_CAPABILITIES_MATRIX.md`
6. `MODULES_OPERATIONS_GUIDE.md`

Core rule:

> A module declares intent; the core validates, executes, and renders reality.

---

## 2. Choose a format

### `.rmod` — recommended for distribution

- Single file.
- Easy to share.
- Easy to version.
- No auxiliary folder required.

### Directory — recommended for development

- Convenient editing of `module.js`.
- Separate manifest.
- Simple `config.json` and `README.md`.
- Better for hot reload while iterating.

---

## 3. Capabilities

Declare only what is needed.

Capabilities v1:

- `providers`
- `commands`
- `decorate-items`
- `input-accessory`
- `keys`

If a module attempts an operation without the required capability, the runtime blocks it and records `permission_denied`.

---

## 4. Minimum `.rmod` structure

```txt
#!rmod/v1
name: hello-module
version: 0.1.0
api_version: 1
kind: script
capabilities: providers

---module.js---
export default function createModule() {
  return {
    provideItems(query, ctx) {
      if (!query.startsWith("hello")) return [];
      return [{
        id: "hello-item",
        title: "Hello from module",
        subtitle: "Example provider item",
        source: "hello-module",
        target: "notepad.exe",
        badge: "demo"
      }];
    }
  };
}
```

Optional blocks:

- `---config.json---`
- `---readme.md---`

---

## 5. Minimum directory structure

```text
modules/
  hello-module/
    module.toml
    module.js
```

`module.toml`:

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

`module.js` uses the same pattern as the `.rmod` `module.js` block.

---

## 6. Public item model

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

Rules:

- `id` and `title` are required.
- `target` indicates the executable/launchable destination.
- Long or invalid fields may be truncated or discarded.
- The core decides final merge, dedupe, ranking, and rendering.

---

## 7. Main hooks

### Provider

```js
provideItems(query, ctx) {
  return [];
}
```

Requires `providers`.

### Decorator

```js
decorateItems(items, ctx) {
  return items;
}
```

Requires `decorate-items`.

### Command

```js
onCommand(command, args, ctx) {
}
```

Requires `commands`.

### Key hook

```js
onKey(event, ctx) {
}
```

Requires `keys`.

---

## 8. Commands and namespacing

Recommended format:

```text
/module::command
```

Rules:

- aliases without namespace are allowed only when there is no collision;
- if there is a collision, use the explicit namespace;
- empty or ambiguous names are rejected.

---

## 9. Quick select and decorations

`quickSelectKey` must be a simple visible key: `1` to `9` or `0`.

Rules:

- the core resolves conflicts;
- the first visible item wins;
- later duplicates may lose the key/badge;
- `hint` and `badge` may be truncated by layout.

---

## 10. Pattern: scoped intent mode

Use this pattern when a module needs a scoped search that should not compete with the global launcher.

Example implemented by `local-scripts`:

```text
>       # list local scripts
> bu    # filter local scripts
build   # normal global launcher
```

Rules:

- use an explicit intent prefix;
- do not provide local results for ambiguous global searches;
- sort inside the module: exact, prefix, contains, alphabetical;
- call `ctx.replaceItems(items)` to replace the visible list for the scoped mode;
- use `InputAccessory` for short feedback, for example `local scripts: 2`;
- do not use `ProviderDef.priority`, `hint`, `badge`, or magic strings to manipulate global ranking.

Minimum example:

```js
export default function createModule() {
  return {
    onQueryChange(query, ctx) {
      if (!query.startsWith(">")) return;
      const term = query.slice(1).trim().toLowerCase();
      const items = scripts
        .filter((script) => script.name.toLowerCase().includes(term))
        .map((script) => ({
          id: `scripts::${script.name}`,
          title: script.name,
          subtitle: script.path,
          target: script.target,
          badge: script.name.toLowerCase() === term ? "exact" : script.ext
        }));

      ctx.setInputAccessory({ text: `local scripts: ${items.length}`, kind: "info" });
      ctx.replaceItems(items);
    }
  };
}
```

This pattern keeps two UX paths separate: passive global launcher and explicit local mode.

---

## 11. Design rules for authors

1. Hooks must be fast and deterministic.
2. Do not block on I/O without your own timeout.
3. Avoid synchronous disk I/O in `onQueryChange`; cache data in memory and update the cache after writes.
4. Providers should return a small set of relevant items.
5. Do not assume `ctx.items()` is populated on every hot query hook; use key/command flows when selection context is required.
6. Do not assume control of UI or pixels.
7. Normalize internal inputs before processing.
8. Treat errors as recoverable.
9. Log useful context.
10. Do not depend on core internals.
11. Do not declare capabilities “just in case”.
12. Test with `--modules-debug`.

---

## 12. IPC validation and safety

The runtime may trim, normalize, or discard:

- empty required fields,
- strings beyond limits,
- invalid quick-select keys,
- oversized payloads,
- invalid values.

Authors must not depend on unfiltered payloads.

---

## 13. Recommended workflow

1. Create a directory module.
2. Declare minimum metadata.
3. Declare minimum capabilities.
4. Implement one hook.
5. Run `rmenu.exe --modules-debug`.
6. Test behavior in the launcher.
7. Check recent errors.
8. Iterate with `/modules.reload`.
9. Freeze the version.
10. Package as `.rmod` for distribution.
