# CORE CLOSURE CHECKLIST — rmenu

Estado: Draft operativo  
Objetivo: cerrar el core de `rmenu` como plataforma estable y mover la evolución futura hacia módulos.

---

## Principio rector

El core se considera cerrado cuando puede sostener módulos reales sin cambios estructurales frecuentes.

Regla principal:

> Si una funcionalidad puede implementarse como módulo, no entra al core.

Cambios futuros al core solo se aceptan por:

- bug crítico,
- crash,
- seguridad/aislamiento,
- performance,
- compatibilidad Windows,
- corrección de contrato v1,
- necesidad general demostrada por varios módulos reales.

---

# Fase 1 — Documentación, vocabulario y frontera arquitectónica

Esta fase no debería requerir cambios de código.

## 1.1 Documento constitucional de arquitectura

- [x] Crear `MODULES_ARCHITECTURE.md` en el root.
- [x] Definir qué es el core de `rmenu`.
- [x] Definir qué no es el core.
- [x] Definir que el core es autoridad sobre UI, ranking, estado, ejecución y policy.
- [x] Definir que los módulos extienden por composición, no por mutación.
- [x] Definir la frontera entre core, runtime de módulos, module host y módulos de usuario.
- [x] Definir la política de estabilidad de API v1.
- [x] Definir la política para cambios breaking futuros.
- [x] Definir criterios para aceptar una nueva primitiva en el core.
- [x] Definir criterios para rechazar una feature y mandarla a módulo.

## 1.2 Vocabulario oficial v1

- [x] Congelar el vocabulario oficial de módulos v1.
- [x] Documentar `Providers` como primitiva para aportar items.
- [x] Documentar `Commands` como primitiva para acciones invocables.
- [x] Documentar `Decorations` como primitiva para enriquecer items visualmente.
- [x] Documentar `Input Accessory` como primitiva para estado contextual del input.
- [x] Documentar `Capabilities` como sistema de permisos declarativos.
- [x] Documentar `Key Hooks` como manejo controlado de teclas.
- [x] Documentar `Runtime Actions` como mutaciones permitidas del estado.
- [x] Declarar explícitamente que no hay nuevas primitivas v1 salvo necesidad probada.

## 1.3 Límites explícitos para módulos

- [x] Documentar que módulos no pueden dibujar UI directamente.
- [x] Documentar que módulos no pueden acceder a Win32/GDI.
- [x] Documentar que módulos no pueden modificar layout global.
- [x] Documentar que módulos no pueden reemplazar el ranking engine.
- [x] Documentar que módulos no pueden mutar estado arbitrariamente.
- [x] Documentar que módulos no pueden saltarse capabilities.
- [x] Documentar que módulos no pueden depender de internals del core.
- [x] Documentar que módulos deben operar mediante hooks, ctx y actions oficiales.

## 1.4 Congelamiento de specs públicas

- [x] Revisar `MODULES_API_SPEC_V1.md`.
- [x] Marcar `MODULES_API_SPEC_V1.md` como `Frozen v1` o dejar explícito qué falta para freeze.
- [x] Revisar `RMOD_SPEC_V1.md`.
- [x] Marcar `RMOD_SPEC_V1.md` como `Frozen v1` o dejar explícito qué falta para freeze.
- [x] Revisar `MANIFEST_SPEC_V1.md`.
- [x] Marcar `MANIFEST_SPEC_V1.md` como `Frozen v1` o dejar explícito qué falta para freeze.
- [x] Revisar `CTX_ACTIONS_SPEC_V1.md`.
- [x] Confirmar que no contradice `MODULES_API_SPEC_V1.md`.
- [x] Revisar `PROVIDER_EXECUTION_POLICY.md`.
- [x] Confirmar que describe la política real del runtime.
- [x] Revisar `ERROR_ISOLATION_POLICY.md`.
- [x] Confirmar que describe la política real de errores, timeouts y degradación.

## 1.5 Política de evolución futura

- [x] Crear una sección `Core Change Policy` en `MODULES_ARCHITECTURE.md` o archivo equivalente.
- [x] Definir que toda feature nueva debe intentarse primero como módulo.
- [x] Definir que una nueva capability no implica automáticamente nueva primitiva.
- [x] Definir que una nueva primitiva requiere evidencia desde módulos reales.
- [x] Definir que cambios de ergonomía/helpers no justifican tocar core si no desbloquean capacidades generales.
- [x] Definir cómo se propone una extensión de API v2.
- [x] Definir cómo se depreca una API existente.
- [x] Definir cómo se documentan decisiones arquitectónicas futuras.

