# MODULES OPERATIONS GUIDE v1

Status: Frozen v1

Operations, diagnostics, and recovery guide for the `rmenu` module system.

---

## 1. Runtime commands

- `/modules.reload` — reload external modules.
- `/modules.list` — list loaded modules.
- `/modules.telemetry.reset` — clear host telemetry.

---

## 2. CLI flags

```powershell
rmenu.exe --modules-debug
rmenu.exe --modules-dir .\modules --modules-debug
```

`--modules-debug` prints module status/telemetry and exits.

`--modules-dir <PATH>` overrides module discovery for this run. It is intended for local development, debugging, and portable installs where modules are not in the default location.

Module directory resolution order:

1. `--modules-dir <PATH>` when provided.
2. `RMENU_MODULES_DIR` environment variable when set and non-empty.
3. `%APPDATA%\rmenu\modules` when available.
4. `modules` next to `rmenu.exe`.
5. `modules` relative to the current working directory as a development fallback.

If a higher-priority candidate does not exist, the resolver continues to the next candidate. `--modules-debug` must report the final resolved directory so install and debug sessions are reproducible.

---

## 3. Host states

- `loaded` — host is operational.
- `degraded` — host has recent errors but is still recoverable.
- `disabled` — host is disabled by policy or configuration.
- `unloaded` — descriptor exists with no active host.

---

## 4. Install modules

### `.rmod`

```powershell
mkdir modules
copy .\my-module.rmod .\modules\my-module.rmod
rmenu.exe --modules-debug
```

### Directory

```text
modules/
  my-module/
    module.toml
    module.js
```

Validate:

```powershell
rmenu.exe --modules-debug
```

See also `MODULES_QUICKSTART.md`.

### `/rmods` registry installs

The `/rmods` command is a core-owned installer UI for `.rmod` single-file packages and `rpack` folder modules published in a GitHub registry repository. The registry layout is:

```text
rmenu-rmods/
  modules/
    my-module.rmod
  rpacks/
    my-folder-module/
      module.toml
      module.js
      config.json
      README.md
  registry.json
  scripts/
    generate-registry.*
  .github/
    workflows/
      update-registry.yml
```

`registry.json` is generated from `modules/*.rmod` and `rpacks/*` by GitHub Actions. Maintainers add or update module files and push; they do not edit the registry by hand.

Schema v1 supports `rmod` and `rpack` package kinds and includes module identity, version, description, integrity metadata, tags, and optional rMenu compatibility. See `docs/rmods-registry.md` for the full schema.

Use `/rmods` in the launcher to fetch the registry, mark modules for change with `Space`, refresh with `F5`/`Ctrl+R`, select updates with `Ctrl+U`, and apply marked changes with `Enter`. Markers mean `[x]` installed, `[ ]` not installed, and `[/]` pending change. Installed files land in `<data_dir>\modules`, with supporting cache/state under `<data_dir>\state`.

---

## 5. Operational limits

In `[Modules]` in `config.ini`:

```ini
provider_total_budget_ms = 35
provider_timeout_ms = 1500
max_items_per_provider_host = 24
dedupe_source_priority = core_first
host_restart_backoff_ms = 800
max_ipc_payload_bytes = 262144
```

### `provider_total_budget_ms`

Approximate global per-query budget for collecting external provider responses. Once the budget is exhausted, later provider hosts may be skipped for that query. This protects input latency.

Default: `35`.

### `provider_timeout_ms`

Maximum time to wait for one external host request before treating it as failed. Timeout failures are isolated to the module host and may trigger restart/backoff/disable policy.

Default: `1500`.

### `max_items_per_provider_host`

Maximum provider items accepted from one host before sanitization and merge. Extra items are dropped to bound memory, rendering, and ranking work.

Default: `24`.

### `dedupe_source_priority`

Controls which source wins when core items and provider items resolve to the same launch target.

Values:

- `core_first`: built-in launcher sources win over provider items.
- `provider_first`: provider items win over built-in launcher sources.

Default: `core_first`.

### `host_restart_backoff_ms`

Minimum delay before trying to restart a failed external host again. This avoids restart loops when a module repeatedly fails.

Default: `800`.

### `max_ipc_payload_bytes`

Maximum IPC request/response payload size. Oversized payloads are rejected to protect the launcher process.

Default: `262144`.

Invalid or missing values fall back to safe defaults. Use `rmenu.exe --modules-debug` to inspect the effective policy and host health.

---

## 6. Quick-select policy v1

### `quick_select_mode = select`

Key `1..0` moves selection to the visible item with that `quickSelectKey`.

It does not automatically submit.

### `quick_select_mode = submit`

Key `1..0` selects and immediately submits the visible item.

### Conflicts

If multiple items use the same key:

- the first visible item wins,
- later duplicates lose key/badge,
- a warning is recorded.

---

## 7. Command policy

Namespaced format:

```text
/module::command
```

Rules:

- if an alias belongs to a single module, it can be routed deterministically;
- if an alias belongs to multiple modules, the non-namespaced alias is rejected;
- on collision, use the explicit namespace.

---

## 8. Manual validation: calculator

The `modules/calculator.rmod` module validates the full action flow from external host to core.

Manual test:

