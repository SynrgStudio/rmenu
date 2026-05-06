# rMods Registry

Status: rmod + rpack implemented

This document defines the repository layout and generated registry schema for the core-owned `/rmods` command.

## Goal

`/rmods` lets rMenu fetch a GitHub-hosted module catalog, show available modules, compare them against local installs, and install/update/remove selected modules under the rMenu data root.

The registry is generated, not edited manually:

1. Add or update a module in the registry repo.
2. Push to GitHub.
3. GitHub Actions regenerates `registry.json`.
4. Run `/rmods` in rMenu and refresh with `R`.

## Package kinds

### `rmod`

Single-file text module conforming to `RMOD_SPEC_V1.md`.

Repository source:

```text
modules/<id>.rmod
```

Install target:

```text
<data_dir>\modules\<id>.rmod
```

### `rpack`

Folder module conforming to `MANIFEST_SPEC_V1.md`. Use this when the module needs multiple files, helpers, scripts, assets, or a cleaner config/readme split.

Repository source:

```text
rpacks/<id>/
  module.toml
  module.js
  config.json
  README.md
  helpers/
  scripts/
  assets/
```

Install target:

```text
<data_dir>\modules\<id>\
  module.toml
  module.js
  ...
```

`rpack` is a folder distribution format, not a zip/archive format.

## Repository layout

```text
rmods/
  modules/
    calculator.rmod
    web-query.rmod
  rpacks/
    shortcuts/
      module.toml
      module.js
      config.json
      README.md
  registry.json
  scripts/
    generate-registry.py
  .github/
    workflows/
      update-registry.yml
```

## Registry URL

Default registry URL:

```text
https://raw.githubusercontent.com/SynrgStudio/rmods/main/registry.json
```

## Schema v1

Top-level shape:

```json
{
  "schema": 1,
  "generated_at": "2026-05-06T00:00:00Z",
  "modules": []
}
```

### `rmod` record

```json
{
  "id": "web-query",
  "name": "web-query",
  "version": "0.1.0",
  "description": "Explicit Google and YouTube query launcher.",
  "kind": "rmod",
  "download_url": "https://raw.githubusercontent.com/SynrgStudio/rmods/main/modules/web-query.rmod",
  "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "size": 1234,
  "tags": []
}
```

### `rpack` record

```json
{
  "id": "shortcuts",
  "name": "shortcuts",
  "version": "0.3.0",
  "description": "Exact search aliases for launching favorite targets.",
  "kind": "rpack",
  "base_url": "https://raw.githubusercontent.com/SynrgStudio/rmods/main/rpacks/shortcuts",
  "sha256": "aggregate-sha256-over-file-list",
  "size": 6523,
  "files": [
    {
      "path": "module.toml",
      "sha256": "file-sha256",
      "size": 237
    },
    {
      "path": "module.js",
      "sha256": "file-sha256",
      "size": 5960
    }
  ],
  "tags": []
}
```

### Field rules

| Field | Applies to | Required | Description |
|---|---|---:|---|
| `id` | all | yes | Stable module/install identifier. Must match `.rmod` header `name` or `module.toml` `name`. |
| `name` | all | yes | Display name. |
| `version` | all | yes | Opaque module version string. |
| `description` | all | no | Human-readable summary. |
| `kind` | all | yes | `rmod` or `rpack`. |
| `download_url` | `rmod` | yes | Raw URL for the `.rmod` file. |
| `base_url` | `rpack` | yes | Raw URL for the rpack folder. File paths append to this URL. |
| `sha256` | all | yes | For `rmod`, SHA-256 of the file. For `rpack`, aggregate SHA-256 over sorted file path/hash/size entries. |
| `size` | all | yes | For `rmod`, file size. For `rpack`, total file size. |
| `files` | `rpack` | yes | Relative file list with per-file SHA-256 and size. |
| `tags` | all | no | Search/filter hints. |
| `requires_rmenu` | all | no | Compatibility requirement string. |

## Validation policy

The generator and rMenu core both validate registry data. Minimum checks:

- supported `schema`;
- safe module IDs;
- unique module IDs;
- supported package kind: `rmod` or `rpack`;
- valid URLs;
- valid lowercase SHA-256 strings;
- positive sizes within limits;
- valid `.rmod` headers/blocks for `rmod`;
- valid `module.toml` and entry file for `rpack`;
- safe `rpack` file paths only:
  - no absolute paths;
  - no `..`;
  - no empty path segments;
  - no path traversal.

## `/rmods` UI semantics

Markers:

```text
[x] installed
[ ] not installed
[/] pending change
```

Controls:

```text
/rmods       open registry list
/rmods ca    filter registry list
Up/Down      move cursor
Space        mark/unmark pending change
F5/Ctrl+R    refresh registry
Ctrl+U       mark update-available modules
Enter        apply pending installs/updates/removals
Esc          close rMenu
```

## State and cache

```text
<data_dir>\state\rmods-installed.json
<data_dir>\state\rmods-registry-cache.json
<data_dir>\state\downloads\
<data_dir>\state\modules\<module-name>\
```

Installed package contents live under `<data_dir>\modules`. User-created module data should live under `<data_dir>\state\modules\<module-name>` via `ctx.moduleStateDir()`, so rpack updates can replace package files without deleting user data.

## Resident helper rpacks

An `rpack` may declare a resident helper in `module.toml` with `[resident]`. The registry stores the rpack files normally; `rmenu-daemon` interprets the manifest after install and manages the helper lifecycle.

Operational expectations:

- helper paths are relative to the rpack folder;
- helper files are included in the `files` integrity list;
- helper startup failures are logged but do not prevent rMenu from opening;
- install/update/uninstall should be followed by daemon helper sync;
- resident helpers using global hooks should document their behavior and security implications in the rpack README.

Current resident-helper rpacks in the SynrgStudio registry include `taskbar-volume` and `thorium-tabs`.

## Deferred work

- multiple registries;
- registry signing;
- dependency resolution between modules;
- detailed module info screen;
- confirmation prompts for destructive removals.
