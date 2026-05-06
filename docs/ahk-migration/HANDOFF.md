# AHK migration handoff

Status: handoff for next wave
Session: `CONT-2026-05-04-1945-ahk-suite-rmenu-migration`

## Completed in this wave

Core primitives:

- robust module directory resolution;
- `--modules-dir <PATH>` and `RMENU_MODULES_DIR`;
- installed/dev module path fallback order;
- `runas:` target prefix for elevated launches;
- external IPC `Toast` action;
- JS `ctx.toast(message)`;
- rmenu-style toast feedback implemented as high-priority input accessory.

Docs:

- minimal core-change decision;
- module directory resolution operations guide;
- daemon boundary;
- shortcuts migration guidance;
- command inventory;
- manual validation checklist;
- helper-backed module authoring guidance.

Modules added:

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

Existing module retained:

- `shortcuts.rmod`

## Validation completed

- `cargo fmt`: OK
- `cargo check`: OK
- `cargo test`: OK, 77 tests passed
- `cargo run --bin rmenu -- --modules-debug`: OK, 14 external descriptors loaded

## Follow-up daemon MVP

Implemented after the initial module wave:

- `rmenu-daemon.exe` binary.
- Default global hotkey: `Ctrl+Shift+Space`.
- Resident-prewarmed launcher mode: daemon loads config, index, modules, and external module hosts once, then shows the launcher UI on hotkey without spawning a fresh `rmenu.exe` each time.
- Configurable `--hotkey`, `--rmenu`, and `--modules-dir` (`--rmenu` retained for startup command compatibility).
- `--install-startup` / `--uninstall-startup` via HKCU Run key.
- `--quit` for stopping the resident daemon.
- Single-instance guard via named mutex.
- Basic logging to `%APPDATA%\\rmenu\\rmenu-daemon.log`.

Validation completed:

- `cargo fmt`: OK
- `cargo check`: OK
- `cargo test`: OK, 82 tests passed
- `cargo run --bin rmenu-daemon -- --quit`: OK

## Persistent data root direction

rMenu should support a persistent data root for modules, companions, config, and state. The default Windows root is `C:\rMenuData`:

```text
<data_dir>\
  modules\
  companions\
    rsnip\
      rsnip.exe
      config\
      state\
      logs\
  config\
  state\
```

The future installer should default to `C:\rMenuData` and may let the user select another folder. If the folder already contains modules, companions, config, or state, rMenu should reuse it without overwriting data.

`--modules-dir` remains useful for development/debugging, but the product model is broader than a modules folder: native companions such as RSnip and future rTask belong under `companions\`.

## Native RSnip companion direction

RSnip should integrate with rMenu as an optional companion app when installed, not as a generic shell-launched `.rmod` command.

Target combined behavior:

- RSnip remains standalone-capable with `rsnip.exe daemon` and its own `Ctrl+Shift+S/R/E` hotkeys when rMenu is absent.
- rMenu daemon discovers and coordinates RSnip when both are installed.
- rMenu menu entries for `snip`, `record`, and `ocr` call RSnip through direct named-pipe IPC (`\\.\pipe\rsnip`).
- Normal menu execution must avoid PowerShell wrappers, visible consoles, and process-launch indirection through `rsnip.exe snip`.
- Hotkey ownership must be explicit so the integrated setup has no duplicate shortcut registration.
- Initial integrated mode keeps `Ctrl+Shift+S/R/E` owned by RSnip. rMenu starts/confirms RSnip, records whether it started the daemon, and only stops RSnip on `rmenu-daemon --quit` if rMenu started it.

See `DECISIONS.md` DEC-007/DEC-008 and `docs/ahk-migration/DAEMON_BOUNDARY.md`.

## Still blocked

T020 manual launcher/module UX validation remains blocked because it requires interactive Windows validation and explicit user approval before UAC prompts. Daemon hotkey validation also requires interactive Windows validation.

Manual validation docs:

- `docs/ahk-migration/MANUAL_VALIDATION.md`

## Deferred backlog

Next-wave candidates:

1. Run manual launcher validation from `MANUAL_VALIDATION.md`.
2. Validate `runas:` aliases with explicit UAC approval.
3. Replace generic rclone defaults with a real safe machine-local backup config or helper.
4. Implement native RSnip companion discovery, lifecycle coordination, and direct IPC menu actions.
5. Provide real color-picker helper or native helper binary.
6. Decide whether to add local per-module config in core.
7. Decide whether to add richer action outcomes (`copy`, `onSubmit`, keep-open behavior, full `ItemAction`).
8. Extend `rmenu-daemon.exe` beyond launcher hotkey only when needed.
9. Migrate daemon-backed AHK features one family at a time:
   - WindowManager;
   - Thorium/browser gestures;
   - TaskbarVolume;
   - TextExpander;
   - AlwaysOnTop;
   - global SnipTool hotkeys.
10. Split Anytype modules when Anytype work resumes.
11. Split TweetFlow modules when TweetFlow work resumes.
12. Retire AHK startup only after daemon/helper parity is manually validated.

## Suggested next session goal

Run manual validation, fix any query/launch UX issues, and decide the first helper/daemon implementation target.
