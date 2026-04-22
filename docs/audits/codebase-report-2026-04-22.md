# Rmenu — Codebase Report (2026-04-22)

## 1) Visión general

`rmenu` es un launcher/menu para Windows inspirado en `dmenu`:
- Recibe opciones por `stdin` o `-e/--elements`.
- Filtra por texto en vivo.
- Permite seleccionar con teclado.
- Imprime el ítem seleccionado por `stdout` (componible en scripts).

Está implementado en **Rust** con API Win32 directa vía crate `windows`, sin framework UI externo.

---

## 2) Estado actual del repositorio

### Git / estado de trabajo
- Rama: `main` (tracking `origin/main`)
- Estado: **clean** (sin cambios locales)
- Último commit: `76dee6d`
- Historial: repo pequeño (4 commits)

### Archivos versionados (`git ls-files`)
- `.gitignore`
- `Cargo.lock`
- `Cargo.toml`
- `LICENSE`
- `POWERSHELL_EXAMPLES.md`
- `README.md`
- `build.bat`
- `build.rs`
- `config_example.ini`
- `img.jpg`
- `install.bat`
- `resource.rc`
- `src/main.rs`
- `src/settings.rs`

### Estado de compilación
- `cargo check`: **OK**
- Warnings de build script: copia de `config_example.ini` y `README.md` al output.

### Tamaño / complejidad aproximada
- Código Rust principal:
  - `src/main.rs`: 509 líneas
  - `src/settings.rs`: 394 líneas
- Total líneas de archivos clave inspeccionados: ~2107
- Binario debug observado: `target/debug/rmenu.exe` ~551 KB

---

## 3) Estructura del proyecto

- `src/main.rs`
  - Entrada principal de la app.
  - Loop de mensajes Win32.
  - Render manual (GDI) de barra + lista.
  - Manejo de teclado (Enter/Esc/Up/Down/Backspace/Tab).
  - Lógica de matching simple (`contains`, case-sensitive opcional).
  - Cálculo de geometría/layout.

- `src/settings.rs`
  - Modelo de configuración (`RmenuConfig`) y sub-secciones.
  - Defaults.
  - Parseo INI custom (manual, sin crate INI dedicada).
  - Parseo de argumentos CLI (`parse_args`) también manual.
  - Merge de overrides CLI sobre config.

- `build.rs`
  - Embebe recursos Windows (`resource.rc`).
  - Copia `config_example.ini` y `README.md` al dir de output de cargo.

- `resource.rc`
  - Metadata de versión para exe (actualmente 0.1.0, desalineado con `Cargo.toml` 0.2.0).

- `build.bat` / `install.bat`
  - Scripts de build e instalación para Windows.

- `README.md` / `POWERSHELL_EXAMPLES.md`
  - Documentación de uso y scripting.

- `config_example.ini`
  - Config de ejemplo extensa.

---

## 4) Cómo funciona internamente

## 4.1 Flujo de arranque
1. Parsea CLI (`parse_args`).
2. Carga config (`RmenuConfig::load`), con fallback a defaults.
3. Aplica overrides CLI.
4. Obtiene elementos:
   - `-e/--elements` (delimitador configurable), o
   - `stdin` si no es TTY.
5. Inicializa estado global (`APP_STATE`, `CONFIG`) con `Mutex<Option<...>>`.
6. Registra clase Win32, crea ventana popup topmost/toolwindow/appwindow.
7. Entra al loop de mensajes con `PeekMessageW`.

## 4.2 Render y UI
- Render en `WM_PAINT` usando GDI (`FillRect`, `TextOutA`, etc.).
- Dibuja:
  - fondo
  - borde
  - prompt opcional
  - input actual
  - lista de matching items
- Item seleccionado resaltado por colores de selección.

## 4.3 Input de teclado
- `Esc` -> `PostQuitMessage(1)`
- `Enter` -> imprime selección/input y `PostQuitMessage(0)`
- `Up/Down` -> mueve índice seleccionado
- `Backspace` -> borra carácter
- `Tab` -> autocompleta con item seleccionado
- `WM_CHAR` -> agrega carácter imprimible al input

## 4.4 Matching
`update_matching_items_refactored`:
- Si input vacío: muestra todo.
- Si no: `contains` sobre cada item.
- Respeta `case_sensitive`.

**Nota:** no hay fuzzy real (score/ranking), sólo substring filter.

---

## 5) Hallazgos importantes (estado real vs docs)

## 5.1 Desalineaciones funcionales/documentales
1. `README.md` afirma “application launcher”, pero el core real hoy es más “menu filter/selector”.
2. `config_example.ini` incluye secciones no implementadas en código:
   - `[Scripts]`
   - `[Hotkeys]`
   - `[Shortcuts]` (experimental)
   - `[AdvancedAppearance]`
3. `instant_selection` existe en config y docs, pero explícitamente no implementado.
4. `resource.rc` marca versión `0.1.0`; `Cargo.toml` está en `0.2.0`.
5. Ejemplos en `build.bat` usan flags inválidos:
   - `-elements` (debería ser `-e`/`--elements`)
   - `-prompt` (debería ser `-p`/`--prompt`)
   - `-help` (debería ser `-h`/`--help`)

## 5.2 Arquitectura técnica
1. Estado global con `static Mutex<Option<...>>` acoplado a window proc.
2. Uso extenso de `unsafe` esperado por Win32, pero sin encapsulación fuerte por módulos.
3. `WM_PAINT` crea recursos GDI (`CreateFontW`, `CreateSolidBrush`) sin gestión robusta de lifecycle (potencial fugas GDI).
4. `TextOutA` usa bytes ANSI; para Unicode real en Windows convendría camino wide (`TextOutW`).
5. Hay `sleep(50ms)` en `WM_CREATE` para focus; anti-pattern posible para latencia percibida.
6. Bucle con `PeekMessageW` + `yield_now` busy-ish; podría optimizarse según objetivo de consumo mínimo.

