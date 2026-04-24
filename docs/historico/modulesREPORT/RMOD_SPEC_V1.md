# RMOD SPEC V1

Estado: Draft v1  
Fecha: 2026-04-22

---

## 1) Objetivo

Definir el formato single-file `.rmod` para módulos de `rmenu`.

Regla principal:

- `.rmod` **siempre** es texto plano legible (no empaquetado binario).
- Si en el futuro existe un formato empaquetado, debe usar otra extensión.

---

## 2) Descubrimiento

En `modules/`, el loader soporta:

1. Carpeta con `module.toml` (modo directorio)
2. Archivo `*.rmod` (modo single-file)

Ambos deben normalizarse al mismo descriptor interno (`ModuleDescriptor`).

---

## 3) Encoding y formato base

- Encoding: `UTF-8` (BOM opcional tolerado, recomendado sin BOM)
- Saltos de línea: `\n` o `\r\n`
- Primera línea obligatoria: `#!rmod/v1`

Estructura:

1. Shebang de formato
2. Header key-value (una línea por clave)
3. Línea en blanco
4. Bloques nombrados `---<nombre>---`

---

## 4) Header v1

Formato por línea:

`key: value`

### Claves obligatorias

- `name`
- `version`
- `api_version`
- `kind`
- `capabilities`

### Claves opcionales

- `enabled`
- `priority`
- `description`
- `author`
- `homepage`

### Reglas

- Claves desconocidas: warning (no error) en v1.
- `api_version` debe ser numérico y soportado por runtime.
- `capabilities` es lista CSV (`a,b,c`) sin espacios obligatorios.
- `kind` v1 permitido: `script`.

---

## 5) Bloques

Bloques delimitados por:

`---<block-name>---`

### Bloques v1

- `module.js` (**obligatorio**)
- `config.json` (opcional)
- `readme.md` (opcional)
- `signature` (opcional, reservado)

### Reglas de bloques

- `module.js` debe existir exactamente una vez.
- Bloques duplicados: error.
- Bloques desconocidos: warning (no error) en v1.
- El contenido de cada bloque se toma literal hasta el siguiente delimitador o EOF.

---

## 6) Normalización interna

El parser `.rmod` debe producir el mismo contrato interno que el loader por carpeta.

```ts
type ModuleDescriptor = {
  sourceType: "directory" | "rmod"
  sourcePath: string

  name: string
  version: string
  apiVersion: number
  kind: "script"
  capabilities: string[]

  enabled: boolean
  priority: number
  description?: string
  author?: string
  homepage?: string

  entryCode: string
  configJson?: string
  readme?: string
}
```

---

## 7) Errores mínimos recomendados

- `RMOD_E_INVALID_MAGIC`
- `RMOD_E_HEADER_MALFORMED`
- `RMOD_E_MISSING_REQUIRED_HEADER`
- `RMOD_E_INVALID_API_VERSION`
- `RMOD_E_DUPLICATE_BLOCK`
- `RMOD_E_MISSING_MODULE_JS`
- `RMOD_E_CONFIG_NOT_JSON`

---

## 8) Ejemplo mínimo válido

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
    name: "hello-module",
    onLoad(ctx) {
      ctx.toast("hello-module loaded");
    }
  };
}
```
