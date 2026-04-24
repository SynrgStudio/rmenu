# MANIFEST SPEC V1

Status: Frozen v1  
Date: 2026-04-24

---

## 1. Objective

Define the minimum declarative format for modules distributed as directories.

The manifest declares identity, version, API, entrypoint, and capabilities. Real enforcement always happens in the core runtime.

---

## 2. Location

Default location:

```text
modules/<module-name>/module.toml
```

Rules:

- one module per directory,
- `module.toml` at the directory root,
- entrypoint path is relative to the manifest directory.

---

## 3. Minimum example

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

## 4. Fields

- `name` — non-empty string, unique among loaded modules.
- `version` — non-empty string.
- `api_version` — numeric public module API version.
- `kind` — module implementation kind. v1 supports `script`.
- `entry` — path to the JS entry file, relative to the module directory.
- `capabilities` — list of requested capabilities.
- `enabled` — optional boolean, default `true`.
- `priority` — optional integer for deterministic discovery/execution order.
- `description` — optional human-readable text.
- `author` — optional author metadata.
- `homepage` — optional URL or project reference.

---

## 5. Capabilities

Official v1 capabilities:

```toml
capabilities = [
  "providers",
  "commands",
  "decorate-items",
  "input-accessory",
  "keys"
]
```

Rules:

- modules should declare only capabilities they use,
- unknown capabilities may warn or be rejected depending on loader policy,
- lacking a capability causes sensitive operations to be denied at runtime.

---

## 6. Entrypoint

The entrypoint must export a default module factory:

```js
export default function createModule() {
  return {
    onLoad(ctx) {}
  };
}
```

---

## 7. Minimum validation

The loader must validate at least:

1. `name` present and non-empty.
2. `version` present and non-empty.
3. `api_version` present, numeric, and supported.
4. `kind` supported.
5. `entry` present and readable.
6. `capabilities` present and parseable.
7. module names are unique.
8. path traversal is rejected for entry files.

---

## 8. Hot reload

When a directory module changes:

1. Runtime detects descriptor or entry change.
2. Runtime unloads the affected module.
3. Runtime reloads descriptor and entry.
4. Runtime restarts the external host when needed.
5. Runtime resets relevant counters on successful reload.
6. On failure, the module becomes degraded/disabled and the error is recorded.

No complex state migration exists in v1.

---

## 9. Security model

- The manifest does not execute code by itself.
- The manifest declares intent/capabilities.
- Runtime enforces capabilities.
- External host isolates module execution.
- IPC payloads are validated by the core.

---

## 10. Compatibility

- `api_version = 1` is required for v1 modules.
- `kind = "script"` is the only official v1 kind.
- Additive optional fields may be ignored by older loaders.
- Incompatible changes require a new API version or spec.
