# rmenu — MODULES IMPLEMENTATION PLAN (Master Plan)

Fecha base: 2026-04-22  
Owner conceptual: Director Creativo  
Rol de ejecución: Secretario Técnico (agente)

---

## 0) Propósito de este documento

Este plan existe para reinicios de chat y continuidad operativa.

Objetivos:

1. Tener una **fuente única de verdad** para implementar el ecosistema modular.
2. Separar claramente:
   - estado actual del launcher core,
   - arquitectura objetivo de módulos,
   - roadmap y tareas ejecutables.
3. Facilitar handoff a chats nuevos sin perder decisiones críticas.

Este documento complementa (no reemplaza):

- `../IMPLEMENTATION_PLAN.md` (plan operativo del launcher actual)
- `extensions_implementation_plan.md` (visión modular resumida)
- `DECISIONS.md` (mini ADRs)
- `CHANGELOG.md` (evolución documental)

---

## 1) Principios rectores (no negociables)

1. **Autoridad del core**
   - input, estado, ranking, submit, render, lifecycle.

2. **Módulos declaran intención**
   - hooks, providers, metadata visual/semántica, actions permitidas.

3. **Core implementa realidad**
   - layout, render, clipping, estilos, ejecución, invariantes.

4. **Sin acceso arbitrario a internals**
   - no GDI/Win32 directo, no mutación de estado global interno, no control del loop.

5. **API pública pequeña y versionada**
   - v1 mínima, estable, con evolución explícita.

6. **Fail-safe por diseño**
   - un módulo no puede tumbar el launcher completo.

---

## 2) Estado actual resumido (baseline)

### Ya implementado en launcher (fuera de módulos)

- Core launcher estable con fuentes history/start/path.
- Cache JSON versionada con invalidación por firma de entorno.
- Backend de ejecución nativo (`ShellExecuteW`) con fallback controlado.
- UI Win32 separada (`src/ui_win32.rs`).
- Ranking separado (`src/ranking.rs`).
- Render Unicode (`TextOutW`).
- Métricas UX reales en `--metrics`.
- Script de auditoría unificada: `scripts/audit.ps1`.

### No implementado aún (modular runtime)

- runtime de módulos en `src/modules/*`.
- hooks/context/actions públicos funcionales.
- providers y commands modulares reales.
- primitives UI conectadas a API modular (input accessory/trailing accessories).
- loader externo `modules/*/module.toml`.

---

## 3) Alcance v1 del ecosistema modular

### Sí entra en v1

- Runtime modular interno estable.
- Hooks básicos + `decorateItems`.
- `ctx` controlado (lectura + utilidades + actions permitidas).
- Providers y commands modulares.
- Primitive `quickSelectKey` (capability oficial).
- Primitive `Input Accessory`.
- Loader declarativo básico + hot reload simple.
- Observabilidad mínima de módulos.

### No entra en v1

- Render hooks arbitrarios.
- Widgets custom dibujados por módulos.
- Inyección libre de layout.
- Sandbox complejo multiproceso con migración de estado.
- Hot reload avanzado con state migration.

---

## 4) API objetivo v1 (contrato público)

## 4.1 Hooks

- `onLoad(ctx)`
- `onUnload(ctx)`
- `onQueryChange(query, ctx)`
- `onSelectionChange(item, index, ctx)`
- `onKey(event, ctx)`
- `onSubmit(item, ctx)`
- `onCommand(command, args, ctx)`
- `decorateItems(items, ctx) -> Item[]`

## 4.2 Context (`ctx`) lectura/utilidades

- `ctx.query()`
- `ctx.items()`
- `ctx.selectedItem()`
- `ctx.selectedIndex()`
- `ctx.mode()`
- `ctx.hasCapability(name)`
- `ctx.log(message)`
- `ctx.toast(message)`

## 4.3 Actions permitidas

