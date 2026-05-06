# Shortcuts migration

Status: active wave guidance
Session: `CONT-2026-05-04-1945-ahk-suite-rmenu-migration`

## Current rmenu support

`rmenu` already includes `modules/shortcuts.rmod` and `modules/shortcuts.user.json`.

The existing module supports:

- exact `key` matches;
- exact `alias` matches;
- visible badges/hints;
- launching the selected shortcut target through the normal core launcher path;
- `Ctrl+B` binding flow from the currently selected launcher item.

Therefore this migration wave does not add a new shortcuts module.

## AHK AppBindings equivalent

AHK `Config.AppBindings` entries map to `shortcuts.user.json` entries:

```json
{
  "shortcuts": [
    {
      "key": "1",
      "alias": "example",
      "title": "Example App",
      "target": "C:\\Path\\To\\App.exe"
    }
  ]
}
```

Rules:

- `key` preserves numeric bindings such as `1..9`.
- `alias` provides a mnemonic command.
- `title` is the visible label.
- `target` is launched by the existing core launch backend.

## Distribution rule

Do not add private machine paths or secrets to distributed module defaults.

Local/user-specific bindings should live in:

```text
modules/shortcuts.user.json
```

or in a future installed user config location when that exists.

## Validation

Use:

```powershell
rmenu.exe --modules-debug
```

Manual checks:

- Type a numeric binding such as `1`.
- Type an alias.
- Select a normal launcher item and press `Ctrl+B` to bind it.