## 1.6 Limpieza y orden documental

- [x] Decidir ubicación oficial de specs: root o `docs/`.
- [x] Decidir ubicación oficial de documentación histórica: `extras/private-docs/` o `docs/audits/`.
- [x] Eliminar o mover duplicados documentales.
- [x] Confirmar que el root no mezcla specs oficiales con reportes históricos ambiguos.
- [x] Actualizar `README.md` para reflejar que `rmenu` es launcher + plataforma modular.
- [x] Agregar en `README.md` una sección corta sobre módulos.
- [x] Agregar en `README.md` enlaces a specs oficiales.
- [x] Agregar en `README.md` comandos de diagnóstico relevantes: `--metrics`, `--debug-ranking`, `--modules-debug`, `--reindex`.
- [x] Actualizar o crear guía corta: “cómo instalar un módulo”.
- [x] Actualizar o crear guía corta: “cómo desarrollar un módulo”.
- [x] Actualizar o crear guía corta: “cómo debuggear un módulo”.

## 1.7 Madurez de metadata y presentación del proyecto

- [x] Revisar `Cargo.toml`.
- [x] Reemplazar author placeholder.
- [x] Revisar descripción del paquete.
- [x] Confirmar versión actual.
- [x] Confirmar licencia.
- [x] Confirmar que `include` contiene archivos necesarios y no basura.
- [x] Revisar `.gitignore`.
- [x] Confirmar que binarios, caches y artifacts pesados no se trackean.
- [x] Confirmar que ejemplos, templates y docs importantes sí se trackean.

---

# Fase 2 — Validación con módulos reales, sin tocar core

Objetivo: demostrar que el core actual alcanza para construir extensiones útiles.

## 2.1 Módulo real: calculator

- [ ] Crear módulo calculator como `.rmod` o carpeta dev.
- [ ] Detectar queries tipo `=2+2` o similar.
- [ ] Mostrar resultado vía `Input Accessory`.
- [ ] Aportar item de resultado vía `Provider` si corresponde.
- [ ] Ejecutar acción de copiar/usar resultado mediante action oficial.
- [ ] Declarar capabilities mínimas necesarias.
- [ ] Confirmar que no requiere cambios al core.
- [ ] Documentar fricciones encontradas.

## 2.2 Módulo real: scripts/commands

- [ ] Crear módulo que liste scripts o comandos locales.
- [ ] Aportar items vía `Provider`.
- [ ] Usar badges/hints/subtitles para mostrar metadata.
- [ ] Registrar comandos con namespace estable.
- [ ] Ejecutar acción sin acceso directo a internals.
- [ ] Confirmar dedupe y ranking aceptables.
- [ ] Confirmar que no requiere cambios al core.
- [ ] Documentar fricciones encontradas.

## 2.3 Módulo real: hotkeys/quick actions

- [ ] Crear módulo que use quick select keys o key hooks.
- [ ] Declarar capability `keys`.
- [ ] Confirmar que key hooks se deniegan sin capability.
- [ ] Confirmar comportamiento ante duplicados de quick select.
- [ ] Confirmar que warnings/debug son entendibles.
- [ ] Confirmar que no requiere cambios al core.
- [ ] Documentar fricciones encontradas.

## 2.4 Revisión de fricciones

- [ ] Clasificar cada fricción como bug, falta de docs, ergonomía, feature específica o falta real de primitiva.
- [ ] Resolver primero docs antes que código si el problema es ambigüedad.
- [ ] Rechazar cambios de core si solo hacen más cómodo un caso aislado.
- [ ] Aceptar cambios de core solo si desbloquean varios módulos o corrigen contrato.

---

# Fase 3 — Hardening funcional del core existente

Objetivo: cerrar huecos reales sin ampliar superficie conceptual.

## 3.1 Runtime y module host

- [ ] Confirmar que cada módulo externo corre aislado del core.
- [ ] Confirmar que timeout por request funciona.
- [ ] Confirmar que host colgado se mata o degrada correctamente.
- [ ] Confirmar que auto-restart respeta backoff.
- [ ] Confirmar que auto-disable ocurre tras umbrales configurados.
- [ ] Confirmar que reload exitoso resetea contadores relevantes.
- [ ] Confirmar que errores de un módulo no rompen otros módulos.
- [ ] Confirmar que `--modules-debug` muestra estado, errores, telemetría y capacidades.

