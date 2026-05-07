# timer

Short timers for rMenu.

## Usage

- `timer` shows premades.
- `timer 10s`, `timer 10 seconds`, `timer 10 segundos`.
- `timer 10m`, `timer 10 minutes`, `timer 10 minutos`.
- `timer 1h`, `timer 1 hour`, `timer 1 hora`.
- `timer 10 10` starts 10 minutes and 10 seconds.
- `timer 1 10 20` starts 1 hour, 10 minutes, and 20 seconds.
- `timer stop` stops the active timer/alarm.

When the timer finishes, the helper loops `sounds/alarm.wav`. Opening rMenu while the alarm is ringing stops it automatically. If the WAV is missing or cannot be played, the helper falls back to simple beeps.
