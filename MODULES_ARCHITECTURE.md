# MODULES ARCHITECTURE — rmenu

Estado: Frozen v1  
Fecha: 2026-04-24  
Alcance: arquitectura pública del core modular de `rmenu`.

---

## 1. Propósito

Este documento es la constitución del sistema modular de `rmenu`.

Su objetivo es fijar:

- qué pertenece al core,
- qué debe vivir como módulo,
- qué vocabulario público existe en v1,
- qué límites no pueden cruzar los módulos,
- y bajo qué condiciones se permite volver a tocar el core.

Regla principal:

> Si una funcionalidad puede implementarse como módulo, no entra al core.

---

## 2. Identidad del core

`rmenu` es un launcher nativo para Windows y una superficie extensible de comandos.

El core debe mantenerse:

- pequeño,
- rápido,
- determinista,
- local-first,
- estable,
- y autoritativo sobre UI, estado, ranking, ejecución y policy.

El core no es un framework gráfico general ni un runtime de plugins sin límites. Es una base controlada para componer extensiones sin comprometer la experiencia principal del launcher.

---

## 3. Qué es el core

El core incluye únicamente las piezas necesarias para que `rmenu` funcione como launcher y como plataforma modular estable.

### 3.1 Launcher base

- UI Win32 base.
- Input, selección y scroll.
- Modo launcher.
- Modo script/dmenu con `stdin` o `-e`.
- Fuzzy search.
- Ranking.
- Sources internos básicos:
  - History,
  - Start Menu,
  - PATH.
- Cache de índice.
- Launch backend vía Windows/ShellExecuteW y fallback controlado.
- Configuración y CLI.
- Métricas y diagnósticos base.

### 3.2 Sistema modular

- Descubrimiento de módulos.
- Formato directorio con `module.toml`.
- Formato single-file `.rmod`.
- Normalización a descriptor interno común.
- Runtime de módulos.
- Module host externo.
- Protocolo IPC.
- Validación y sanitización de payloads.
- Capabilities y enforcement.
- Políticas de timeout, budget, dedupe, restart y disable.
- Hot reload.
- Telemetría y debug de módulos.

### 3.3 Primitivas UI permitidas para módulos

- `badge`.
- `hint`.
- `subtitle`/`source` cuando aplique.
- `quickSelectKey`.
- `InputAccessory`.

El core renderiza todas las primitivas. Los módulos solo declaran intención.

---

## 4. Qué no es el core

No pertenecen al core:

- calculadoras,
- conversores,
- buscadores especializados,
- comandos de productividad específicos,
- integraciones con apps externas,
- catálogos de scripts,
- snippets,
- workflows particulares,
- automatizaciones específicas,
- fuentes de datos opcionales,
- temas avanzados,
- widgets,
- paneles,
- renderers custom,
- layouts específicos para un módulo.

Todo eso debe implementarse como módulo, salvo que varios módulos reales demuestren una necesidad general que no pueda expresarse con las primitivas v1.

---

## 5. Frontera de responsabilidades

### 5.1 Core

El core es responsable de:

- mantener invariantes de estado,
- renderizar UI,
- aplicar ranking y dedupe,
- validar entradas de módulos,
- aislar errores,
- aplicar capabilities,
- ejecutar policies,
- lanzar targets,
- coordinar hooks y providers,
- exponer diagnósticos.

### 5.2 Module runtime

El runtime de módulos es responsable de:

- cargar descriptors,
- iniciar hosts externos,
- enrutar hooks,
- recolectar respuestas,
- aplicar budgets y timeouts,
- registrar telemetría,
- reiniciar o deshabilitar hosts defectuosos.

### 5.3 Module host

El host externo es responsable de:

- ejecutar código de módulo fuera del proceso principal,
- traducir requests IPC a hooks del módulo,
- devolver resultados serializados,
- no bloquear indefinidamente el core.

### 5.4 Módulos de usuario

Un módulo es responsable de:

- declarar metadata y capabilities,
- implementar hooks rápidos y deterministas,
- aportar items o decoraciones mediante el contrato público,
- manejar sus propios errores,
- no depender de internals,
- no asumir control del renderizado ni del event loop.

---

## 6. Vocabulario oficial v1

El vocabulario v1 queda congelado en estas primitivas.

### 6.1 Providers

Un provider aporta items externos al dataset visible.

Uso típico:

- resultados calculados,
- scripts locales,
- comandos dinámicos,
- búsquedas especializadas.