- `ctx.setQuery(text)`
- `ctx.setSelection(index)`
- `ctx.moveSelection(offset)`
- `ctx.submit()`
- `ctx.close()`
- `ctx.addItems(items)`
- `ctx.replaceItems(items)` (contexto restringido)
- `ctx.registerCommand(def)`
- `ctx.registerProvider(def)`
- `ctx.setInputAccessory(accessory)`
- `ctx.clearInputAccessory()`

## 4.4 Tipos canónicos

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

type InputAccessory = {
  text: string
  kind?: "info" | "success" | "warning" | "error" | "hint"
  priority?: number
}
```

---

## 5) Arquitectura de implementación (propuesta de módulos internos)

Nueva carpeta:

- `src/modules/mod.rs` — runtime manager
- `src/modules/types.rs` — contrato público
- `src/modules/context.rs` — ctx controlado
- `src/modules/actions.rs` — aplicación validada de actions
- `src/modules/hooks.rs` — dispatch de hooks
- `src/modules/providers.rs` — registro/ejecución de providers
- `src/modules/commands.rs` — registro/ruteo de comandos
- `src/modules/state.rs` — estado runtime modular (aislado)
- `src/modules/manifest.rs` — parse/validación de `module.toml` (fase loader)
- `src/modules/loader.rs` — descubrimiento/carga/reload (fase loader)
- `src/modules/permissions.rs` — capabilities/permisos

Integración core:

- `src/sources/mod.rs`: merge de fuentes core + providers módulos.
- `src/ranking.rs`: ranking sobre items canónicos (sin exponer internals).
- `src/ui_win32.rs`: render de decorations/capabilities + input accessory.
- `src/main.rs`: bootstrap del runtime modular + flags de debug.

---

## 6) Pipeline operacional objetivo

1. Query cambia
2. Core actualiza estado base
3. Providers (core + módulos) generan candidatos
4. Core merge/dedupe
5. Core ranking
6. Hook `decorateItems` aplica metadata final
7. Core renderiza (label/hint/chip/accessory)
8. Core procesa input
9. Core submit
10. Hooks post-evento

Regla: ningún paso permite mutación arbitraria fuera de actions validadas.

---

## 7) Roadmap por fases (detallado)

## Fase M0 — Contrato y esqueleto

Objetivo: congelar API v1 y crear estructura mínima compilable.

### Entregables

- `src/modules/*` creado con stubs.
- Tipos públicos (`types.rs`) definidos.
- `api_version` en runtime y manifiesto espec.
- Mini docs de contrato en `modules docs`.

### Criterio de Done

- `cargo check` verde.
- contratos claros en docs.
- no lógica de módulo aún, solo estructura.

---

## Fase M1 — Runtime mínimo interno (sin loader externo)

Objetivo: ejecutar módulos "built-in" para validar arquitectura.

### Entregables

- registro de módulos en memoria,
- dispatch hooks base (`onLoad`, `onQueryChange`, `decorateItems`),
- `ctx` lectura + log/toast,
- actions básicas (`setQuery`, `setSelection`, `moveSelection`).

### Criterio de Done

- módulo interno demo modifica comportamiento vía hooks.
- sin acceso a internals.
- tests de contrato iniciales.

---

## Fase M2 — Providers y Commands modulares

Objetivo: contenido/comandos extensibles de forma controlada.

### Entregables

- `registerProvider` operativo,
- `registerCommand` + `onCommand`,
- merge/dedupe de providers en pipeline,
- time budget básico por provider.

### Criterio de Done

- módulo interno aporta items dinámicos.
- módulo comando responde en flujo normal.
- ranking/UX core permanece estable.

---

## Fase M3 — UI primitives v1

Objetivo: habilitar enriquecimiento visual semántico sin exponer renderer.

### Entregables

1. **Trailing accessories**
   - chip de `quickSelectKey` en zona derecha,
   - hint/path coexistiendo con truncado estable.

2. **Input Accessory**
   - `set/clear`, prioridad, un slot visible.

3. **Input quick-select**
   - teclas `1..0` resueltas por core.

### Criterio de Done

- módulos solo declaran metadata.
- core renderiza consistente.
- layout no se rompe en anchos chicos (con fallback claro).

---

## Fase M4 — Loader externo y hot reload simple

Objetivo: módulos en carpeta, plug-and-play básico.

### Entregables

- formato `modules/*/module.toml`,
- descubrimiento de módulos,
- load/unload/reload simple,
- error handling por módulo.

### Criterio de Done

- cambio de archivo recarga módulo.
- error en módulo no tumba launcher.

---

## Fase M5 — Observabilidad, permisos y hardening

Objetivo: operación confiable en producción.

### Entregables

- `--modules-debug` (módulos cargados, hooks, errores),
- métricas por módulo/provider,
- permisos/capabilities enforceados,
- tests de aislamiento/fallos.

### Criterio de Done

- diagnóstico claro de problemas.
- límites de módulo aplicados.

---

## 8) Backlog ejecutable (IDs)

## M0
- [ ] **M0-001** Crear `src/modules/` con skeleton.
- [ ] **M0-002** Definir `types.rs` canónico (`Item`, `Action`, `InputAccessory`).
- [ ] **M0-003** Definir `api_version = 1` + notas de compat.

## M1
- [ ] **M1-001** Implementar registry interno de módulos.
- [ ] **M1-002** Implementar dispatch `onLoad/onUnload`.
- [ ] **M1-003** Implementar dispatch `onQueryChange`.
- [ ] **M1-004** Implementar `decorateItems` pipeline.
- [ ] **M1-005** Implementar `ctx` lectura (query/items/selection/mode).
- [ ] **M1-006** Implementar actions básicas (`setQuery`, `setSelection`, `moveSelection`).

## M2
- [ ] **M2-001** Implementar `registerProvider`.
- [ ] **M2-002** Integrar providers al pipeline de dataset.
- [ ] **M2-003** Implementar `registerCommand` + routing `/cmd`.
- [ ] **M2-004** Añadir timeout/cancelación básica por provider.

## M3
- [ ] **M3-001** Añadir `capabilities.quickSelectKey` operativo en render.
- [ ] **M3-002** Añadir input `1..0` core-resolved.
- [ ] **M3-003** Implementar `InputAccessory` state en core.
- [ ] **M3-004** Render de input accessory con prioridad.
- [ ] **M3-005** Implementar layout zones (`label + hint + chip`).

## M4
- [ ] **M4-001** Diseñar/parsear `module.toml`.
- [ ] **M4-002** Descubrimiento de carpeta `modules/`.
- [ ] **M4-003** Carga/descarga segura.
- [ ] **M4-004** Hot reload simple por file change.

## M5
- [ ] **M5-001** `--modules-debug`.
- [ ] **M5-002** Métricas por hook/provider.
- [ ] **M5-003** Permisos/capabilities enforce.
- [ ] **M5-004** Tests de aislamiento y errores.

---

## 8.1) Granularidad por archivo (plan de arranque desde cero)

Esta sección traduce fases a cambios concretos en código.

## M0 — Contrato y esqueleto (file-by-file)

### `src/modules/types.rs`
Definir structs/enums públicos mínimos:

- `ModuleApiVersion` (const o type alias)
- `ModuleItem`
- `ModuleAction`
- `ModuleItemCapabilities`
- `ModuleItemDecorations`
- `ModuleInputAccessory`
- `ModuleMode`
- `ModuleKeyEvent`

**Regla:** este archivo no depende de Win32/UI.

### `src/modules/state.rs`
Definir estado runtime modular (solo core):

- registry de módulos cargados,
- providers registrados,
- commands registrados,
- input accessory activo.

### `src/modules/mod.rs`
Exponer superficie interna:

- `pub struct ModuleRuntime`
- `pub fn new() -> Self`
- `pub fn api_version() -> u32`

### `src/main.rs`
- Instanciar `ModuleRuntime` sin uso funcional aún.

---

## M1 — Runtime interno mínimo (file-by-file)

### `src/modules/hooks.rs`
Agregar dispatcher determinista:

- `dispatch_on_load(...)`
- `dispatch_on_unload(...)`
- `dispatch_on_query_change(...)`
- `dispatch_decorate_items(...)`

### `src/modules/context.rs`
Implementar `ModuleCtx` con lectura/utilidades:

- `query/items/selected/mode`
- `log/toast`

### `src/modules/actions.rs`
Implementar primeras actions:

- `set_query`
- `set_selection`
- `move_selection`

con validación de límites.

### `src/modules/mod.rs`
Agregar API interna para registrar módulo built-in:

- `register_builtin_module(...)`

**Sin loader de disco aún.**

---

## M2 — Providers y Commands

### `src/modules/providers.rs`
- `register_provider(...)`
- `execute_providers(query, epoch, budget) -> Vec<ModuleItem>`

### `src/modules/commands.rs`
- `register_command(...)`
- `dispatch_command(...)`

### `src/sources/mod.rs` o punto de integración de dataset
- Merge de items core + providers (dedupe estable).

### `src/ranking.rs`
- Operar sobre `ModuleItem` normalizado o adapter interno estable.

---

## M3 — Primitives UI

### `src/ui_win32.rs`
Implementar render semántico de:

1. `capabilities.quickSelectKey` (chip trailing)
2. `decorations.hint` (coexistencia con chip)
3. `InputAccessory` en barra de input

Reglas de layout por zonas:

- zone label
- zone hint
- zone chip

### `src/ui_win32.rs` (input)
- Teclas `1..0` resueltas por core contra `quickSelectKey`.
- Policy configurable (`select|submit`).

### `src/modules/actions.rs`
- `set_input_accessory`
- `clear_input_accessory`
- resolución por prioridad.

---

## M4 — Loader externo + manifest

### `src/modules/manifest.rs`
- parse + validación `module.toml`

### `src/modules/loader.rs`
- descubrimiento de `modules/*`
- load/unload/reload simple

### `src/modules/permissions.rs`
- enforcement por capability declarada.

---

## M5 — Observabilidad y hardening

### `src/modules/mod.rs` + `src/main.rs`
- `--modules-debug`

### `src/modules/*`
- métricas por hook/provider
- contador de fallos consecutivos
- auto-disable opcional por umbral

---

## 8.2) Orden exacto de arranque recomendado (primer sprint técnico)

1. M0 completo (tipos + runtime vacío compilable)
2. M1 parcial: `onLoad`, `onQueryChange`, `decorateItems`
3. M3 parcial: quickSelect render + input numérico (sin loader)
4. M2 parcial: 1 provider built-in demo
5. tests de contrato básicos

Razonamiento: prioriza validar contrato+UX antes de complejidad de loader.

---

## 8.3) Contratos mínimos que deben existir antes de escribir lógica de módulos

Checklist de “gate”:

- [ ] `MODULES_API_SPEC_V1.md` congelado para M0/M1.
- [ ] `MANIFEST_SPEC_V1.md` congelado para M4.
- [ ] `CTX_ACTIONS_SPEC_V1.md` congelado para M1/M3.
- [ ] `PROVIDER_EXECUTION_POLICY.md` congelado para M2.
- [ ] `ERROR_ISOLATION_POLICY.md` congelado para M5.

Si uno de estos no está cerrado, no avanzar implementación de su fase.

---

## 8.4) Definición de interfaces iniciales (orientativa)

```rust
pub trait RuntimeModule {
    fn name(&self) -> &str;
    fn on_load(&mut self, ctx: &mut ModuleCtx) {}
    fn on_unload(&mut self, ctx: &mut ModuleCtx) {}
    fn on_query_change(&mut self, query: &str, ctx: &mut ModuleCtx) {}
    fn decorate_items(&mut self, items: &mut [ModuleItem], ctx: &mut ModuleCtx) {}
}
```

Nota: firma final puede variar, pero la intención es comenzar simple.

---

## 8.5) Criterios de freeze por fase (para no mezclar scope)

- **Freeze M0:** no agregar hooks nuevos.
- **Freeze M1:** no abrir loader externo.
- **Freeze M2:** no tocar renderer.
- **Freeze M3:** no introducir permisos avanzados.
- **Freeze M4:** no meter features visuales nuevas.
- **Freeze M5:** no expandir API; solo robustez/observabilidad.

---

## 8.6) Plantilla de progreso para reinicio de chat

Copiar/pegar en chat nuevo:

```md
### Modules Runtime Status
- Fase actual: M?
- Última tarea cerrada: M?-???
- Próximas 3 tareas:
  1. ...
  2. ...
  3. ...
- Riesgos activos:
  - ...
- Bloqueos:
  - ...
```

---

## 9) Estrategia de testing

### Unit tests

- tipos/serialización de contrato,
- dispatch de hooks,
- validación de actions,
- conflicto de prioridades (`InputAccessory`),
- conflicto quick key duplicada.

### Integration tests

- provider modular + ranking + render metadata,
- comando modular de punta a punta,
- reload módulo.

### Regression tests

- no romper launcher clásico sin módulos,
- no romper latencia objetivo,
- no romper Unicode/layout.

---

## 10) Métricas y presupuestos recomendados

Medir siempre en `release`:

- `startup_prepare_ms`
- `time_to_window_visible_ms`
- `time_to_first_paint_ms`
- `time_to_input_ready_ms`
- `search_p95_ms`
- costo extra modular (`modules_overhead_ms` futuro)

Presupuesto v1 sugerido (objetivo):

- overhead modular sin módulos activos: ~0 perceptible
- overhead con 1-2 módulos ligeros: < 1-2 ms por query típica

---

## 11) Riesgos y mitigaciones

1. **API creep**
   - Mitigar con v1 mínima + ADR por nueva capacidad.

2. **Módulos lentos bloquean UX**
   - timeout/cancelación + budgets + métricas por provider.

3. **Inconsistencia visual**
   - primitives semánticas + render centralizado.

4. **Acoplamiento accidental a internals**
   - `ctx` estricto + no exponer referencias mutables internas.

5. **Errores de módulo impactan core**
   - aislamiento + captura + desactivación selectiva.

---

## 12) Definición de Done (ecosistema modular v1)

Se considera v1 lista cuando:

1. runtime modular funcional con hooks/context/actions,
2. providers y comandos modulares operativos,
3. quickSelect y InputAccessory funcionando por contrato,
4. módulos sin acceso a renderer/internals críticos,
5. loader externo + hot reload simple estable,
6. observabilidad básica de módulos,
7. tests de contrato/aislamiento/regresión verdes,
8. docs completas en `*`.

---

## 13) Protocolo para reinicio de chat (snapshot rápido)

En un chat nuevo, empezar por:

1. `MODULES_IMPLEMENTATION_PLAN.md`
2. `extensions_implementation_plan.md`
3. `DECISIONS.md`
4. `../IMPLEMENTATION_PLAN.md` (estado core launcher)

Luego reportar:

- fase actual (M0..M5),
- IDs de tareas completadas/pendientes,
- riesgos activos,
- próximos 3 pasos.

---

## 14) Próximos documentos recomendados

- `MODULES_API_SPEC_V1.md`
- `MANIFEST_SPEC_V1.md`
- `CTX_ACTIONS_SPEC_V1.md`
- `PROVIDER_EXECUTION_POLICY.md`
- `ERROR_ISOLATION_POLICY.md`

Estos documentos ayudan a ejecutar implementación sin ambigüedades.
