# ITEM_TRAILING_ACCESSORIES_PRIMITIVE — Shortcut Chip + Hint Coexistence

## 1) Problema que resuelve

Cuando un item tiene:
- label (izquierda),
- hint/path (derecha),
- y shortcut visual (ej. `1`),

la UI puede colisionar visualmente o mover labels de forma inconsistente.

Se necesita una primitive que permita agregar accesorios de cola (trailing) sin romper layout.

## 2) Regla de autoridad

- **Módulo declara intención:** `quickSelectKey` / metadata semántica.
- **Core implementa realidad:** layout por zonas, truncado, spacing, colores, input y render.

## 3) API pública propuesta

### Hooks involucrados
- `decorateItems(items, ctx) -> Item[]`

### Context methods
- No requiere métodos nuevos para v1.

### Actions permitidas
- No requiere actions nuevas para v1.

### Tipos
```ts
type Item = {
  id: string
  title: string
  subtitle?: string
  action: Action

  capabilities?: {
    quickSelectKey?: string // "1".."9"|"0"
  }

  decorations?: {
    hint?: string
    badge?: string // opcional, core puede derivar badge desde quickSelectKey
    badgeKind?: "shortcut" | "status" | "tag"
  }
}
```

## 4) Comportamiento y lifecycle

- Si existe `quickSelectKey`, core muestra chip en la cola derecha.
- El hint/path comparte la zona derecha pero sin invadir chip.
- El label mantiene su anclaje izquierdo (no “salta” por módulo).
- Si no hay espacio suficiente:
  1. truncar/ocultar hint,
  2. mantener chip si entra junto con legibilidad mínima de label,
  3. si el ancho es extremo, priorizar label y ocultar chip.

## 5) Render/ejecución en core

### Layout por zonas (derecha a izquierda)

1. **Chip zone** (fijo, right-aligned)
2. **Hint zone** (a la izquierda del chip)
3. **Label zone** (izquierda)

### Reglas

- spacing fijo entre zonas (token de theme).
- chip con ancho mínimo estable.
- hint con truncado por elipsis al inicio o final (definir política única).
- label con truncado estándar del core.

### Input asociado

- Teclas `1..0` las resuelve el core contra `quickSelectKey` visible.
- Política configurable:
  - `quick_select_mode = select`
  - `quick_select_mode = submit`

## 6) Casos de uso

- Shortcut `1` para Blender 5.0 sin mover label visual.
- Items con hint largo + chip numérico.
- Coexistencia de badges de estado y shortcut en roadmap v2.

## 7) Restricciones (no permitido)

- Módulos no controlan posición/píxeles del chip.
- Módulos no dibujan badges/chips.
- Módulos no alteran algoritmo de truncado/layout.

## 8) Criterios de aceptación

- [x] Label no cambia anclaje horizontal por presencia de shortcut.
- [x] Chip de shortcut aparece en zona derecha fija.
- [x] Hint/path se recorta sin colisionar con chip.
- [x] Teclas `1..0` resuelven item con `quickSelectKey`.
- [x] Sin acceso de módulos al renderer Win32/GDI.

## 9) Riesgos y mitigaciones

- **Riesgo:** layout roto en resoluciones chicas.
  - **Mitigación:** límites mínimos por zona + fallback de ocultar hint antes que chip.

- **Riesgo:** conflicto de quick keys duplicadas.
  - **Mitigación:** política core determinista (primer visible/score más alto + warning log).

- **Riesgo:** acoplar API visual a renderer actual.
  - **Mitigación:** mantener primitive semántica (zones/capabilities), no pixeles en API.