Reglas:

- requiere capability `providers`,
- está limitado por timeout y budget,
- sus items son sanitizados,
- sus resultados entran al merge/dedupe/ranking del core.

### 6.2 Commands

Un command representa una acción invocable por nombre.

Reglas:

- requiere capability `commands`,
- debe usar namespacing cuando haya colisión,
- formato recomendado: `/modulo::comando`,
- el core resuelve rutas y rechaza aliases ambiguos.

### 6.3 Decorations

Decorations enriquecen items existentes sin reemplazar el renderer.

Campos v1:

- `badge`,
- `hint`,
- `quickSelectKey`,
- `subtitle`,
- `source`.

Reglas:

- requiere capability `decorate-items`,
- el core puede truncar, ignorar o normalizar campos,
- la decoración nunca otorga control sobre layout o dibujo.

### 6.4 Input Accessory

Un input accessory muestra estado contextual asociado al input.

Campos v1:

- `text`,
- `kind`: `info | success | warning | error | hint`,
- `priority`.

Reglas:

- requiere capability `input-accessory`,
- solo hay un accessory visible a la vez,
- mayor prioridad gana,
- render y layout son responsabilidad del core.

### 6.5 Capabilities

Capabilities son permisos declarativos.

Capabilities v1:

- `providers`,
- `commands`,
- `decorate-items`,
- `input-accessory`,
- `keys`.

Reglas:

- un módulo debe declarar solo lo que usa,
- el manifest declara intención,
- el runtime aplica enforcement,
- una operación sin permiso se rechaza con `permission_denied`.

### 6.6 Key Hooks

Key hooks permiten reaccionar a teclas controladas.

Reglas:

- requiere capability `keys`,
- no permite interceptar el event loop completo,
- no permite redefinir keybindings globales del core,
- errores o timeouts se aíslan por módulo.

### 6.7 Runtime Actions

Runtime actions son mutaciones controladas solicitadas por módulos.

Familias v1:

- query/selection: `setQuery`, `setSelection`, `moveSelection`,
- flujo: `submit`, `close`,
- contenido: `addItems`, `replaceItems`,
- registro: `registerCommand`, `registerProvider`,
- UI primitive: `setInputAccessory`, `clearInputAccessory`.

Reglas:

- toda action se valida antes de modificar estado,
- el core puede rechazar actions por estado inválido o permiso insuficiente,
- no existe mutación directa del estado interno.

---

## 7. Límites explícitos para módulos

Un módulo no puede:

- dibujar UI directamente,
- acceder a Win32/GDI,
- modificar el layout global,
- reemplazar el ranking engine,
- mutar estado arbitrariamente,
- saltarse capabilities,
- interceptar el event loop del core,
- acceder a estructuras internas,
- depender del orden físico de memoria o internals Rust,
- romper la operación del launcher si falla,
- bloquear la UI con I/O lento,
- definir primitivas visuales nuevas por su cuenta.

Un módulo solo puede operar mediante:

- manifest/metadata,
- capabilities,
- hooks oficiales,
- providers,
- commands,
- items normalizados,
- decorations,
- input accessory,
- runtime actions permitidas.

---

## 8. Contratos públicos v1

Los contratos públicos v1 están definidos en:

- `MODULES_API_SPEC_V1.md` — hooks, ctx, items y restricciones.
- `RMOD_SPEC_V1.md` — formato `.rmod` single-file.
- `MANIFEST_SPEC_V1.md` — formato `module.toml`.
- `CTX_ACTIONS_SPEC_V1.md` — fachada `ctx` y mutaciones permitidas.
- `MODULES_CAPABILITIES_MATRIX.md` — capabilities y enforcement.
- `PROVIDER_EXECUTION_POLICY.md` — budget, timeout, merge y dedupe.
- `ERROR_ISOLATION_POLICY.md` — fallos, estados y recuperación.
- `MODULES_AUTHORING_GUIDE.md` — guía para autores.
- `MODULES_OPERATIONS_GUIDE.md` — operación y debugging.
- `MODULES_QUICKSTART.md` — instalación, desarrollo y diagnóstico rápido.
- `DECISIONS.md` — decisiones arquitectónicas aceptadas.

Estos documentos forman el contrato conceptual v1. Si hay conflicto entre ellos, este documento define la intención arquitectónica y las specs específicas definen el detalle operativo.

---

## 9. Política de estabilidad API v1

La API v1 debe mantenerse estable.

