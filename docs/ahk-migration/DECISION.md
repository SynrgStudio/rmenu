# AHK suite migration decision

Status: accepted for current continuity wave
Date: 2026-05-05
Session: `CONT-2026-05-04-1945-ahk-suite-rmenu-migration`

## Context

The current AHK suite in `C:\tuicommandcenter\AHKFolder` includes a Command Center, web/search commands, terminal/PI launchers, helper-backed tools, Anytype capture, TweetFlow, and global hotkey/mouse-hook automations.

`rmenu` core v1 is frozen as a launcher and module platform. New functionality must be implemented as modules first unless a general platform primitive is required by multiple modules.

## Decision

This wave keeps the core minimal and implements only general platform support required by the active module set.

Approved core changes for this wave:

1. **Robust module directory resolution**
   - Add a deterministic resolution path for development and installed usage.
   - This is a platform/diagnostics improvement, not a product feature.

2. **Minimal admin launch support (`runas`)**
   - Add support for elevated launching through Windows `ShellExecuteW` with verb `runas`.
   - This is a general launch primitive needed by multiple modules (`terminal.rmod`, `pi-launcher.rmod`).

3. **rmenu-style toast feedback / `ctx.toast`**
   - Add lightweight visual feedback using `rmenu`'s own visual language, not native Windows notifications.
   - This is a general feedback primitive for helper-backed modules and error reporting.

Active modules for this wave:

- `web-query.rmod`
- `url-open.rmod`
- `chatgpt-open.rmod`
- `yandex-open.rmod`
- `terminal.rmod`
- `pi-launcher.rmod`
- `rclone-backup.rmod`
- `snip.rmod`
- `screen-record.rmod`
- `ocr.rmod`
- `color-picker.rmod`
- existing `shortcuts.rmod` review/guidance only

Deferred core work:

- local per-module config loading in core;
- full `ItemAction` model;
- `onSubmit` implementation;
- `copy` action;
- keep-open/submit outcome control;
- broad JS bridge parity;
- daemon implementation.

Deferred modules/features:

- Anytype split modules;
- TweetFlow modules;
- daemon-backed modules for global hooks and resident automations.

## Boundaries

The following do not belong in `rmenu` core or ordinary `.rmod` modules in this wave:

- WindowManager gestures;
- global hotkeys;
- global TextExpander;
- Thorium mouse gestures;
- taskbar volume control;
- AlwaysOnTop hotkey;
- SnipTool global hotkeys.

Those belong in a future `rmenu-daemon.exe` or helper binaries.

## Minimal primitive details

### Module directory resolution

Resolution order:

1. `--modules-dir <PATH>` when the path exists.
2. `RMENU_MODULES_DIR` when set, non-empty, and existing.
3. `%APPDATA%\rmenu\modules` when existing.
4. `modules` next to `rmenu.exe` when existing.
5. cwd `modules` as the final fallback.

`--modules-debug` must print the resolved path.

### Minimal `runas`

This wave does not introduce a full item action model. Admin launch is represented as a reserved launch-target prefix:

```text
runas:<target-and-args>
```

Examples:

```text
runas:wt.exe
runas:wt.exe powershell -NoExit -Command pi
```

The core strips `runas:`, parses the remaining target the same way as normal launch targets, and calls `ShellExecuteW` with verb `runas`. Normal `target` behavior remains unchanged.

This minimal representation is intentionally narrow and can later be replaced or complemented by a full item action model.

### rmenu-style toast / `ctx.toast`

`ctx.toast(message)` is a request for short user feedback rendered with `rmenu`'s visual language, not a native Windows notification.

Minimum behavior for this wave:

- external JS `ctx.toast(message)` sends a toast request to the runtime;
- toast text is bounded/sanitized;
- the launcher must not crash if toast rendering fails;
- visual styling should reuse current `rmenu` colors/font/border where possible;
- a simple implementation may initially render toast feedback through the existing input accessory area if a separate popup would be too large for this wave.

## Consequences

- The first implementation wave remains small and testable.
- Current modules can mostly use existing `target` launch behavior.
- Admin launch and helper feedback get first-class platform support.
- Helper-backed modules may use internal `.rmod` `config.json` and relative helper paths for now.
- Future Anytype/TweetFlow work can add stronger config/secrets and submit primitives when needed.
