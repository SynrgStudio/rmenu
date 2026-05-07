import fs from 'node:fs';
import path from 'node:path';

const PREFIX = 'timer';
const UNIT_SECONDS = new Map([
  ['s', 1], ['sec', 1], ['secs', 1], ['second', 1], ['seconds', 1],
  ['seg', 1], ['segs', 1], ['segundo', 1], ['segundos', 1],
  ['m', 60], ['min', 60], ['mins', 60], ['minute', 60], ['minutes', 60],
  ['minuto', 60], ['minutos', 60],
  ['h', 3600], ['hr', 3600], ['hrs', 3600], ['hour', 3600], ['hours', 3600],
  ['hora', 3600], ['horas', 3600]
]);

function normalize(value) {
  return String(value || '').trim().toLowerCase();
}

function helperPath(ctx) {
  return path.join(ctx.moduleDir(), 'bin', 'timer-helper.ps1');
}

function stateDir(ctx) {
  return ctx.moduleStateDir() || path.join(ctx.moduleDir(), 'state');
}

function quoteArg(value) {
  return `'${String(value).replace(/'/g, "''")}'`;
}

function alarmPath(ctx) {
  return path.join(ctx.moduleDir(), 'sounds', 'alarm.wav');
}

function helperTarget(ctx, command, seconds = 0, label = 'Timer') {
  const script = [
    '&',
    quoteArg(helperPath(ctx)),
    quoteArg(command),
    quoteArg(stateDir(ctx)),
    String(seconds),
    quoteArg(label),
    quoteArg(alarmPath(ctx))
  ].join(' ');
  const encoded = Buffer.from(script, 'utf16le').toString('base64');
  return `hidden:powershell.exe -NoProfile -ExecutionPolicy Bypass -EncodedCommand ${encoded}`;
}

function readState(ctx) {
  try {
    const file = path.join(stateDir(ctx), 'state.json');
    if (!fs.existsSync(file)) return null;
    const content = fs.readFileSync(file, 'utf8').replace(/^\uFEFF/, '');
    const parsed = JSON.parse(content);
    return parsed && typeof parsed === 'object' ? parsed : null;
  } catch (_) {
    return null;
  }
}

function stopAlarmIfRinging(ctx) {
  const state = readState(ctx);
  if (!state || state.state !== 'ringing') return false;
  fs.mkdirSync(stateDir(ctx), { recursive: true });
  fs.writeFileSync(path.join(stateDir(ctx), 'stop.flag'), '', 'utf8');
  return true;
}

function runningRemainingSeconds(ctx) {
  const state = readState(ctx);
  if (!state || state.state !== 'running') return null;
  const deadline = Number(state.deadline_epoch_ms || Date.parse(state.deadline || ''));
  if (!Number.isFinite(deadline)) return null;
  const remaining = Math.ceil((deadline - Date.now()) / 1000);
  return remaining > 0 ? remaining : null;
}

function setRunningAccessory(ctx) {
  const remaining = runningRemainingSeconds(ctx);
  if (!remaining) return false;
  ctx.setInputAccessory({
    text: `timer: ${formatDuration(remaining)} remaining`,
    kind: 'success',
    priority: 100
  });
  return true;
}

function tokenizeDuration(input) {
  return Array.from(input.matchAll(/\d+|[\p{L}]+/gu)).map((match) => match[0].toLowerCase());
}

function parseDuration(input) {
  const tokens = tokenizeDuration(input);
  if (tokens.length === 0) return null;

  if (tokens.every((token) => /^\d+$/.test(token))) {
    const values = tokens.map((token) => Number(token));
    if (values.some((value) => value <= 0 || !Number.isSafeInteger(value))) return null;
    if (values.length === 1) return values[0] * 60;
    if (values.length === 2) return (values[0] * 60) + values[1];
    if (values.length === 3) return (values[0] * 3600) + (values[1] * 60) + values[2];
    return null;
  }

  let seconds = 0;
  for (let index = 0; index < tokens.length; index += 1) {
    const rawValue = tokens[index];
    if (!/^\d+$/.test(rawValue)) return null;
    const value = Number(rawValue);
    const unit = UNIT_SECONDS.get(tokens[index + 1] || '');
    if (!unit || value <= 0 || !Number.isSafeInteger(value)) return null;
    seconds += value * unit;
    index += 1;
  }

  return seconds > 0 ? seconds : null;
}

