# rmod template (starter downloadable)

Este template está pensado para subirlo tal cual a la web para descarga directa.

## Archivo principal

- `starter-module.rmod`

## Uso esperado

1. Copiar el archivo a `modules/`.
2. Renombrarlo si quieres (ej. `my-module.rmod`).
3. Editar header + `config.json` + `module.js`.
4. Ejecutar reload de módulos en rmenu.

## Qué incluye

- Header v1 completo
- `config.json` con hotkeys configurables
- `module.js` funcional con:
  - `onLoad` / `onUnload`
  - `onKey` configurable (sin combinaciones hardcodeadas en lógica)
  - comando `/starter.reload-config`
  - `decorateItems` asignando `quickSelectKey` para 1..0

## Notas

- Está alineado con `modulesREPORT/RMOD_SPEC_V1.md`.
- Si el runtime final cambia algún detalle del evento de teclado, ajusta `eventToCombo(event)`.
