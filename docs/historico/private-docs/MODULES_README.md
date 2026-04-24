# Modules Documentation

Carpeta viva para documentación de arquitectura modular/extensiones de `rmenu`.

## Objetivo

Centralizar:
- estrategia arquitectónica de módulos,
- plan de implementación por fases,
- primitivas de UI extensible,
- decisiones futuras (sin mezclar con el plan de producto general del launcher).

## Archivos

- `MODULES_IMPLEMENTATION_PLAN.md` → master plan detallado (roadmap, fases, tareas, criterios de done y protocolo de reinicio de chat).
- `extensions_implementation_plan.md` → plan modular resumido/estratégico.
- `MODULES_API_SPEC_V1.md` → contrato público v1 de hooks/ctx/actions/items.
- `MANIFEST_SPEC_V1.md` → contrato de `module.toml`.
- `CTX_ACTIONS_SPEC_V1.md` → comportamiento exacto de `ctx` y actions permitidas.
- `PROVIDER_EXECUTION_POLICY.md` → política de ejecución/timeout/budget para providers.
- `ERROR_ISOLATION_POLICY.md` → política de aislamiento y degradación por módulo.
- `RMOD_SPEC_V1.md` → especificación del formato single-file `.rmod` (texto plano).
- `PRIMITIVE_TEMPLATE.md` → plantilla para nuevas primitivas UI/core.
- `item_trailing_accessories_primitive.md` → spec de layout por zonas para coexistencia `label + hint + shortcut chip`.
- `templates/rmod/starter-module.rmod` → template descargable para arrancar módulos `.rmod`.
- `templates/directory/*` → template en formato carpeta para desarrollo iterativo.
- `MODULES_EXECUTION_CHECKLIST.md` → checklist operativo por batches para cierre de implementación.
- `MODULES_OPERATIONS_GUIDE.md` → guía operativa de debug/estado/comandos del runtime modular.
- `MODULES_AUTHORING_GUIDE.md` → guía oficial para crear módulos (`.rmod` y carpeta).
- `MODULES_CAPABILITIES_MATRIX.md` → matriz final capability -> permisos/hooks/actions.
- `MODULES_RELEASE_CHECKLIST.md` → checklist final release-ready del sistema modular.
- `CHANGELOG.md` → evolución de la documentación modular.
- `DECISIONS.md` → mini ADRs de arquitectura modular.

## Fuentes base usadas

Este plan consolida y normaliza contenido de:
- `extras/modulesREPORTE/reporte_base.md`
- `extras/modulesREPORTE/modules_architecture.md`
- `extras/modulesREPORTE/ui_extension_primitive.md`

## Convención para agregar más docs

Cuando agregues una nueva extensión/primitiva:

1. Crear archivo nuevo en esta carpeta (ej. `ui_footer_primitive.md`).
2. Seguir `PRIMITIVE_TEMPLATE.md`.
3. Actualizar `extensions_implementation_plan.md` en:
   - sección de "Primitivas UI",
   - roadmap/checklist,
   - riesgos/aceptación si corresponde.

Mantener siempre la regla:

**Módulo declara intención. Core renderiza/ejecuta realidad.**
