import fs from "node:fs";
import path from "node:path";

const DEFAULT_KEYBINDINGS = {
  toggleHint: "ctrl+h",
  nextItem: "ctrl+j",
  prevItem: "ctrl+k",
  submitSelection: "ctrl+enter",
  clearQuery: "ctrl+backspace"
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

function loadModuleConfig(moduleDir, log) {
  const configPath = path.join(moduleDir, "hotkeys.json");

  try {
    const raw = fs.readFileSync(configPath, "utf-8");
    const parsed = JSON.parse(raw);

    return {
      bindings: {
        ...DEFAULT_KEYBINDINGS,
        ...(parsed.bindings || {})
      },
      quickSelectMode: parsed.quickSelectMode === "select" ? "select" : "submit",
      showAccessory: parsed.showAccessory !== false
    };
  } catch (err) {
    log(`[configurable-hotkeys] No se pudo leer hotkeys.json, usando defaults: ${String(err)}`);
    return {
      bindings: { ...DEFAULT_KEYBINDINGS },
      quickSelectMode: "submit",
      showAccessory: true
    };
  }
}

export default function createModule() {
  let config = {
    bindings: { ...DEFAULT_KEYBINDINGS },
    quickSelectMode: "submit",
    showAccessory: true
  };

  return {
    name: "configurable-hotkeys",

    onLoad(ctx) {
      // Asumimos que el runtime expone la ruta del módulo en ctx.moduleDir().
      const moduleDir = typeof ctx.moduleDir === "function" ? ctx.moduleDir() : process.cwd();
      config = loadModuleConfig(moduleDir, (msg) => ctx.log(msg));

      ctx.registerCommand({
        name: "hotkeys.reload",
        description: "Recarga hotkeys.json",
        run: () => {
          config = loadModuleConfig(moduleDir, (msg) => ctx.log(msg));
          ctx.toast("Hotkeys recargadas");
        }
      });

      ctx.registerCommand({
        name: "hotkeys.show",
        description: "Muestra configuración de hotkeys activa",
        run: () => {
          ctx.toast(`Hotkeys: ${JSON.stringify(config.bindings)}`);
        }
      });

      if (config.showAccessory) {
        ctx.setInputAccessory({
          text: `Hotkeys: next(${config.bindings.nextItem}) prev(${config.bindings.prevItem}) submit(${config.bindings.submitSelection})`,
          kind: "hint",
          priority: 5
        });
      }

      ctx.log("[configurable-hotkeys] loaded");
    },

    onUnload(ctx) {
      ctx.clearInputAccessory();
      ctx.log("[configurable-hotkeys] unloaded");
    },

    onKey(event, ctx) {
      const combo = eventToCombo(event);
      const bindings = Object.fromEntries(
        Object.entries(config.bindings).map(([name, key]) => [name, normalizeCombo(key)])
      );

      if (combo === bindings.nextItem) {
        ctx.moveSelection(1);
        return { handled: true };
      }

      if (combo === bindings.prevItem) {
        ctx.moveSelection(-1);
        return { handled: true };
      }

      if (combo === bindings.submitSelection) {
        if (config.quickSelectMode === "select") {
          return { handled: true };
        }
        ctx.submit();
        return { handled: true };
      }

      if (combo === bindings.clearQuery) {
        ctx.setQuery("");
        return { handled: true };
      }

      if (combo === bindings.toggleHint) {
        if (config.showAccessory) {
          config.showAccessory = false;
          ctx.clearInputAccessory();
        } else {
          config.showAccessory = true;
          ctx.setInputAccessory({
            text: "Hint activado desde módulo configurable-hotkeys",
            kind: "info",
            priority: 5
          });
        }
        return { handled: true };
      }

      return { handled: false };
    },

    decorateItems(items) {
      // Ejemplo: asignar quickSelectKey a los 10 primeros items visibles.
      const digits = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"];

      return items.map((item, index) => {
        if (index >= digits.length) {
          return item;
        }

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
