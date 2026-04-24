# modulesREPORT — Changelog

Este changelog registra cambios de especificación/arquitectura modular.

## [Unreleased]

### Added
- Estructura documental modular separada en `modulesREPORT/`.
- Plan maestro: `extensions_implementation_plan.md`.
- Plantilla de primitivas: `PRIMITIVE_TEMPLATE.md`.
- Primitive documentada: Input Accessory / Inline Query Preview (consolidada en plan).
- Nueva primitive: `item_trailing_accessories_primitive.md` (layout por zonas para `label + hint + quick shortcut chip`).
- Especificación `RMOD_SPEC_V1.md` para formato single-file `.rmod` (texto plano, bloques explícitos, normalización a descriptor interno).
- Template descargable `templates/rmod/starter-module.rmod` listo para edición/uso.
- Guía de template `templates/rmod/README.md`.
- Template espejo en formato carpeta: `templates/directory/{module.toml,index.js,config.json,README.md}`.
- Checklist operativo por batches: `MODULES_EXECUTION_CHECKLIST.md`.
- Guía operativa de módulos: `MODULES_OPERATIONS_GUIDE.md`.

### Changed
- Se establece separación formal entre:
  - plan operativo de producto (`IMPLEMENTATION_PLAN.md`), y
  - arquitectura modular extensible (`modulesREPORT/*`).

### Notes
- Esta documentación es fuente de diseño para futuras implementaciones en core.
- Al implementar, registrar en este changelog el impacto de contrato/API.
