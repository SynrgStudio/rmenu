# directory template (modo desarrollo)

Template base para crear módulos en formato carpeta (ideal para iterar rápido).

## Archivos

- `module.toml` — manifest v1
- `index.js` — lógica del módulo
- `config.json` — configuración editable

## Uso

1. Copiar esta carpeta dentro de `modules/` (ej: `modules/starter-module/`).
2. Editar `module.toml` (`name`, `author`, etc).
3. Editar `config.json` y `index.js`.
4. Recargar módulos desde rmenu.

## Equivalencia con .rmod

Este template es espejo del starter single-file:
- `templates/rmod/starter-module.rmod`

Ambos deben normalizar al mismo `ModuleDescriptor` interno.
