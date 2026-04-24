# rmenu — Plan de implementación de arquitectura modular/extensiones

Fecha: 2026-04-22
Estado: diseño consolidado (no implementado en core aún)

---

## 1) Objetivo

Implementar una plataforma de módulos para `rmenu` donde:

- el core siga pequeño, estable y determinista,
- las funcionalidades opcionales vivan fuera del núcleo,
- las extensiones se integren por contrato público (hooks/context/actions/providers),
- la UI pueda enriquecerse sin exponer render internals.

Regla rectora:

**Módulos declaran intención. Core aplica estado/render/input.**

---

## 2) Principios no negociables

1. **Autoridad del core**
   - input, query, ranking, selección, submit, render, lifecycle.

2. **Sin acceso arbitrario a internals**
   - módulos no acceden a estado mutable interno, renderer Win32/GDI, cache interna ni event loop.

3. **Extensión por composición, no por mutación**
   - providers + hooks + metadata + actions permitidas.

4. **Consistencia visual controlada por core**
   - módulos no dibujan; solo aportan metadata.

5. **API pública chica y versionada**
   - mínima superficie, estable y auditada.

---

## 3) Superficie pública v1 (target)

## 3.1 Hooks

- `onLoad(ctx)`
- `onUnload(ctx)`
- `onQueryChange(query, ctx)`
- `onSelectionChange(item, index, ctx)`
- `onKey(event, ctx)`
- `onSubmit(item, ctx)`
- `onCommand(command, args, ctx)`
- `decorateItems(items, ctx) -> Item[]`

## 3.2 Context (`ctx`) lectura/utilidades

- `ctx.query()`
- `ctx.items()`
- `ctx.selectedItem()`
- `ctx.selectedIndex()`
- `ctx.mode()`
- `ctx.hasCapability(name)`
- `ctx.log(message)`
- `ctx.toast(message)`

## 3.3 Actions permitidas

- `ctx.setQuery(text)`
- `ctx.setSelection(index)`
- `ctx.moveSelection(offset)`
- `ctx.submit()`
- `ctx.close()`
- `ctx.addItems(items)`
- `ctx.replaceItems(items)` (contexto controlado)
- `ctx.registerCommand(def)`
- `ctx.registerProvider(def)`

## 3.4 Modelo de item (canónico)

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

## 4) Providers, comandos y pipeline

## 4.1 Providers (mecanismo preferido de contenido)

Los módulos agregan contenido por `registerProvider`, no mutando listas internas arbitrariamente.

## 4.2 Comandos

Soporte explícito a comandos modulares (`/scripts`, `/projects`, etc.) vía `registerCommand` + `onCommand`.

## 4.3 Pipeline de ejecución recomendado

1. fuentes core (history/start/path)
2. providers de módulos
3. merge + dedupe
4. ranking core
5. `decorateItems`
6. render core
7. input/submit core

---

## 5) Primitivas UI extensibles

Las primitivas UI son slots semánticos controlados por core.

## 5.1 Primitive A — Item decorations/capabilities

Estado: definido conceptualmente.

Uso: badges/tags/hints/quick shortcut por item.

Regla: módulo setea metadata; core renderiza y aplica input.

## 5.2 Primitive B — Input Accessory / Inline Query Preview

Fuente: `ui_extension_primitive.md`.

### API

- `ctx.setInputAccessory(accessory)`
- `ctx.clearInputAccessory()`

```ts
type InputAccessory = {
  text: string
  kind?: "info" | "success" | "warning" | "error" | "hint"
  priority?: number
}
```

### Comportamiento

- Ephemeral y query-dependent.
- Se actualiza en `onQueryChange`.
- Se limpia en invalidez, clear explícito, cierre, o override por prioridad mayor.
- Solo un accessory visible a la vez (v1).

### Responsabilidad core

- alineado y layout,
- spacing/truncado/clipping,
- color por `kind`,
- integración con theme,
- política de prioridad.

### No permitido para módulos

- dibujar en pantalla,
- tocar layout del input,
- callbacks de render,
- acceso a GDI/renderer.

---

## 6) Arquitectura objetivo en código (propuesta)

Nueva carpeta `src/modules/`:

- `mod.rs` → orquestación/runtime manager
- `types.rs` → contrato público (Item, Action, Accessory, etc.)
- `hooks.rs` → dispatch de hooks
- `context.rs` → `ctx` controlado
- `actions.rs` → aplicación validada de actions
- `providers.rs` → registro/ejecución providers
- `commands.rs` → registro/ruteo comandos
- `manifest.rs` → parse/validación `module.toml`
- `loader.rs` → descubrimiento/carga/hot reload simple
- `permissions.rs` → capabilities/permiso por módulo

