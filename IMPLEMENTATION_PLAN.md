# rmenu — Plan serio de implementación

Fecha: 2026-04-22
Estado base: launcher Win32 en Rust funcional, sin daemon/hotkey global, con fuzzy v1, historial y fuentes automáticas.

---

## 1) Objetivo de producto

Convertir `rmenu` en un launcher diario para Windows, rápido y predecible, con foco en:

1. latencia baja percibida,
2. ranking útil (apps reales primero),
3. operación estable,
4. ciclo de desarrollo simple (sin fricción por procesos residentes en esta etapa).

---

## 2) Estado actual (resumen técnico)

## Qué hace hoy

- UI Win32/GDI en una ventana popup.
- Input por teclado + lista de resultados.
- Fuzzy scoring v1 (exact/prefix/token/contains/subsequence + penalties).
- Modo launcher automático cuando no hay `-e` ni `stdin`.
- Fuentes actuales:
  - historial persistente,
  - Start Menu,
  - PATH.
- Enter ejecuta target (`cmd /C start`).
- Modo `stdin/-e` sigue funcionando (estilo dmenu).

## Problemas conocidos

- Índice se construye cada ejecución (sin cache versionada).
- Blacklist de comandos CLI está hardcodeada.
- No hay metadatos visuales para distinguir duplicados de nombre.
- No hay mediciones de performance formales.
- Código centralizado en `src/main.rs` (mantenibilidad limitada).
- Sin hotkey global (decisión intencional por ahora para iterar rápido).

---

## 3) Principios de ejecución

1. **No scope creep**: primero calidad del core, luego extras.
2. **Dev loop rápido**: evitar daemon/hotkey hasta que el core cierre.
3. **Todo cambio con criterio medible**: ranking, latencia, estabilidad.
4. **Fallback simple**: si falla una fuente, launcher sigue funcionando.
5. **Compatibilidad práctica**: mantener `-e`/`stdin` útil para scripting.

---

## 4) Arquitectura objetivo (sin reescritura completa)

Dividir `main.rs` en módulos dentro de `src/`:

- `app_state.rs` — estado de UI y tipos (`LauncherItem`, fuentes, selección).
- `fuzzy.rs` — scoring y ordenamiento.
- `sources/`
  - `history.rs`
  - `start_menu.rs`
  - `path.rs`
  - `index.rs` (merge/dedupe/cache)
- `launcher.rs` — ejecución de targets + resolución.
- `ui_win32.rs` — paint/input/message loop.
- `config_runtime.rs` — config cargada + defaults runtime.

No cambia comportamiento por sí mismo; mejora mantenibilidad y velocidad de iteración.

---

## 5) Roadmap por fases

## Fase A — Core estable (1-2 días)

### A1. Config externa para ranking/sources

Agregar en `config.ini` sección runtime:
- `launcher_mode_default = true`
- `enable_start_menu = true`
- `enable_path = true`
- `enable_history = true`
- `history_max_items = 300`
- `source_boost_history = 650`
- `source_boost_start_menu = 480`
- `source_boost_path = 0`
- `blacklist_path_commands = powercfg,where,whoami,...`

**Aceptación**
- Se pueden tocar boosts/blacklist sin recompilar.

### A2. Subtítulo de resultado

Mostrar `label` + `target` (ruta abreviada) o `source` para distinguir duplicados.

**Aceptación**
- Si hay 3 resultados con mismo label, el usuario distingue cuál abrir.

### A3. Cache de índice en disco

- Guardar índice combinado en `%APPDATA%/rmenu/index.json`.
- Estrategia simple:
  - carga cache si existe,
  - refresco en background opcional no bloqueante en futuras iteraciones, o refresh on startup con timeout.

**Aceptación**
- Segunda apertura notablemente más rápida con mismo entorno.

---

## Fase B — Calidad operacional (2-4 días)

### B1. Observabilidad mínima

Agregar flag `--debug-ranking <query>` para imprimir top N con score + fuente.

**Aceptación**
- Se puede diagnosticar por qué rankeó primero X sobre Y.

### B2. Casos de regresión (tests unitarios)

Extraer fuzzy a `fuzzy.rs` y agregar tests para:
- `pow` prioriza `PowerShell`/`Windows PowerShell` sobre ruido PATH.
- exact > prefix > contains > subsequence.
- compact match (`7zip` vs `7-zip`).
- blacklist excluye comandos definidos.

**Aceptación**
- `cargo test` verde con set de regresión.

### B3. Mejoras de ejecución

- Mensajes de error claros en `stderr` (si no silent).
- Resolver mejor rutas con espacios y targets `.lnk`.

**Aceptación**
- Menos falsos "no abre" en apps comunes.

---

## Fase C — Performance branding (2-3 días)

### C1. Presupuestos y métricas

Definir y medir:
- `startup_to_window_visible_ms` (cold/warm),
- `search_p95_ms` en 1k/5k ítems,
- tamaño de índice,
- memoria aproximada.

### C2. Optimización orientada a presupuesto

- precomputar campos normalizados (`label_lc`, `label_compact`),
- evitar realocaciones innecesarias,
- ordenar top-K sin ordenar todo si es necesario.

### C3. README de rendimiento

Publicar cifras reales y método reproducible.

**Aceptación**
- README con métricas concretas, no claims vagos.

---

## Fase D — Integración global opcional (después de estabilizar)

No ahora. Cuando A/B/C estén cerradas.

Opciones:
1. `rmenu-daemon.exe` separado para hotkey global.
2. `rmenu.exe --daemon` con lifecycle claro.

**Regla**: no romper el flujo de desarrollo sin daemon.

---

## 6) Definición de “Done” por release

## v0.3 (objetivo inmediato)

