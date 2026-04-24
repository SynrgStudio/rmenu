# STATE — rmenu

Fecha: 2026-04-24
Branch: `main`
Estado git al crear este archivo: working tree limpio antes de agregar `STATE.md`.

---

## Resumen ejecutivo

`rmenu` tiene el launcher core funcionando y una primera validación real de módulos completada con `calculator.rmod`.

El sistema modular soporta módulos externos `.rmod` y módulos de directorio con `module.toml`, host externo, IPC, capabilities, providers, commands, input accessories, key hooks, decorations, quick select, dedupe y diagnósticos.

La implementación funcional validada hasta ahora es:

- launcher core estable;
- módulo real `calculator` funcionando;
- `InputAccessory` desde módulo externo funcionando end-to-end;
- `ctx.replaceItems([])` desde módulo externo funcionando para suprimir resultados fuzzy durante cálculo;
- render de input accessory sin prefijo `[success]`;
- no hay spam de `permission_denied` por hooks no declarados.

---

## Última validación técnica

Comandos ejecutados:

```bash
cargo check
cargo test
```

Resultado:

```text
43 tests passed
0 failed
```

No se ejecutó `cargo fmt` global porque `cargo fmt --check` quiere reformatear varios archivos preexistentes y generaría ruido grande no relacionado.

---

## Estado actual de módulos

### 1. Calculator

Archivo:

```text
modules/calculator.rmod
```

Estado: funciona y fue validado manualmente.

Comportamiento esperado:

- abrir `rmenu`;
- escribir un cálculo simple sin prefijo:

```text
2+2
10/2
(2+3)*4
```

Resultado:

- muestra el resultado en la misma barra del input, alineado a la derecha;
- formato: `=<resultado>`;
- color: verde por `InputAccessoryKind::Success`;
- no aparece `[success]` en el texto;
- la lista inferior se limpia mientras el cálculo es válido;
- no aparecen resultados fuzzy raros durante un cálculo.

Capabilities declaradas:

```text
input-accessory
```

Importante: el core no contiene lógica de calculadora. El core solo provee primitives genéricas. La lógica de cálculo vive en el `.rmod`.

### 2. Local scripts

Estado: revertido / no implementado actualmente.

Se intentó implementar un módulo `modules/local-scripts/` para validar providers + commands, pero la UX quedó inconsistente:

- los items del provider competían con History/Start Menu/PATH;
- los exact triggers no quedaban arriba de forma fiable;
- varios parches hicieron que fuzzy/ranking se volviera confuso;
- se decidió revertir todo lo relacionado a `local-scripts` y volver al último estado funcional.

Lección: antes de reimplementar `local-scripts`, conviene diseñar formalmente una primitive o policy para `intent shortcuts` / exact provider matches, en vez de forzarlo con ranking, hints o reemplazos ad hoc.

---

## Cambios de core ya presentes y funcionales

Estos cambios están en el último estado funcional:

- host externo puede devolver actions al core;
- `ctx.setInputAccessory(...)` real desde módulos externos;
- `ctx.clearInputAccessory()` real desde módulos externos;
- `ctx.replaceItems(...)` real desde módulos externos;
- el ciclo de UI respeta `replaceItems([])` para evitar fuzzy/ranking posterior;
- `input_accessory_text()` renderiza solo `accessory.text`, sin prefijo de kind;
- hooks externos sin capability no se invocan y no generan spam de `permission_denied`.

---

## Documentación y specs

Specs oficiales en root:

```text
MODULES_ARCHITECTURE.md
MODULES_API_SPEC_V1.md
RMOD_SPEC_V1.md
MANIFEST_SPEC_V1.md
CTX_ACTIONS_SPEC_V1.md
PROVIDER_EXECUTION_POLICY.md
ERROR_ISOLATION_POLICY.md
MODULES_CAPABILITIES_MATRIX.md
MODULES_AUTHORING_GUIDE.md
MODULES_OPERATIONS_GUIDE.md
MODULES_QUICKSTART.md
DECISIONS.md
CORE_CLOSURE_CHECKLIST.md
```

Histórico preservado en:

```text
docs/historico/
```

La documentación del calculator fue añadida previamente. Si se reinicia el trabajo, revisar:

- `CORE_CLOSURE_CHECKLIST.md`
- `MODULES_QUICKSTART.md`
- `MODULES_OPERATIONS_GUIDE.md`

---

## Tasklist actual

Fuente primaria:

```text
CORE_CLOSURE_CHECKLIST.md
```

### Fase 1 — Documentación, vocabulario y frontera arquitectónica

Estado: mayormente completada.

Completado:

