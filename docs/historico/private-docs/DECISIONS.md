# Modules — Decisions (mini ADRs)

Registro corto de decisiones arquitectónicas sobre extensiones/módulos.

---

## ADR-MOD-001 — Autoridad del core
**Status:** Accepted  
**Date:** 2026-04-22

### Decisión
El core conserva autoridad sobre input, estado, ranking, submit, render y lifecycle.

### Consecuencia
Los módulos no acceden a internals arbitrarios ni al renderer.

---

## ADR-MOD-002 — Extensión por composición
**Status:** Accepted  
**Date:** 2026-04-22

### Decisión
La extensibilidad se implementa con hooks + ctx controlado + actions permitidas + providers.

### Consecuencia
Se evita mutación directa de estado interno y se mantiene API pequeña/estable.

---

## ADR-MOD-003 — Visual metadata only
**Status:** Accepted  
**Date:** 2026-04-22

### Decisión
Los módulos pueden declarar metadata visual/semántica (`capabilities`, `decorations`) pero no dibujar.

### Consecuencia
UI consistente, desacoplada del backend Win32/GDI y más fácil de evolucionar.

---

## ADR-MOD-004 — QuickSelect como capability oficial
**Status:** Proposed  
**Date:** 2026-04-22

### Decisión
`quickSelectKey` debe ser capability oficial del item.

### Consecuencia esperada
Core renderiza badge y resuelve input numérico; módulos solo asignan metadata.

---

## ADR-MOD-005 — Input Accessory como primitive UI v1
**Status:** Accepted  
**Date:** 2026-04-22

### Decisión
Agregar primitive `Input Accessory` (`ctx.setInputAccessory` / `ctx.clearInputAccessory`) con prioridad y render centralizado.

### Consecuencia
Se habilitan previews inline (calc/convert/command hint) sin exponer render internals.

---

## ADR-MOD-006 — Hot reload simple primero
**Status:** Accepted  
**Date:** 2026-04-22

### Decisión
v1 con reload simple: unload + reload + re-register, sin migración compleja de estado.

### Consecuencia
Menor complejidad inicial y menor riesgo operativo.

---

## Template para nuevas decisiones

```md
## ADR-MOD-XXX — <título>
**Status:** Proposed | Accepted | Superseded
**Date:** YYYY-MM-DD

### Contexto
...

### Decisión
...

### Consecuencia
...

### Alternativas consideradas
- ...
```