## 3.2 Loader `.rmod` y directorio

- [ ] Confirmar descubrimiento mixto de módulos por carpeta y `.rmod`.
- [ ] Confirmar normalización común a `ModuleDescriptor`.
- [ ] Confirmar errores claros para `.rmod` inválido.
- [ ] Confirmar errores claros para `module.toml` inválido.
- [ ] Confirmar que módulos deshabilitados no se cargan.
- [ ] Confirmar prioridad determinista.
- [ ] Confirmar hot reload por módulo afectado.
- [ ] Confirmar debounce contra loops de reload.
- [ ] Confirmar comportamiento si un módulo cambia mientras está en uso.

## 3.3 Capabilities y permisos

- [ ] Confirmar que `registerProvider` requiere capability correspondiente.
- [ ] Confirmar que `registerCommand` requiere capability correspondiente.
- [ ] Confirmar que `setInputAccessory` requiere capability correspondiente.
- [ ] Confirmar que `onKey`/key hooks requieren capability correspondiente.
- [ ] Confirmar que violaciones se registran como `permission_denied`.
- [ ] Confirmar que el módulo sigue aislado tras violación de permisos.
- [ ] Confirmar que la matriz capability -> acción está documentada y testeada.

## 3.4 Providers, commands y dedupe

- [ ] Confirmar budget global por query para providers.
- [ ] Confirmar timeout por provider configurable.
- [ ] Confirmar cap de items por provider.
- [ ] Confirmar sanitización de items aportados por provider.
- [ ] Confirmar dedupe determinista.
- [ ] Confirmar política `core-first`.
- [ ] Confirmar política `provider-first`.
- [ ] Confirmar namespace de comandos.
- [ ] Confirmar resolución de colisiones de comandos.
- [ ] Confirmar comandos runtime mínimos: list, reload, telemetry reset.

## 3.5 UI primitives v1

- [ ] Confirmar rendering de badge.
- [ ] Confirmar rendering de badge kind.
- [ ] Confirmar rendering de hint.
- [ ] Confirmar rendering de subtitle/source si aplica.
- [ ] Confirmar rendering de input accessory.
- [ ] Confirmar colores por `InputAccessory.kind`.
- [ ] Confirmar prioridad de input accessory.
- [ ] Confirmar quick select visible.
- [ ] Confirmar conflicto de quick select duplicado: primer visible gana + warning.
- [ ] Confirmar layout en anchos extremos.
- [ ] Confirmar que módulos no pueden dibujar UI custom.

## 3.6 IPC y seguridad operacional

- [ ] Confirmar límite máximo de payload request.
- [ ] Confirmar límite máximo de payload response.
- [ ] Confirmar validación estricta de campos obligatorios.
- [ ] Confirmar límites de longitud para strings.
- [ ] Confirmar rechazo/sanitización de items inválidos.
- [ ] Confirmar que payload inválido no crashea el core.
- [ ] Confirmar que stdout/stderr del host no contamina protocolo si aplica.
- [ ] Confirmar que errores IPC son visibles en debug.

---

# Fase 4 — Tests y verificación

## 4.1 Tests automatizados

- [ ] Tests de parser `.rmod` válido.
- [ ] Tests de parser `.rmod` inválido.
- [ ] Tests de bloques duplicados.
- [ ] Tests de bloques faltantes.
- [ ] Tests de headers obligatorios faltantes.
- [ ] Tests de `module.toml` válido.
- [ ] Tests de `module.toml` inválido.
- [ ] Tests de loader mixto: directorio + `.rmod`.
- [ ] Tests de capabilities permitidas.
- [ ] Tests de capabilities denegadas.
- [ ] Tests de provider timeout.
- [ ] Tests de provider item cap.
- [ ] Tests de dedupe.
- [ ] Tests de command namespace/collision.
- [ ] Tests de IPC payload limit.
- [ ] Tests de hot reload.
- [ ] Tests de host restart/backoff.
- [ ] Tests de auto-disable por errores/timeouts.
- [ ] Tests de quick select duplicado.
- [ ] Tests de input accessory priority/kind.

## 4.2 Comandos de verificación local