## 5.3 Producto / UX
1. No hay hotkey global en la app.
2. No hay historial persistente.
3. No hay indexado de apps/paths del sistema.
4. No hay fuzzy ranking (solo contains).
5. README aún no está orientado al “núcleo mínimo ultra rápido” que querés llevar.

---

## 6) Riesgos actuales

- **Riesgo de scope creep documental:** docs prometen más de lo que implementa el binario.
- **Riesgo técnico:** gestión GDI en render repetido puede degradar sesiones largas.
- **Riesgo de experiencia:** sin hotkey + sin historial + sin catálogo de apps, cuesta uso diario real como launcher.
- **Riesgo de posicionamiento:** hoy es percibido como “dmenu para scripts”, no como launcher cotidiano.

---

## 7) Oportunidades de mejora (priorizadas)

## P0 (crítico para “usable por otro humano”)
1. **Hotkey global** (ej. Alt+Space configurable).
2. **Modo launcher mínimo**: escribir y ejecutar (`ShellExecute`/`CreateProcess`).
3. **Historial persistente** (archivo en `%APPDATA%/rmenu/history.txt`).
4. **Fuentes de ítems básicas**:
   - historial
   - PATH (.exe/.cmd/.bat)
   - Start Menu entries (idealmente)
5. **Fuzzy rápido simple** (score por prefix + substring + orden estable).
6. **README aterrizado + GIF corto real**.

## P1 (estabilidad/calidad)
1. Arreglar scripts `.bat` con flags correctos.
2. Alinear versiones (`resource.rc` vs `Cargo.toml`).
3. Limpiar `config_example.ini` (o implementar realmente secciones extra).
4. Migrar render a Unicode (`TextOutW`).
5. Encapsular creación/liberación GDI objects.

## P2 (más adelante)
1. Refactor por módulos (`ui`, `input`, `sources`, `exec`, `history`).
2. Métricas de latencia de arranque y tiempo a primer paint.
3. Tests de parsing/config/fuzzy scoring.

---

## 8) Plan de implementación completo (propuesto)

## Fase 0 — Baseline (30–45 min)
- Medir punto de partida:
  - tiempo de arranque percibido
  - tiempo de filtrado con 1k/10k ítems
- Confirmar flujo actual de salida por stdout para no romper scripting.
- Definir “modo launcher” explícito (nuevo flag o modo por default sin stdin).

**Entregable:** métricas base + criterios de aceptación cerrados.

## Fase 1 — “Versión explotable” mínima (2–4 h)
Objetivo: exactamente el checklist que propusiste.

### 1. Hotkey global
- Registrar hotkey (`RegisterHotKey`) en proceso residente o helper.
- Al activarse: abrir/mostrar rmenu en foreground.
- Configurable en `config.ini` (`hotkey = Alt+Space` básico).

### 2. Escribir -> ejecutar
- Si Enter con item seleccionado: ejecutar target.
- Si Enter sin selección: intentar ejecutar input literal.
- Resolución simple:
  - `PATH`
  - rutas absolutas
  - extensiones ejecutables típicas (`.exe`, `.cmd`, `.bat`, `.lnk`).

### 3. Historial persistente
- Guardar query/selección exitosas en archivo plano (append + dedupe básica).
- Cargar historial al inicio como fuente prioritaria.

### 4. Paths del sistema
- Escanear:
  - directorios de `PATH`
  - Start Menu (usuario + all users) si llegás en tiempo
- Cache en memoria por sesión.

### 5. Fuzzy básico ultra rápido
- Scoring entero simple:
  - prefix match fuerte
  - substring match medio
  - fallback
- Limitar top-N y cortar temprano para mantener latencia baja.

### 6. README + GIF
- README nuevo corto:
  - qué hace
  - hotkey
  - instalación
  - 4 comandos máximos
- GIF de 10–20s mostrando:
  - abrir hotkey
  - escribir
  - ejecutar app
  - historial reaparece

**Entregable:** release “usable por otro humano”.

## Fase 2 — Robustez inmediata (1–2 días)
- Corregir lifecycle GDI para evitar leaks.
- Unicode end-to-end (render + input + ejecución).
- Manejo de errores de ejecución visible pero silencioso opcional.
- Normalizar config/documentación.

## Fase 3 — Performance branding (1 semana)
- Medición formal:
  - cold start ms
  - time-to-first-paint
  - fuzzy over N items
  - memoria residente
- Optimizar target “<10 ms open” donde sea factible.
- Publicar benchmark reproducible en README.

---

## 9) Recomendaciones concretas para tu estrategia

1. **Sí, hacerlo más limitado es la decisión correcta.**
2. Reescribí el relato del proyecto: de “mega configurable” a “launcher mínimo absurdamente rápido”.
3. Evitá tocar features no esenciales hasta tener:
   - hotkey
   - ejecutar
   - historial
   - índice de apps
4. Todo cambio debe responder a una métrica de latencia/uso diario.

---

## 10) Próximos pasos sugeridos (inmediatos)

1. Crear branch de trabajo para “launcher-minimal-v1”.
2. Implementar Fase 1 en este orden:
   1) ejecución
   2) historial
   3) indexado PATH
   4) fuzzy simple
   5) hotkey
   6) README + GIF
3. Publicar una release temprana sin features extras.

---

## Anexo — Dependencias actuales

- `windows = 0.48.0`
- `dirs = 5.0.1`
- `atty = 0.2`
- build-dep: `embed-resource = 2.5.1`

Dependencias bajas y adecuadas para objetivo de binario liviano.