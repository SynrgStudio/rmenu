# MODULES QUICKSTART — rmenu

Status: Frozen v1  
Purpose: short guide for installing, developing, and debugging modules.

---

## 1. Module location

By default, `rmenu` discovers modules in:

```text
modules/
```

Supported formats:

```text
modules/
  my-module.rmod
  another-module/
    module.toml
    module.js
    config.json       # optional
    README.md         # optional
```

The module identity is defined by the manifest (`name`), not by the file or folder name.

---

## 2. Install a `.rmod` module

1. Create the folder if needed:

```powershell
mkdir modules
```

2. Copy the file:

```powershell
copy .\my-module.rmod .\modules\my-module.rmod
```

3. Verify loading:

```powershell
rmenu.exe --modules-debug
```

4. If `rmenu` is already open and the module was edited, use:

```text
/modules.reload
```

---

## 3. Install a directory module

Minimum structure:

```text
modules/
  hello-module/
    module.toml
    module.js
```

Minimum `module.toml`:

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

Validate:

```powershell
rmenu.exe --modules-debug
```

---

## 4. Develop a module

Recommended flow:

1. Start with the directory format.
2. Declare minimal capabilities.
3. Implement one small hook.
4. Run `rmenu.exe --modules-debug`.
5. Iterate with hot reload or `/modules.reload`.
6. Check `recent_errors` and telemetry.
7. Package as `.rmod` once stable.

---

## 5. Quick debug

### Show loaded modules

```powershell
rmenu.exe --modules-debug
```

### Reload modules from rmenu

```text
/modules.reload
```

### List modules from rmenu

```text
/modules.list
```

### Reset telemetry

```text
/modules.telemetry.reset
```

---

## 6. Included module: calculator

`rmenu` includes a real validation module:

```text
modules/calculator.rmod
```

Behavior:

- recognizes simple calculations typed directly in the input, for example `2+2`, `10/2`, `(2+3)*4`;
- does not require an `=` prefix;
- shows the result in the input bar, aligned to the right, as `=4`;
- uses `InputAccessory` with kind `success`, so the core decides the color;
- replaces the lower result list with `ctx.replaceItems([])` while the calculation is valid, avoiding irrelevant fuzzy results;
- declares only the `input-accessory` capability.

The core contains no calculator logic. It only exposes generic actions (`setInputAccessory`, `clearInputAccessory`, `replaceItems`) used by the `.rmod`.

---

## 7. Included module: local-scripts v2

`rmenu` includes a second real validation module:

```text
modules/local-scripts.rmod
```

Behavior:

- uses the explicit `>` prefix to enter local script mode;
- `>` lists configured scripts;
- `> bu` filters scripts by name;
- `> build` prioritizes the exact match and marks it with the `exact` badge;
- without the prefix, for example `build`, the global launcher is unchanged;
- replaces the lower result list with `ctx.replaceItems(items)` to avoid competing with History, Start Menu, PATH, and global fuzzy ranking;
- shows contextual count feedback with `InputAccessory`;
- Enter runs the script `target` through the normal launcher path.

Expected example:

```text
> build
```

```text
1  build        .\modules\local-scripts\scripts\build.ps1        [exact]
2  build-prod   .\modules\local-scripts\scripts\build-prod.ps1   [ps1]
```

Included demo scripts:

```text
modules/local-scripts/scripts/build.ps1
modules/local-scripts/scripts/build-prod.ps1
modules/local-scripts/scripts/test.bat
modules/local-scripts/scripts/lint.ps1
modules/local-scripts/scripts/open-logs.cmd
```

This module validates the “scoped intent mode” pattern: an explicit local intent uses `replaceItems` to control only that mode's visible results, without touching global ranking or adding special core boosts.

---

## 8. Included module: shortcuts

`rmenu` includes a third validation module:

```text
modules/shortcuts.rmod
```

Behavior:

- provides exact search aliases for favorite launch targets;
- activates only when `input.trim()` exactly matches a configured `key` or `alias`;
- does not activate for partial or extended input such as `bl`, `b foo`, or `1 foo`;
- replaces the lower result list with the matched shortcut only;
- shows a visual cue badge, for example `[b]`, on the right side of the row;
- Enter runs the shortcut target through the normal launcher path.

Default demo config:

```text
1 or b  -> Blender
2 or t  -> Terminal
```

Expected example:

```text
b
```

```text
Blender    C:\Program Files\Blender Foundation\Blender 5.0\blender-launcher.exe    [b]
```

Non-activating examples that fall back to normal launcher search:

```text
bl
b foo
1 foo
```

This module validates exact alias shortcuts without global hotkeys, ranking boosts, or new core primitives.

### Adding a user shortcut

1. Search for a normal launcher item.
2. Select it.
3. Press `Ctrl+B`.
4. The input changes to:

```text
/shortcuts::bind 
```

5. Type the alias, for example:

```text
/shortcuts::bind bs
```

6. Press Enter.

The shortcut is saved to:

```text
modules/shortcuts.user.json
```

After that, typing `bs` and pressing Enter launches the selected target.

---

## 9. Common errors

### `RMOD_E_INVALID_MAGIC`

The `.rmod` does not start with:

```text
#!rmod/v1
```

### `RMOD_E_MISSING_MODULE_JS`

The required block is missing:

```text
---module.js---
```

### `permission_denied`

The module attempted an operation without declaring the required capability.

Example:

```text
permission_denied module='calc' operation='provide_items' capability='providers'
```

Fix: add the required capability or stop using that operation.

### Host timeout

The module took longer than `provider_timeout_ms` or did not respond.

Fix:

- reduce per-query work,
- cache when appropriate,
- avoid blocking I/O,
- inspect errors with `--modules-debug`.

---

## 10. Quick capability reference

| Goal | Capability |
|---|---|
| Provide items | `providers` |
| Register/receive commands | `commands` |
| Decorate items | `decorate-items` |
| Show status next to input | `input-accessory` |
| Receive key events | `keys` |

Declare only what is needed.

---

## 11. Related contracts

Read in this order:

1. `MODULES_ARCHITECTURE.md`
2. `MODULES_API_SPEC_V1.md`
3. `RMOD_SPEC_V1.md` or `MANIFEST_SPEC_V1.md`
4. `MODULES_CAPABILITIES_MATRIX.md`
5. `MODULES_AUTHORING_GUIDE.md`
