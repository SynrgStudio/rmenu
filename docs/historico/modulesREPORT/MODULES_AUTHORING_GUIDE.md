# Modules Authoring Guide (v1)

Guía oficial para crear módulos `rmenu` en formato `.rmod` (single-file) y formato carpeta (`module.toml` + `module.js`).

---

## 1) Elegir formato

### `.rmod` (recomendado para distribución)

Ventajas:
- un solo archivo,
- fácil de compartir/versionar,
- parseo directo por el loader.

### Carpeta (recomendado para desarrollo iterativo)

Ventajas:
- edición incremental de `module.js` y `module.toml`,
- estructura más cómoda para módulos medianos/grandes,
- más simple para debugging local.

---

## 2) Contratos obligatorios

Leer y respetar:
- `RMOD_SPEC_V1.md`
- `MANIFEST_SPEC_V1.md`
- `MODULES_API_SPEC_V1.md`
- `CTX_ACTIONS_SPEC_V1.md`

Regla: **módulo declara intención; core ejecuta/renderiza realidad**.

---

## 3) Capacidades (`capabilities`) y permisos

Declarar solo lo necesario.

Ejemplos comunes:
- `providers`
- `commands`
- `decorate-items`
- `input-accessory`
- `keys`

Si un módulo intenta usar una operación sin capability declarada, el runtime bloquea y registra `permission_denied`.

---

## 4) Estructura mínima `.rmod`

```txt
#!rmod/v1
name: hello-module
version: 0.1.0
api_version: 1
kind: script
capabilities: commands

---module.js---
export default function createModule() {
  return {
    onLoad(ctx) {
      ctx.registerCommand({ name: "hello", description: "demo" });
    },
    onCommand(command, args, ctx) {
      if (command === "hello") {
        ctx.toast("hello from module");
      }
    }
  };
}
```

Opcionales de `.rmod`:
- `---config.json---`
- `---readme.md---`

---

## 5) Estructura mínima en carpeta

Archivos:
- `module.toml`
- `module.js`
- opcional: `config.json`, `README.md`

`module.toml` mínimo:

```toml
name = "hello-module"
version = "0.1.0"
api_version = 1
kind = "script"
capabilities = ["commands"]
enabled = true
priority = 0
```

`module.js` mínimo: mismo patrón que `.rmod`.

---

## 6) Reglas de diseño para autores

1. Hooks rápidos y deterministas.
2. No bloquear I/O sin timeout.
3. Providers deben devolver pocos items relevantes.
4. No asumir control de UI/píxeles.
5. Normalizar inputs internos (query/args) antes de procesar.
6. Tratar errores como recuperables y loggear contexto.

---

## 7) Comandos y namespacing

- Formato explícito recomendado: `/modulo::comando`.
- Alias sin namespace se permite solo si no hay colisión de owners.
- Si hay colisión, el runtime exige namespace.

---

## 8) Quick-select e items decorados

- `quickSelectKey` esperado: `"1".."9"|"0"`.
- El core resuelve conflictos (primer visible gana).
- `hint` y `badge` pueden ser truncados/ocultados por layout.

---

## 9) Validación y seguridad IPC

El runtime puede recortar/normalizar/dropear items IPC inválidos:
- campos obligatorios vacíos,
- strings fuera de límites,
- valores no válidos en campos semánticos.

Autores no deben depender de payload "sin filtrar".

---

## 10) Workflow recomendado de authoring

1. Crear módulo con template (`templates/rmod` o `templates/directory`).
2. Probar carga con `rmenu --modules-debug`.
3. Verificar capabilities y `permission_denied`.
4. Probar `/modules.reload` durante iteración.
5. Revisar `host_telemetry` y `recent_errors`.
6. Congelar versión y empaquetar `.rmod` para distribución.