Integración con UI/core:

- `src/ui_win32.rs` consume metadata renderizable y `input accessory` desde estado core.
- `src/ranking.rs` opera sobre modelo canónico sin exponer internals a módulos.

---

## 7) Roadmap por fases

## Fase M0 — Fundaciones de contrato

- Tipos canónicos públicos (`Item`, `Action`, `Capabilities`, `Decorations`, `InputAccessory`).
- `ModuleCtx` de solo lectura + utilidades.
- Actions internas con validación de invariantes.

## Fase M1 — Runtime mínimo

- Module manager + registry.
- Hooks base (`onLoad`, `onUnload`, `onQueryChange`, `decorateItems`).
- Providers y comandos (registro + pipeline).

## Fase M2 — UI primitives v1

- Render de `quickSelectKey` (chip/badge) en core.
- Input numérico resuelto por core.
- Input Accessory en input bar (right-aligned) con prioridad.

## Fase M3 — Carga externa y hot reload básico

- `modules/*/module.toml` + entry.
- Hot reload simple (unload/reload/register).
- manejo de errores por módulo sin tumbar launcher.

## Fase M4 — Harden y observabilidad

- métricas por módulo (tiempo hooks/providers),
- debug flags (`--modules-debug`),
- control de permisos/capabilities,
- tests de contrato/aislamiento/regresión.

---

## 8) Checklist ejecutable

## Contrato/API
- [ ] Crear `src/modules/types.rs` con schema v1.
- [ ] Definir `api_version` para módulos.
- [ ] Publicar reglas de compatibilidad/deprecación.

## Runtime
- [ ] Implementar module registry.
- [ ] Implementar dispatch de hooks con orden determinista.
- [ ] Implementar `ctx` y actions con validación.

## Contenido
- [ ] Implementar providers modulares.
- [ ] Integrar providers al pipeline de dataset/ranking.
- [ ] Implementar comandos modulares.

## UI primitives
- [ ] Soporte `capabilities.quickSelectKey` en core.
- [ ] Soporte `decorations.badge/hint` en render.
- [ ] Soporte Input Accessory (`set/clear`, prioridad, render).

## Loader/hot reload
- [ ] Parser de `module.toml`.
- [ ] Descubrimiento de carpeta `modules/`.
- [ ] Reload simple seguro.

## Calidad
- [ ] Tests de contrato de hooks/actions.
- [ ] Tests de aislamiento de errores de módulo.
- [ ] Tests de conflict resolution (prioridad accessory).
- [ ] Tests de regresión UX (chips/accessory no rompen layout).

## Docs
- [ ] Especificar manifest v1.
- [ ] Especificar hooks/ctx/actions.
- [ ] Especificar primitives v1.
- [ ] Añadir guía de authoring de módulos.

---

## 9) Riesgos y mitigaciones

1. **API demasiado grande temprano**
   - Mitigar: v1 mínima, capabilities explícitas, versionado.

2. **Módulos rompen UX visual**
   - Mitigar: primitives semánticas, render centralizado en core.

3. **Acoplamiento accidental a internals**
   - Mitigar: `ctx` estricto, sin punteros/estado global mutable.

4. **Providers lentos bloquean experiencia**
   - Mitigar: budgets/timeout/cancelación por query.

5. **Errores de módulo tumban launcher**
   - Mitigar: aislamiento por módulo + fail-safe + desactivación puntual.

---

## 10) Definición de Done (arquitectura modular v1)

Se considera v1 lista cuando:

- existe runtime de módulos con hooks/ctx/actions/documentación,
- providers y comandos modulares están operativos,
- quickSelect + input accessory funcionan por metadata/actions,
- módulos no tienen acceso a renderer ni internals críticos,
- hay tests de contrato + aislamiento,
- hot reload básico funciona,
- docs de authoring y manifest están cerradas.

---

## 11) Próxima extensión documental (espacio reservado)

Primitivas documentadas actualmente:

- [x] `item_trailing_accessories_primitive.md`

Pendientes sugeridas:

- [ ] `ui_footer_status_primitive.md`
- [ ] `ui_secondary_panel_primitive.md`
- [ ] `item_inline_hint_primitive.md`
- [ ] `command_palette_enrichment_primitive.md`

Cada nueva primitive debe seguir `PRIMITIVE_TEMPLATE.md`.
