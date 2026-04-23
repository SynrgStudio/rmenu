# MODULES API SPEC V1

Estado: Draft v1  
Fecha: 2026-04-22

---

## 1) Objetivo

Definir el contrato público mínimo para módulos en `rmenu`.

Regla: **API chica, estable, sin exposición de internals**.

---

## 2) Versionado

- `api_version = 1`
- Todo módulo debe declarar versión de API compatible.
- Cambios breaking requieren bump de versión de API.

---

## 3) Hooks v1

- `onLoad(ctx)`
- `onUnload(ctx)`
- `onQueryChange(query, ctx)`
- `onSelectionChange(item, index, ctx)`
- `onKey(event, ctx)`
- `onSubmit(item, ctx)`
- `onCommand(command, args, ctx)`
- `decorateItems(items, ctx) -> Item[]`

### Reglas

- Hooks deben ser deterministas y rápidos.
- No bloquear loop de UI.
- Errores en hook se aíslan por módulo.

---

## 4) Context (`ctx`) v1

### Lectura

- `ctx.query()`
- `ctx.items()`
- `ctx.selectedItem()`
- `ctx.selectedIndex()`
- `ctx.mode()`
- `ctx.hasCapability(name)`

### Utilidades

- `ctx.log(message)`
- `ctx.toast(message)`

### Mutación permitida (actions)

- `ctx.setQuery(text)`
- `ctx.setSelection(index)`
- `ctx.moveSelection(offset)`
- `ctx.submit()`
- `ctx.close()`
- `ctx.addItems(items)`
- `ctx.replaceItems(items)` (restricto)
- `ctx.registerCommand(def)`
- `ctx.registerProvider(def)`
- `ctx.setInputAccessory(accessory)`
- `ctx.clearInputAccessory()`

---

## 5) Modelo de item v1

```ts
type Item = {
  id: string
  title: string
  subtitle?: string
  source?: string
  action: Action

  capabilities?: {
    quickSelectKey?: string
  }

  decorations?: {
    badge?: string
    badgeKind?: "shortcut" | "status" | "tag"
    hint?: string
    icon?: string
  }
}
```

---

## 6) Input Accessory v1

```ts
type InputAccessory = {
  text: string
  kind?: "info" | "success" | "warning" | "error" | "hint"
  priority?: number
}
```

- Un solo accessory visible a la vez.
- Mayor prioridad gana.
- Render y layout son responsabilidad del core.

---

## 7) Restricciones explícitas

Módulos no pueden:

- dibujar UI directamente,
- acceder a renderer Win32/GDI,
- mutar estado interno arbitrario,
- modificar ranking internamente,
- interceptar event loop del core.

---

## 8) Errores y aislamiento

- Error en módulo no debe tumbar launcher.
- Hook/provider defectuoso puede desactivarse.
- Logs con identificación de módulo obligatorios.

---

## 9) Compatibilidad futura

- Extensiones de API deben ser aditivas cuando sea posible.
- Features nuevas deben pasar por ADR en `DECISIONS.md`.
