# AHK CommandCenter to rmenu inventory

Status: migration wave inventory
Session: `CONT-2026-05-04-1945-ahk-suite-rmenu-migration`

## Implemented in this wave

| AHK intent | rmenu module | Query / alias | Notes |
| --- | --- | --- | --- |
| Google search | `web-query.rmod` | `g <query>` | Opens Google search URL. |
| YouTube search | `web-query.rmod` | `y <query>` | Opens YouTube search URL. |
| Open URL | `url-open.rmod` | `w <url>` | Adds `https://` when scheme is omitted. |
| ChatGPT | `chatgpt-open.rmod` | `chatgpt`, `gpt` | Opens ChatGPT. |
| Yandex | `yandex-open.rmod` | `ya` | Opens Yandex. |
| Terminal | `terminal.rmod` | `ter`, `terminal` | Opens Windows Terminal. |
| Admin terminal | `terminal.rmod` | `ater`, `aterminal` | Uses `runas:` target prefix. |
| Pi | `pi-launcher.rmod` | `pi` | Opens Pi in Windows Terminal. |
| Admin Pi | `pi-launcher.rmod` | `api` | Uses `runas:` target prefix. |
| Rclone backup | `rclone-backup.rmod` | `backup`, `rclone`, `bk` | Generic config-backed rclone launcher. |
| Snip screenshot | `snip.rmod` | `snip` | Sends SnipTool `snip` flag to existing daemon. |
| Start SnipTool daemon | `snip.rmod` | `snipd` | Starts configured helper if present. |
| Screen recording | `screen-record.rmod` | `rec`, `record`, `screen` | Sends SnipTool `record` flag. |
| OCR | `ocr.rmod` | `ocr`, `text` | Sends SnipTool `ocr` flag. |
| Color picker | `color-picker.rmod` | `color`, `cp`, `picker` | Helper-backed; missing helper reports inside rmenu. |
| App bindings | existing `shortcuts.rmod` | configured `key`/`alias` | Uses `modules/shortcuts.user.json`; no new shortcuts module. |

## Explicitly not implemented in this wave

| AHK area | Future replacement | Reason |
| --- | --- | --- |
| Anytype capture | split Anytype modules | Deferred by decision. |
| TweetFlow | split TweetFlow modules | Deferred by decision. |
| Global launch hotkey | `rmenu-daemon.exe` | Requires resident process. |
| WindowManager | daemon-backed module/process | Requires global mouse hooks. |
| Thorium/browser gestures | daemon-backed module/process | Requires browser/global mouse hooks. |
| TaskbarVolume | daemon-backed module/process | Requires taskbar hit-testing/global mouse hooks. |
| TextExpander | daemon-backed module/process | Requires global keyboard buffering. |
| AlwaysOnTop | daemon-backed module/process | Requires global hotkey/window manipulation. |
| Snip global hotkeys | daemon-backed module/process | Current wave only provides launcher commands. |

## Core primitives used

- Module directory resolution: `--modules-dir`, `RMENU_MODULES_DIR`, `%APPDATA%\\rmenu\\modules`, exe-adjacent `modules`, cwd `modules`.
- Elevated launch: `target` prefix `runas:`.
- Module feedback: `ctx.toast(message)` rendered as rmenu-style feedback.
