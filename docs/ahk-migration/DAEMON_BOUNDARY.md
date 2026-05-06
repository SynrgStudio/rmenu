# Daemon boundary for AHK migration

Status: future boundary
Session: `CONT-2026-05-04-1945-ahk-suite-rmenu-migration`

## Decision

The current `rmenu` core remains a launcher and module platform. Ordinary `.rmod` modules are short-lived providers/commands/decorators for the launcher UI.

Resident automations from the AHK suite must not be moved into `rmenu.exe` core or ordinary `.rmod` modules.

They belong in a future companion process:

```text
rmenu-daemon.exe
```

or in dedicated helper binaries launched by modules.

## Future daemon responsibilities

A future daemon may own:

- global hotkey to open `rmenu`;
- WindowManager gestures and snapping;
- Thorium/browser mouse gestures;
- taskbar volume wheel/middle-click behavior;
- global TextExpander/hotstrings;
- AlwaysOnTop global hotkey;
- SnipTool global hotkeys;
- daemon status/log/reload commands once the daemon exists.

## RSnip companion boundary

RSnip is a native companion application, not an ordinary helper script. It owns screen capture, recording, OCR, overlays, clipboard integration, and editor behavior.

Integrated behavior when both rMenu and RSnip are installed:

```text
rmenu-daemon.exe
  discovers/coordinates RSnip
  exposes menu commands for snip/record/OCR
  sends direct IPC to RSnip

rsnip.exe daemon
  owns capture/record/OCR internals
  may own global Ctrl+Shift+S/R/E hotkeys unless a documented integrated mode changes ownership
```

The normal rMenu menu path must not be:

```text
rMenu item -> powershell.exe -> rsnip.exe snip -> RSnip daemon
```

The target path is:

```text
rMenu item -> rMenu native companion client -> \\.\pipe\rsnip -> RSnip daemon
```

Standalone behavior remains valid:

```text
rsnip.exe daemon -> Ctrl+Shift+S/R/E -> RSnip action
```

Discovery order for current planning:

1. explicit rMenu config/environment override if implemented;
2. dev path `C:\rSnip\target\release\rsnip.exe`;
3. future packaged install path or registry marker;
4. PATH fallback only when unambiguous.

Lifecycle rule:

- rMenu starts/confirms the RSnip daemon when RSnip is discovered.
- If RSnip was already running, RSnip remains the owner of its daemon and global `Ctrl+Shift+S/R/E` hotkeys; `rmenu-daemon --quit` leaves it running.
- If rMenu started RSnip for integrated mode, rMenu records that ownership for the current process and sends RSnip shutdown on `rmenu-daemon --quit`.
- rMenu does not register RSnip's `Ctrl+Shift+S/R/E` hotkeys in the initial integrated mode; RSnip owns them, so there is no duplicate hotkey registration.
- If a future explicit integrated/no-hotkeys RSnip mode exists, rMenu may become the hotkey owner, but that requires a separate documented decision.
- If RSnip is missing or unreachable, rMenu shows explicit feedback instead of silently launching a broken command.

## Out of core scope

Do not add these to `rmenu.exe` core:

- `Ctrl+LButton` / `Ctrl+RButton` window manipulation;
- global keyboard/mouse hooks;
- taskbar hit-testing loops;
- text-expander keyboard buffering;
- browser-specific mouse gestures;
- snipping/recording overlays;
- color picker overlay rendering;
- rclone progress UI.

## Current wave behavior

This wave only adds:

- launcher modules for Command Center-style commands;
- helper-backed module launchers;
- minimal core support for module discovery, admin launch, and rmenu-style feedback.

## AHK retirement path

Recommended sequence:

1. Replace Command Center command surface with `.rmod` modules.
2. Add helper binaries/scripts for rclone and color picker; treat RSnip as a native companion for snip/record/OCR.
3. Validate launcher/module parity manually.
4. Build or extend `rmenu-daemon.exe` for companion coordination.
5. Move global hotkeys and hooks from AHK to the daemon/companion architecture one family at a time.
6. Keep AHK as rollback until each daemon feature is manually validated.
7. Remove AHK startup only after all required resident behaviors have native equivalents.

## Rollback

Rollback remains simple while AHK is kept installed:

- disable new module/helper entry if it fails;
- keep existing AHK module loaded;
- re-enable AHK CommandCenter/hotkeys as needed;
- use `rmenu --modules-debug` to diagnose module state.
