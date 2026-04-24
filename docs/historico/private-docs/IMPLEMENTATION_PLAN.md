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
- Enter ejecuta target con backend nativo (`ShellExecuteW`) + fallback controlado a `cmd /C start` cuando aplica.
- Modo `stdin/-e` sigue funcionando (estilo dmenu).

## Problemas conocidos

- Falta consolidar backend de lanzamiento con cobertura ampliada para casos edge de quoting/targets especiales.
- `main.rs` todavía concentra UI Win32 (mantenibilidad limitada).
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

## Fase E — Web/Distribución de módulos (predocumentación)

Agregar en la página web oficial un sector **Modules** con catálogo de módulos mantenidos por el proyecto.

Cada módulo del catálogo debe incluir, como mínimo:
- nombre,
- descripción corta,
- versión,
- descarga en formato `.rmod` (single-file),
- descarga en formato carpeta (template/dev).

Objetivo:
- facilitar adopción sin fricción,
- unificar distribución oficial,
- mantener paridad entre formato single-file y formato desarrollo.

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

## SPRINT 4 (deuda técnica prioritaria)

- [x] **T-019 Corregir formato real del índice**
  - [x] Migrar cache a JSON real (`%APPDATA%/rmenu/index.json`).
  - [x] Serializar/deserializar con formato explícito y versionado.
  - [x] Añadir migración/backward handling de cache anterior (TSV-like legacy).

- [x] **T-020 Invalidez/refresh de cache por cambios del sistema**
  - [x] Detectar cambios en Start Menu y PATH para invalidar cache.
  - [x] Estrategia implementada: `mtime` roots + firma hash de PATH.
  - [x] Soporte de refresh manual (`--reindex`) y refresh automático seguro.

- [x] **T-021 Endurecer ejecución (salida de `cmd /C start`)**
  - [x] Introducir backend nativo de lanzamiento (ShellExecuteW) para Windows.
  - [x] Mantener fallback controlado solo donde sea necesario.
  - [x] Cubrir casos base de quoting/URLs/shell-like fallback con tests.

- [x] **T-022 Extraer UI Win32 de `main.rs`**
  - [x] Mover message loop, paint y keyboard handling a `src/ui_win32.rs`.
  - [x] Reducir `main.rs` a orquestación (config + dataset + arranque UI).
  - [x] Definir contratos claros entre UI y core (`src/ranking.rs` + `ui_win32::run_ui(...)`).

- [x] **T-023 Migración de render ANSI a Unicode**
  - [x] Reemplazar camino `TextOutA` por render wide (`TextOutW`).
  - [x] Mantener render de input/lista/hints en UTF-16.
  - [x] Eliminar camino ANSI de dibujo en UI Win32.

- [x] **T-024 Métricas de UX real (latencia visible)**
  - [x] Medir `time_to_window_visible_ms`.
  - [x] Medir `time_to_first_paint_ms`.
  - [x] Medir `time_to_input_ready_ms` (primer frame interactivo).
  - [x] Exponer estos valores en `--metrics` y documentar metodología.

- [x] **T-025 Higiene documental de reportes de auditoría**
  - [x] Mover snapshot histórico a `docs/audits/codebase-report-2026-04-22.md`.
  - [x] Dejar `codebase-report.md` como puntero explícito (no fuente de estado actual).
  - [x] Mantener README/plan como fuentes vivas de estado.

## WEB / ECOSISTEMA MÓDULOS (predocumentación)

- [ ] **T-026 Sector "Modules" en web oficial**
  - [ ] Listar todos los módulos oficiales mantenidos por el proyecto.
  - [ ] Mostrar por módulo: nombre, descripción y versión actual.
  - [ ] Publicar ambos formatos de descarga por módulo: `.rmod` y carpeta.
  - [ ] Mantener sincronía de versión entre ambos formatos.

## ESTADO DE PRODUCCIÓN ACTUAL (SNAPSHOT PARA REINICIO DE CHAT)

Esta sección es la fuente de verdad rápida para retomar trabajo en un chat nuevo.

### Qué está listo y estable (core)

- Launcher mode por defecto operativo (sin `-e` / sin stdin).
- Script mode (`-e` y stdin) operativo.
- Fuentes activas: History + Start Menu + PATH.
- Ranking fuzzy v1 estable con boosts por fuente y blacklist configurable.
- Matching mantiene compatibilidad por nombre técnico de ejecutable (`mspaint`, `powershell`) aunque el label visual sea amigable.
- Fast-path de ranking: evita cálculo secundario de target cuando el match del label ya es fuerte (menor costo por query).
- Scroll/selección de lista estable (selección visible).
- Render en dos columnas (label a izquierda, hint de ruta a derecha).
- Label visual enriquecido para `.exe` (usa `FileDescription` cuando está disponible; ej. `Paint`, `PowerShell`) y fallback por alias para casos `WindowsApps` (`mspaint` -> `Paint`).
- Render de texto en UI migrado a Unicode (`TextOutW`, UTF-16) para nombres/rutas no ASCII.
- Métricas UX reales expuestas en `--metrics` (`time_to_window_visible_ms`, `time_to_first_paint_ms`, `time_to_input_ready_ms`).
- Modo de medición UI evita focus/sleep no esenciales para reducir ruido de `--metrics`.
- Historial persistente operativo.
- Cache de índice operativo (`%APPDATA%/rmenu/index.json`, JSON versionado + firma de entorno).
- Backend de launch nativo operativo (`ShellExecuteW`) con fallback controlado.
- Diagnóstico operativo:
  - `--debug-ranking <query>`
  - `--metrics` (incluye latencias UX: window visible / first paint / input-ready)
  - `--reindex` (refresh manual de índice)
- Módulos extraídos:
  - `src/app_state.rs`
  - `src/fuzzy.rs`
  - `src/sources/mod.rs`
  - `src/launcher.rs`
  - `src/ranking.rs`
  - `src/ui_win32.rs`

### Qué se terminó en roadmap

- Sprint 1: T-001 a T-007 completados.
- Sprint 2: T-008 a T-011 completados.
- Sprint 3: T-012 a T-014 completados.
- Sprint 4: T-019, T-020, T-021, T-022, T-023, T-024 y T-025 completados.

### Deuda técnica prioritaria (orden recomendado Sprint 4)

1. Consolidar benchmarks periódicos en README sin números sintéticos (usar resultados medidos por máquina).

### Decisiones de producto vigentes (no romper)

- No meter hotkey/daemon todavía (queda pospuesto deliberadamente).
- Priorizar latencia/estabilidad/ranking antes de features extra.
- Mantener `README + plan` alineados con implementación real.
- Tratar `codebase-report.md` como puntero histórico; los snapshots viven en `docs/audits/`.

### Comandos de verificación al retomar

- `cargo check`
- `cargo test`
- `rmenu.exe --metrics`
- `rmenu.exe --debug-ranking pow`
- `rmenu.exe --reindex`

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
