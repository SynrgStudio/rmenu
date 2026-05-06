# Manual validation checklist

Status: manual validation required
Session: `CONT-2026-05-04-1945-ahk-suite-rmenu-migration`

## Non-interactive validation

Run:

```powershell
cargo fmt
cargo check
cargo test
cargo run --bin rmenu -- --modules-debug
```

Expected:

- All Rust checks pass.
- `--modules-debug` lists every active module as `enabled=true` and `status=loaded`.

## Launcher validation

Start rmenu and validate each query returns one clear item:

- `g test query`
- `y test query`
- `w example.com`
- `chatgpt`
- `gpt`
- `ya`
- `ter`
- `terminal`
- `pi`
- `backup`
- `rclone`
- `bk`
- `snip`
- `snipd`
- `rec`
- `record`
- `ocr`
- `text`
- `color`
- `cp`
- a configured `shortcuts.rmod` key/alias

## Launch validation

Validate only safe launches first:

- web URLs open in default browser;
- normal terminal opens;
- Pi opens from configured terminal command;
- rclone command opens in terminal but can be cancelled before destructive sync if config is placeholder/default;
- SnipTool flag commands write to `%TEMP%\\sniptool_cmd.flag` only when intended.

## UAC validation

Requires explicit user approval before testing:

- `ater`
- `aterminal`
- `api`

Expected:

- Windows UAC prompt appears.
- Declining prompt does not crash rmenu.
- Accepting prompt launches elevated target.

## Missing helper validation

With default helper paths absent, these should render warning feedback rather than silently failing:

- `snipd` if `helpers/sniptool/sniptool.py` is absent;
- `color` / `cp` / `picker` if `helpers/color-picker/color-picker.exe` is absent.

Expected:

- rmenu-style toast/input feedback is visible.
- Warning item remains selectable but does not launch a broken helper target.

## Deferred validation

Do not validate in this wave:

- Anytype commands;
- TweetFlow commands;
- daemon/global hooks;
- WindowManager gestures;
- TaskbarVolume;
- TextExpander;
- AlwaysOnTop;
- browser gestures.