- arquitectura modular documentada;
- specs v1 en root;
- histórico movido a `docs/historico/`;
- README actualizado;
- metadata de `Cargo.toml` limpiada;
- `.gitignore` revisado.

### Fase 2 — Validación con módulos reales

#### 2.1 Calculator

Estado: completado salvo acción futura de copiar/usar resultado.

Completado:

- módulo `.rmod` creado;
- detección de cálculos simples sin prefijo `=`;
- resultado vía `InputAccessory`;
- lista inferior limpiada con `replaceItems([])`;
- capabilities mínimas;
- fricciones documentadas.

Pendiente opcional:

- definir UX para copiar/usar resultado con Enter o comando oficial.

#### 2.2 Scripts/commands

Estado: pendiente.

Intento anterior revertido.

Pendiente:

- diseñar comportamiento antes de implementar;
- decidir si se necesita una primitive/policy para exact intent matches;
- crear módulo que liste scripts/comandos locales;
- aportar items vía provider;
- usar subtitles/badges/hints;
- registrar comandos namespaced;
- validar dedupe/ranking sin hacks;
- documentar fricciones.

Recomendación para reintento:

- no implementar ranking especial usando `hint` o strings mágicos;
- no depender solo de `provider_first` para intención exacta;
- considerar una primitive explícita, por ejemplo:
  - `matchKind: "exact" | "fuzzy"`,
  - `priorityBoost`,
  - `intentKey`,
  - o una policy de provider exact match;
- diseñar primero en spec antes de tocar core.

#### 2.3 Hotkeys/quick actions

Estado: pendiente.

Pendiente:

- crear módulo que use quick select keys o key hooks;
- declarar capability `keys`;
- confirmar denegación sin capability;
- confirmar duplicados de quick select;
- confirmar warnings/debug;
- confirmar que no requiere cambios al core.

#### 2.4 Revisión de fricciones

Estado: parcialmente iniciada.

Fricciones conocidas:

1. Módulos externos necesitaban actions reales hacia core.
   - Resuelto para `setInputAccessory`, `clearInputAccessory`, `replaceItems`.

2. Render de input accessory incluía `[success]`.
   - Resuelto: se renderiza solo el texto.

3. Hooks no declarados generaban `permission_denied` ruidoso.
   - Resuelto: no se invocan hooks externos sin capability.

4. Provider exact intent vs ranking fuzzy/core.
   - No resuelto. Requiere diseño antes de reimplementar `local-scripts`.

---

## Fases posteriores pendientes

### Fase 3 — Hardening funcional del core existente

Pendiente validar/fortalecer:

- aislamiento real por módulo externo;
- timeout por request;
- host colgado se mata/degrada;
- auto-restart con backoff;
- auto-disable tras umbrales;
- reload resetea contadores correctos;
- errores de un módulo no rompen otros;
- `--modules-debug` suficiente.

### Fase 4 — Tests y verificación

Tests existentes: 43.

Pendiente agregar pruebas para:

- manifest válido/inválido;
- capabilities permitidas/denegadas;
- provider timeout;
- provider item cap;
- hot reload;
- host restart/backoff;
- auto-disable;
- payload limits;
- input accessory priority/kind;
- actions externas desde host (`setInputAccessory`, `replaceItems`, registro futuro si se reintroduce).

### Fase 5 — Pulido final de producto/core

Pendiente:

- revisar `config_example.ini` completo;
- documentar mejor `[Modules]`;
- validar UX base sin módulos;
- validar performance;
- confirmar defaults seguros.

### Fase 6 — Declaración de freeze

No iniciar todavía.

Bloqueantes:

- faltan al menos dos módulos reales validados;
- falta hardening;
- faltan tests específicos;
- falta performance mínima;
- falta decisión de exact provider intent/ranking.

---

## Recomendación para próximo paso

No seguir con `local-scripts` directamente.

Próximo paso recomendado:

1. Crear una mini spec para `Provider exact intent / local script shortcuts`.
2. Decidir si entra como extensión v1 opcional o si se resuelve solo con provider priority formal.
3. Recién después implementar `local-scripts` v2.

Alternativa más segura:

- saltar temporalmente a Fase 2.3 `hotkeys/quick actions`, porque valida otra primitive y evita el problema abierto de ranking exacto.

---

## Comandos útiles

Validación no visual:

```bash
cargo check
cargo test
```

Build release:

```powershell
cargo build --release
.\target\release\rmenu.exe
```

Diagnóstico de módulos:

```powershell
.\target\release\rmenu.exe --modules-debug
```

Nota: `rmenu` busca módulos en `modules/` relativo al current working directory. Ejecutar desde:

```powershell
cd D:\rmenu
.\target\release\rmenu.exe
```