1. Run `rmenu`.
2. Type:

```text
2+2
```

Expected result:

- the bar shows `=4` aligned to the right;
- the text uses the `InputAccessoryKind::Success` color;
- the lower list does not show fuzzy results while the calculation is valid;
- no `permission_denied` messages appear for undeclared hooks.

If fuzzy results appear during calculation, check that the module uses `ctx.replaceItems([])` and that the core respects `items_replaced_in_cycle`.

If `[success] =4` appears, check that `input_accessory_text()` renders only `accessory.text`.

---

## 9. Manual validation: local-scripts v2

The `modules/local-scripts.rmod` module validates the explicit local-intent pattern with `ctx.replaceItems(...)`.

Manual test:

1. Run `rmenu` from the project root.
2. Type:

```text
>
```

Expected result:

- the list shows configured local scripts;
- the input accessory shows `local scripts: N`;
- no History, Start Menu, or PATH results appear.

3. Filter:

```text
> bu
```

Expected result:

- `build` and `build-prod` appear;
- items keep subtitle/path and extension badge.

4. Exact match:

```text
> build
```

Expected result:

- `build` is first;
- `build` shows the `exact` badge;
- `build-prod` appears below it.

5. Global launcher:

```text
build
```

Expected result:

- local-scripts mode is not entered;
- normal global ranking is used.

6. Submit:

- select `build` inside `> build`;
- press Enter;
- the script should run through the `target` generated by the module.

If fuzzy/global results appear inside `>`, check that the module calls `ctx.replaceItems(items)` and that the core applies external actions to the real `AppState`.

---

## 10. Manual validation: shortcuts

The `modules/shortcuts.rmod` module validates exact search aliases for favorite launch targets.

Manual test:

1. Run `rmenu` from the project root.
2. Type:

```text
b
```

Expected result:

- the list shows only Blender;
- the row shows the `[b]` visual cue badge;
- the input accessory shows `shortcut: Blender`;
- pressing Enter launches `C:\Program Files\Blender Foundation\Blender 5.0\blender-launcher.exe`.

3. Type:

```text
1
```

Expected result:

- Blender appears as the matched shortcut;
- the row still shows `[b]` as the visual cue;
- pressing Enter launches Blender.

4. Type non-exact inputs:

```text
bl
b foo
1 foo
```

Expected result:

- the shortcuts module does not activate;
- normal global launcher search is used.

If the shortcut activates for partial input, check that the module compares `input.trim()` against `key` and `alias` exactly, not with `startsWith`.

### Add-shortcut validation

1. Search for a normal launcher item, for example Bambu Studio.
2. Select it.
3. Press `Ctrl+B`.

Expected result:

- the input changes to `/shortcuts::bind `;
- the accessory says `binding <title>: type alias and press Enter`.

4. Type the alias after the command:

```text
/shortcuts::bind bs
```

5. Press Enter.
6. Restart or reopen `rmenu` if needed, then type:

```text
bs
```

Expected result:

- the newly bound item appears as the only shortcut result;
- the row shows `[bs]` as visual cue;
- Enter launches the saved target.

User-defined shortcuts are persisted in module state:

```text
<data_dir>\state\modules\shortcuts\shortcuts.user.json
<data_dir>\state\modules\shortcuts\shortcuts.pending.json
```

Older local installs may have used the installed module directory. Move those files to the state directory before updating the `shortcuts` rpack if you need to preserve them.

### Latency validation

With `shortcuts` loaded, typing normal text should remain immediate. Plain text key presses are not dispatched to module key hooks, and hot query snapshots are lightweight. If input feels delayed:

- confirm the release binary was rebuilt with `cargo build --release`;
- check that modules do not perform synchronous disk I/O in `onQueryChange`;
- inspect `--modules-debug` for host errors/timeouts.

---

## 11. Common errors

### `RMOD_E_INVALID_MAGIC`

The `.rmod` file does not start with:

```text
#!rmod/v1
```

### `RMOD_E_MISSING_MODULE_JS`

Missing block:

```text
---module.js---
```

### `permission_denied`

The module did not declare the required capability.

Example:

```text
permission_denied module='x' operation='provide_items' capability='providers'
```

### `module-host timed out`

The host did not respond within the configured timeout.

### `module '<name>' disabled after repeated failures`

The module exceeded the consecutive error/timeout threshold.

---

## 12. Recommended diagnostic flow

1. Run:

```powershell
rmenu.exe --modules-debug
```

2. Check:

- host state,
- `request_count`,
- `error_count`,
- `timeout_count`,
- `restart_count`,
- `recent_errors`.

3. Fix manifest/capabilities/script.

4. Reload:

```text
/modules.reload
```

5. Confirm the state returns to `loaded`.

---

## 13. Recovery

If a module is broken:

1. Disable it in the manifest (`enabled = false`) or move it out of `modules/`.
2. Run `/modules.reload` or restart `rmenu`.
3. Fix errors.
4. Re-enable it.
5. Validate with `--modules-debug`.

---

## 14. Health signals

A healthy system should show:

- hosts in `loaded`,
- few or zero recent errors,
- zero timeouts,
- low restart counts,
- stable latencies,
- external descriptors matching expected modules.
