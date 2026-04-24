import fs from "node:fs";
import path from "node:path";

const DEFAULT_CONFIG = {
  bindings: {
    nextItem: "ctrl+j",
    prevItem: "ctrl+k",
    submit: "ctrl+enter",
    clearQuery: "ctrl+backspace",
    toggleAccessory: "ctrl+h"
  },
  quickSelectMode: "submit",
  showAccessoryOnLoad: true,
  accessoryText: "Starter module activo"
};

function normalizeCombo(input) {
  return String(input || "")
    .toLowerCase()
    .split("+")
    .map((part) => part.trim())
    .filter(Boolean)
    .sort()
    .join("+");
}

function eventToCombo(event) {
  const parts = [];
  if (event.ctrl) parts.push("ctrl");
  if (event.alt) parts.push("alt");
  if (event.shift) parts.push("shift");
  if (event.meta) parts.push("meta");
  parts.push(String(event.key || "").toLowerCase());
  return normalizeCombo(parts.join("+"));
}

function safeConfig(rawConfig) {
  const cfg = rawConfig && typeof rawConfig === "object" ? rawConfig : {};
  const bindings = cfg.bindings && typeof cfg.bindings === "object" ? cfg.bindings : {};

  return {
    bindings: {
      ...DEFAULT_CONFIG.bindings,
      ...bindings
    },
    quickSelectMode: cfg.quickSelectMode === "select" ? "select" : "submit",
    showAccessoryOnLoad: cfg.showAccessoryOnLoad !== false,
    accessoryText:
      typeof cfg.accessoryText === "string" && cfg.accessoryText.trim().length > 0
        ? cfg.accessoryText
        : DEFAULT_CONFIG.accessoryText
  };
}

function loadConfigFromDisk(moduleDir, log) {
  const configPath = path.join(moduleDir, "config.json");
  try {
    const raw = fs.readFileSync(configPath, "utf-8");
    return safeConfig(JSON.parse(raw));
  } catch (error) {
    log(`[starter-module] config.json inválido o ausente, usando defaults: ${String(error)}`);
    return { ...DEFAULT_CONFIG };
  }
}

function normalizedBindings(config) {
  return Object.fromEntries(
    Object.entries(config.bindings).map(([name, combo]) => [name, normalizeCombo(combo)])
  );
}

export default function createModule() {
  let config = { ...DEFAULT_CONFIG };
  let moduleDir = process.cwd();

  function readConfig(ctx) {
    moduleDir = typeof ctx.moduleDir === "function" ? ctx.moduleDir() : moduleDir;
    config = loadConfigFromDisk(moduleDir, (msg) => ctx.log(msg));
  }

  function setAccessory(ctx, text, kind = "hint") {
    ctx.setInputAccessory({ text, kind, priority: 5 });
  }

  return {
    name: "starter-module",

    onLoad(ctx) {
      readConfig(ctx);

      ctx.registerCommand({
        name: "starter.reload-config",
        description: "Recarga config.json",
        run: () => {
          readConfig(ctx);
          ctx.toast("starter-module: config recargada");
        }
      });

      if (config.showAccessoryOnLoad) {
        setAccessory(ctx, config.accessoryText, "info");
      }

      ctx.log("[starter-module] loaded");
    },

    onUnload(ctx) {
      ctx.clearInputAccessory();
      ctx.log("[starter-module] unloaded");
    },

    onKey(event, ctx) {
      const combo = eventToCombo(event);
      const keys = normalizedBindings(config);

      if (combo === keys.nextItem) {
        ctx.moveSelection(1);
        return { handled: true };
      }
      if (combo === keys.prevItem) {
        ctx.moveSelection(-1);
        return { handled: true };
      }
      if (combo === keys.submit) {
        if (config.quickSelectMode === "submit") {
          ctx.submit();
        }
        return { handled: true };
      }
      if (combo === keys.clearQuery) {
        ctx.setQuery("");
        return { handled: true };
      }
      if (combo === keys.toggleAccessory) {
        setAccessory(ctx, "Starter accessory toggleado", "hint");
        return { handled: true };
      }

      return { handled: false };
    },

    decorateItems(items) {
      const digits = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"];
      return items.map((item, index) => {
        if (index >= digits.length) return item;
        return {
          ...item,
          capabilities: {
            ...(item.capabilities || {}),
            quickSelectKey: digits[index]
          },
          decorations: {
            ...(item.decorations || {}),
            badge: digits[index],
            badgeKind: "shortcut"
          }
        };
      });
    }
  };
}