function formatDuration(totalSeconds) {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  const parts = [];
  if (hours) parts.push(`${hours}h`);
  if (minutes) parts.push(`${minutes}m`);
  if (seconds) parts.push(`${seconds}s`);
  return parts.join(' ') || '0s';
}

function timerItem(ctx, id, label, seconds, subtitle = null) {
  const duration = formatDuration(seconds);
  return {
    id: `timer::${id}`,
    title: `Timer ${label}`,
    subtitle: subtitle || `Start ${duration} timer`,
    source: 'timer',
    target: helperTarget(ctx, 'start', seconds, label),
    badge: duration,
    hint: 'Enter starts timer; opening rMenu stops a ringing alarm'
  };
}

function stopItem(ctx, state) {
  const label = state && typeof state.label === 'string' ? state.label : 'timer';
  return {
    id: 'timer::stop',
    title: state && state.state === 'ringing' ? 'Stop ringing timer' : 'Stop active timer',
    subtitle: label,
    source: 'timer',
    target: helperTarget(ctx, 'stop'),
    badge: 'stop',
    hint: 'Stops the current timer/alarm'
  };
}

function premades(ctx) {
  const config = typeof ctx.moduleConfig === 'function' ? ctx.moduleConfig() : null;
  const configured = config && Array.isArray(config.premades) ? config.premades : [];
  return configured
    .map((entry, index) => ({
      label: String(entry.label || '').trim(),
      seconds: Number(entry.seconds),
      index
    }))
    .filter((entry) => entry.label && Number.isSafeInteger(entry.seconds) && entry.seconds > 0);
}

export default function createModule() {
  return {
    onLoad(ctx) {
      try {
        if (stopAlarmIfRinging(ctx)) {
          ctx.setInputAccessory({
            text: 'timer: alarm stopped',
            kind: 'success',
            priority: 95
          });
          return;
        }
        setRunningAccessory(ctx);
      } catch (_) {
        ctx.setInputAccessory({
          text: 'timer: could not stop alarm',
          kind: 'warning',
          priority: 95
        });
      }
    },

    provideItems(query, ctx) {
      const input = String(query || '').trim();
      const lower = normalize(input);
      if (lower !== PREFIX && !lower.startsWith(`${PREFIX} `)) return [];

      const rest = input.slice(PREFIX.length).trim();
      const state = readState(ctx);
      const items = [];
      if (state && (state.state === 'running' || state.state === 'ringing')) {
        items.push(stopItem(ctx, state));
      }

      if (!rest) {
        return items.concat(premades(ctx).map((entry) => timerItem(
          ctx,
          `premade-${entry.index}`,
          entry.label,
          entry.seconds
        )));
      }

      if (normalize(rest) === 'stop') {
        return [stopItem(ctx, state)];
      }

      const seconds = parseDuration(rest);
      if (!seconds) return items;
      const duration = formatDuration(seconds);
      return [timerItem(ctx, `custom-${seconds}`, duration, seconds, `Start custom timer: ${duration}`)].concat(items);
    },

    onQueryChange(query, ctx) {
      const input = String(query || '').trim();
      const lower = normalize(input);
      if (!input) {
        if (!setRunningAccessory(ctx)) ctx.clearInputAccessory();
        return;
      }
      if (lower !== PREFIX && !lower.startsWith(`${PREFIX} `)) {
        return;
      }

      const rest = input.slice(PREFIX.length).trim();
      if (!rest) {
        ctx.setInputAccessory({ text: 'timer: choose a premade', kind: 'hint', priority: 80 });
        return;
      }
      if (normalize(rest) === 'stop') {
        ctx.setInputAccessory({ text: 'timer: stop active timer', kind: 'warning', priority: 80 });
        return;
      }

      const seconds = parseDuration(rest);
      ctx.setInputAccessory(seconds
        ? { text: `timer: ${formatDuration(seconds)}`, kind: 'success', priority: 80 }
        : { text: 'timer: invalid duration', kind: 'warning', priority: 80 });
    }
  };
}