- [ ] Ejecutar `cargo fmt`.
- [ ] Ejecutar `cargo check`.
- [ ] Ejecutar `cargo test`.
- [ ] Ejecutar `cargo clippy` si forma parte del flujo del proyecto.
- [ ] Ejecutar `rmenu --metrics`.
- [ ] Ejecutar `rmenu --debug-ranking <query>`.
- [ ] Ejecutar `rmenu --modules-debug`.
- [ ] Ejecutar prueba manual de launcher mode.
- [ ] Ejecutar prueba manual de script/dmenu mode con stdin.
- [ ] Ejecutar prueba manual de script/dmenu mode con `-e`.

## 4.3 Performance mínima aceptable

- [ ] Definir presupuesto objetivo de startup.
- [ ] Definir presupuesto objetivo de search p95.
- [ ] Definir presupuesto objetivo de time-to-window-visible.
- [ ] Definir presupuesto objetivo de provider budget.
- [ ] Confirmar que módulos lentos no degradan severamente la UI.
- [ ] Confirmar que cache de índice se invalida correctamente.
- [ ] Confirmar que `--reindex` funciona.

---

# Fase 5 — Pulido final de producto/core

## 5.1 UX base

- [ ] Confirmar que launcher mode sigue siendo simple y rápido sin módulos.
- [ ] Confirmar que no hay regresión en fuzzy/ranking.
- [ ] Confirmar labels amigables para apps.
- [ ] Confirmar búsqueda por nombre técnico del ejecutable.
- [ ] Confirmar launch vía ShellExecuteW.
- [ ] Confirmar fallback controlado de launch.
- [ ] Confirmar historial persistente.
- [ ] Confirmar blacklist/source boosts desde config.

## 5.2 Configuración

- [ ] Revisar `config_example.ini` completo.
- [ ] Documentar sección `[Modules]`.
- [ ] Documentar timeout, budget, item caps y payload limits.
- [ ] Documentar política de dedupe.
- [ ] Documentar hot reload si es configurable.
- [ ] Confirmar defaults seguros.
- [ ] Confirmar que config inválida no crashea.

## 5.3 Distribución de módulos

- [ ] Definir ubicación estándar local de módulos.
- [ ] Definir estructura recomendada para módulos de ejemplo.
- [ ] Definir convención de nombres.
- [ ] Definir convención de capabilities.
- [ ] Definir convención de comandos namespaced.
- [ ] Definir cómo compartir `.rmod`.
- [ ] Definir cómo versionar módulos.

---

# Fase 6 — Declaración de freeze

## 6.1 Checklist final antes de freeze

- [ ] `MODULES_ARCHITECTURE.md` existe y está aprobado.
- [ ] Specs v1 están revisadas y congeladas.
- [ ] README refleja arquitectura actual.
- [ ] Docs duplicadas o históricas están ordenadas.
- [ ] Metadata del proyecto está limpia.
- [ ] Tres módulos reales funcionan sin tocar core.
- [ ] Fricciones de módulos están clasificadas.
- [ ] Bugs bloqueantes están corregidos.
- [ ] Tests/verificaciones están verdes.
- [ ] Performance mínima está validada.
- [ ] Política de cambios futuros al core está escrita.

## 6.2 Declaración formal

- [ ] Crear nota `CORE_FREEZE_V1.md` o sección equivalente en `MODULES_ARCHITECTURE.md`.
- [ ] Declarar fecha de freeze.
- [ ] Declarar alcance del core congelado.
- [ ] Declarar API de módulos congelada.
- [ ] Declarar qué tipos de cambios al core siguen permitidos.
- [ ] Declarar que nuevas features deben implementarse primero como módulos.

---

# Definition of Done — Core Cerrado v1

El core de `rmenu` se considera cerrado cuando:

- la frontera core/módulos está documentada,
- el vocabulario v1 está congelado,
- las specs públicas no se contradicen,
- existen módulos reales que validan la API,
- el runtime externo es robusto ante errores,
- capabilities se enforcean,
- UI primitives v1 son suficientes para módulos útiles,
- tests y checks principales pasan,
- el repo está ordenado,
- y existe una política explícita para impedir expansión accidental del core.

A partir de ese punto, el trabajo principal pasa a ser:

- nuevos módulos,
- mejores módulos,
- templates,
- documentación para autores,
- polish visual dentro de primitivas existentes,
- bugfixes,
- performance,
- y evolución futura controlada.