- Configurable blacklist + boosts en config.
- Subtítulo/ruta en UI.
- Cache de índice funcional.
- Tests de fuzzy/regresión básicos.
- README actualizado al modo launcher.

## v0.4

- Refactor modular completo.
- Debug ranking CLI.
- Métricas iniciales publicadas.

## v0.5

- Optimizaciones de latencia sobre métricas.
- Base lista para introducir hotkey/daemon opcional.

---

## 7) Riesgos y mitigaciones

1. **Riesgo:** ranking impredecible por cambios de scoring.
   - Mitigación: tests de regresión + `--debug-ranking`.

2. **Riesgo:** indexado lento en máquinas con PATH enorme.
   - Mitigación: cache + timeout + filtros de extensión/blacklist.

3. **Riesgo:** demasiada lógica en `main.rs` dificulta fixes.
   - Mitigación: refactor modular en Fase B.

4. **Riesgo:** hotkey demasiado temprano rompe flujo dev.
   - Mitigación: posponer a Fase D (decisión explícita).

---

## 8) Próxima iteración (tareas concretas)

Orden recomendado para empezar hoy:

1. Mover blacklist y boosts a config (`settings.rs` + parsing).
2. Renderizar subtítulo simple (`label — target abreviado`).
3. Implementar `index.json` cache básico.
4. Extraer fuzzy a módulo y agregar tests de regresión.
5. Actualizar README al estado real de launcher.

---

## 9) Lista oficial de tareas (checklist ejecutable)

## AHORA (Sprint actual)

- [x] **T-001 Config runtime en `config.ini`**
  - [x] Agregar keys: `enable_history`, `enable_start_menu`, `enable_path`.
  - [x] Agregar keys: `source_boost_history`, `source_boost_start_menu`, `source_boost_path`.
  - [x] Agregar key: `blacklist_path_commands` (CSV).
  - [x] Parsear en `settings.rs` y aplicar defaults seguros.

- [x] **T-002 Conectar ranking a config**
  - [x] Eliminar boosts hardcodeados.
  - [x] Usar valores cargados desde config.
  - [x] Fallback a defaults si falta config.

- [x] **T-003 Blacklist configurable de PATH**
  - [x] Reemplazar lista hardcodeada por set desde config.
  - [x] Mantener blacklist default mínima para seguridad UX.

- [ ] **T-004 Subtítulo/metadata visual en resultados**
  - [x] Mostrar `label` (línea principal).
  - [x] Mostrar `target` abreviado o `source` (línea secundaria o sufijo).
  - [ ] Verificar que no rompa layout en resoluciones chicas.

- [x] **T-005 Cache de índice (`index.json`)**
  - [x] Persistir índice combinado en `%APPDATA%/rmenu/index.json`.
  - [x] Cargar cache al inicio si existe.
  - [x] Rebuild si cache inválido/corrupto.

- [x] **T-006 QA manual Sprint**
  - [x] Casos: `pow`, `not`, `calc`, `code`, `cmd`, `explorer`.
  - [x] Verificar que `powercfg` no suba al top.
  - [x] Verificar ejecución real de 10 apps/targets.

- [x] **T-007 Docs mínimas actualizadas**
  - [x] README: explicar modo launcher por default.
  - [x] README: explicar config de boosts/blacklist.

## SIGUIENTE (Sprint 2)

- [x] **T-008 Extraer módulos (`fuzzy.rs`, `sources/*`, `launcher.rs`)**
  - [x] Extraer `fuzzy_score` y helpers a `src/fuzzy.rs`.
  - [x] Extraer tipos de estado (`AppState`, `LauncherItem`, `LauncherSource`) a `src/app_state.rs`.
  - [x] Mover carga de fuentes/cache/history a `src/sources/mod.rs`.
  - [x] Mover utilidades de ejecución/render-hints a `src/launcher.rs`.
- [x] **T-009 Tests de regresión de ranking (unit tests)**
- [x] **T-010 Flag `--debug-ranking <query>`**
- [x] **T-011 Mejorar mensajes de error de ejecución**

## DESPUÉS (Sprint 3)

- [x] **T-012 Métricas formales (startup, p95 search, memoria)**
- [x] **T-013 Optimizaciones de top-K y precomputados**
  - [x] Precomputar normalizados de label (`label_lc`, `label_compact`).
  - [x] Reusar `fuzzy_score_precomputed_lower` para búsquedas case-insensitive.
  - [x] Implementar selección parcial top-K (sin ordenar todo) para datasets grandes.
- [x] **T-014 README de performance con benchmark reproducible**

## BLOQUEADO / POSPUESTO (intencional)

- [ ] **T-015 Hotkey global/daemon** (solo después de cerrar A/B/C)
- [ ] **T-016 Navegación avanzada de lista** (`PageUp/PageDown`, `Home/End`, saltos rápidos)
- [ ] **T-017 Atajos numéricos por posición visible (1..10)**
  - Mostrar prefijos fijos `1)` a `10)` en las filas visibles.
  - Teclas `1..0` ejecutan la fila visible actual correspondiente (no índice global).
  - Debe seguir funcionando al scrollear: cambia el contenido, no la posición del atajo.
- [ ] **T-018 Script de stress/benchmark de ranking**
  - Ejecutar lotes grandes de queries contra `--debug-ranking`.
  - Soportar queries repetidas, mezcladas y secuenciales.
  - Exportar resultados (score/source/top-N) para análisis de estabilidad.

---

## 10) Criterio de calidad para seguir avanzando

Antes de pasar de fase:
- build/check sin errores,
- comportamiento manual validado en casos reales (`pow`, `not`, `calc`, `code`),
- evidencia objetiva (tests o métricas),
- docs alineadas con lo implementado.
