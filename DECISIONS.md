# DECISIONS — rmenu

Registro corto de decisiones arquitectónicas activas.

Las decisiones históricas o exploratorias pueden vivir en `extras/private-docs/`. Este archivo resume decisiones vigentes que afectan el contrato público.

---

## DEC-001 — Core modular v1 congelado por primitivas pequeñas

Estado: Accepted  
Fecha: 2026-04-24

### Contexto

`rmenu` evolucionó de launcher nativo a superficie extensible de comandos. Para evitar crecimiento accidental del core, la arquitectura modular necesita una frontera explícita.

### Decisión

El core v1 se estabiliza alrededor de estas primitivas:

- providers,
- commands,
- decorations,
- input accessory,
- capabilities,
- key hooks,
- runtime actions.

Nuevas features deben implementarse primero como módulos. Una nueva primitiva solo se acepta con evidencia desde varios módulos reales y decisión documentada.

### Consecuencias

- El core deja de recibir features específicas.
- La expansión funcional ocurre mediante módulos.
- El contrato público se documenta en specs v1 congeladas.
- Cambios breaking requieren nueva API o decisión explícita.

---

## DEC-002 — `.rmod` v1 es texto plano

Estado: Accepted  
Fecha: 2026-04-24

### Contexto

Los módulos necesitan un formato distribuible y fácil de auditar.

### Decisión

`.rmod` v1 es siempre texto plano UTF-8 con magic `#!rmod/v1`, header key-value y bloques nombrados.

Si en el futuro existe un formato empaquetado/binario, debe usar otra extensión.

### Consecuencias

- Los módulos son legibles y auditables.
- El formato es simple de versionar.
- El loader puede normalizar `.rmod` y directorio al mismo descriptor.

---

## DEC-003 — El core renderiza; los módulos declaran intención

Estado: Accepted  
Fecha: 2026-04-24

### Contexto

Permitir renderizado arbitrario desde módulos comprometería estabilidad, performance y coherencia visual.

### Decisión

Módulos no pueden dibujar UI, acceder a Win32/GDI ni modificar layout global. Solo pueden aportar primitivas declarativas: items, badges, hints, quick-select keys e input accessory.

### Consecuencias

- UI permanece coherente.
- El core conserva control de layout y performance.
- Features visuales complejas deben esperar una primitiva oficial o vivir fuera del core.

---

## DEC-004 — Fail module, not launcher

Estado: Accepted  
Fecha: 2026-04-24

### Contexto

Módulos externos pueden fallar, colgarse o devolver payload inválido.

### Decisión

Errores, timeouts y payloads inválidos se aíslan por módulo/host. El launcher debe continuar operativo.

### Consecuencias

- Runtime aplica timeouts, backoff y disable.
- `--modules-debug` expone telemetría.
- El core prioriza estabilidad sobre completar resultados de módulos defectuosos.
