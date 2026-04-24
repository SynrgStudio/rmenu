# RMOD SPEC V1

Status: Frozen v1  
Date: 2026-04-24

---

## 1. Objective

Define the single-file `.rmod` format for `rmenu` modules.

Main rule:

> `.rmod` v1 is plain UTF-8 text.

If a packed/binary format exists in the future, it must use a different extension.

---

## 2. File shape

A `.rmod` file contains:

1. magic line,
2. header key-value lines,
3. blank line,
4. named content blocks.

Line endings may be `\n` or `\r\n`.

Required first line:

```text
#!rmod/v1
```

---

## 3. Header

Required fields:

```text
name: my-module
version: 0.1.0
api_version: 1
kind: script
capabilities: providers,commands
```

Optional fields:

```text
enabled: true
priority: 0
description: Human readable description
author: Name
homepage: https://example.com
```

Rules:

- `name` must be non-empty and unique among loaded modules.
- `version` is an opaque module version string.
- `api_version` must be numeric and supported by the runtime.
- `kind = script` is the only official v1 kind.
- `capabilities` is a comma-separated list.
- Unknown header fields may be ignored by v1 loaders.

---

## 4. Blocks

A block starts with:

```text
---block-name---
```

Required block:

```text
---module.js---
```

Optional blocks:

```text
---config.json---
---readme.md---
```

Rules:

- duplicate blocks are invalid,
- `module.js` must export a default factory function,
- `config.json`, when present, must be valid JSON,
- unknown blocks may be ignored by v1 loaders.

---

## 5. JavaScript entry

`module.js` must export a default function:

```js
export default function createModule() {
  return {
    onLoad(ctx) {},
    onQueryChange(query, ctx) {},
    provideItems(query, ctx) { return []; }
  };
}
```

The returned object may implement any public hook from `MODULES_API_SPEC_V1.md`.

---

## 6. Config

`config.json` is passed to the module host as module configuration.

The core does not assign semantics to config fields. Each module owns its config schema.

---

## 7. Error codes

Recommended minimum errors:

| Code | Meaning |
|---|---|
| `RMOD_E_INVALID_MAGIC` | Missing or invalid `#!rmod/v1` magic line |
| `RMOD_E_HEADER_MALFORMED` | Header line is malformed |
| `RMOD_E_MISSING_REQUIRED_HEADER` | Required header is missing |
| `RMOD_E_INVALID_API_VERSION` | `api_version` is invalid or unsupported |
| `RMOD_E_DUPLICATE_BLOCK` | Block appears more than once |
| `RMOD_E_MISSING_MODULE_JS` | Required `module.js` block is missing |
| `RMOD_E_CONFIG_NOT_JSON` | `config.json` is not valid JSON |

The loader may add human context to the error, but the stable code should remain unchanged for debugging and tests.

---

## 8. Minimum valid example

```text
#!rmod/v1
name: hello
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
        id: "hello",
        title: "Hello",
        target: "notepad.exe"
      }];
    }
  };
}
```

---

## 9. Example with config and readme

```text
#!rmod/v1
name: local-scripts
version: 0.1.0
api_version: 1
kind: script
capabilities: input-accessory
priority: 10
description: Lists local scripts behind an explicit prefix.

---module.js---
export default function createModule() {
  return {
    onQueryChange(query, ctx) {
      if (!query.startsWith(">")) return;
      ctx.replaceItems([]);
    }
  };
}

---config.json---
{ "prefix": ">" }

---readme.md---
# local-scripts

Type `>` to enter local script mode.
```