Permitido sin bump de API:

- aclarar documentación,
- corregir bugs,
- endurecer validaciones sin romper casos válidos,
- agregar campos opcionales ignorables,
- mejorar telemetría,
- mejorar performance,
- mejorar mensajes de error,
- ampliar tests.

No permitido sin bump de API:

- cambiar significado de hooks existentes,
- remover campos públicos,
- cambiar nombres de capabilities,
- cambiar formato obligatorio de `.rmod`,
- cambiar semántica de command routing,
- permitir a módulos saltarse policies del core,
- exponer internals como API pública.

---

## 10. Core Change Policy

Después del freeze v1, el core solo acepta cambios por una de estas causas:

1. Bug crítico.
2. Crash.
3. Seguridad o aislamiento.
4. Performance medible.
5. Compatibilidad Windows.
6. Corrección del contrato v1.
7. Necesidad general demostrada por varios módulos reales.

No son razones suficientes:

- “sería cómodo”,
- “este módulo lo necesita”,
- “queda más lindo en el core”,
- “es más rápido hardcodearlo”,
- “quizá algún día haga falta”.

Antes de tocar core, responder:

1. ¿Puede implementarse como módulo?
2. ¿Puede resolverse con una primitiva existente?
3. ¿Es una necesidad general o un caso específico?
4. ¿Hay al menos dos o tres módulos reales afectados?
5. ¿La solución amplía el lenguaje público o solo agrega ergonomía?
6. ¿Se puede resolver con documentación o template?

Si la respuesta no justifica core, debe ser módulo.

---

## 11. Criterios para aceptar una nueva primitiva

Una nueva primitiva solo puede agregarse si cumple todo:

- no puede expresarse con providers, commands, decorations, input accessory, capabilities, key hooks o runtime actions,
- desbloquea múltiples módulos reales,
- tiene semántica pequeña y estable,
- no expone internals,
- no permite renderizado arbitrario,
- tiene policy de error/timeout si ejecuta lógica,
- tiene spec pública,
- tiene tests,
- tiene guía de authoring,
- tiene path de compatibilidad o bump de API.

---

## 12. Rechazo de features hacia módulos

Una feature debe mandarse a módulo si:

- depende de un workflow específico,
- integra una herramienta externa concreta,
- agrega una fuente opcional de datos,
- agrega un comando especializado,
- agrega una transformación de texto específica,
- agrega automatización local,
- solo beneficia a un perfil de usuario,
- se puede expresar como provider/command/decorator.

---

## 13. Breaking changes y API v2

Una API v2 solo se propone cuando v1 impide una capacidad general real.

Proceso mínimo:

1. Documentar fricción con ejemplos de módulos reales.
2. Probar alternativas usando primitivas v1.
3. Escribir decisión en `DECISIONS.md` o documento equivalente.
4. Definir spec v2.
5. Definir compatibilidad o migración.
6. Mantener v1 mientras sea razonable, salvo decisión explícita.

---

## 14. Deprecación

Una API pública no se elimina silenciosamente.

Proceso:

1. Marcar como deprecated en spec.
2. Explicar reemplazo.
3. Mantener compatibilidad por una ventana razonable.
4. Documentar remoción en changelog/decisión.
5. Remover solo con bump de API si rompe módulos existentes.

---

## 15. Política documental

Ubicación oficial de documentación pública v1: root del repo.

Documentos históricos, reportes, snapshots y razonamiento exploratorio deben vivir en:

- `extras/private-docs/`, o
- `docs/audits/` para auditorías fechadas.

El root debe reservarse para:

- README,
- specs oficiales,
- guías oficiales,
- checklist de cierre,
- políticas públicas del sistema.

---

## 16. Definition of Done del core v1

El core se considera cerrado cuando:

- esta arquitectura está documentada,
- el vocabulario v1 está congelado,
- las specs públicas no se contradicen,
- el loader `.rmod` y directorio están estables,
- el runtime externo aísla errores,
- capabilities se enforcean,
- providers/commands/decorations/input accessory funcionan dentro de policy,
- hay módulos reales que validan la API sin tocar core,
- tests y verificaciones principales pasan,
- y la política de cambios futuros está aceptada.

---

## 17. Declaración final

El core no debe crecer por acumulación de features.

El core debe proteger:

- estabilidad,
- velocidad,
- coherencia,
- contrato público,
- y seguridad operacional.

La expansión funcional de `rmenu` debe ocurrir primero mediante módulos.
