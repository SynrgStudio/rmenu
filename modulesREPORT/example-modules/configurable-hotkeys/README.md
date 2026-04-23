# configurable-hotkeys (ejemplo realista)

Módulo de ejemplo para `rmenu` que muestra cómo tener **hotkeys configurables** sin hardcodear combinaciones en el flujo principal.

## Archivos

- `module.toml` — manifest v1.
- `index.js` — implementación del módulo.
- `hotkeys.json` — configuración editable por usuario.

## Qué hace

1. Lee `hotkeys.json` al cargar.
2. Registra comandos:
   - `hotkeys.reload`
   - `hotkeys.show`
3. Maneja `onKey` con bindings configurables:
   - siguiente/anterior item
   - submit
   - limpiar query
   - toggle de accessory
4. En `decorateItems`, agrega `quickSelectKey` (`1..0`) a los primeros 10 items.

## Ejemplo de uso

- Editar `hotkeys.json`.
- Ejecutar comando `hotkeys.reload`.
- Probar las combinaciones nuevas sin reiniciar launcher.

## Nota

Es un ejemplo "real" para la API v1 definida en `modulesREPORT/*`.
Puede requerir adaptar detalles menores del shape exacto de `onKey(event)` cuando el runtime final esté cerrado.
