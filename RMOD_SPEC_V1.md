# RMOD SPEC V1

Estado: Frozen v1  
Fecha: 2026-04-24

---

## 1. Objetivo

Definir el formato single-file `.rmod` para módulos de `rmenu`.

Regla principal:

> `.rmod` siempre es texto plano legible.

Si en el futuro existe un formato empaquetado/binario, debe usar otra extensión.

---

## 2. Descubrimiento

El loader soporta dos formas en `modules/`:

1. Carpeta con `module.toml`.
2. Archivo `*.rmod`.

Ambas se normalizan al mismo descriptor interno.

---

## 3. Encoding y formato base

- Encoding: UTF-8.
- BOM: tolerado, no recomendado.
- Saltos de línea: `\n` o `\r\n`.
- Primera línea obligatoria: `#!rmod/v1`.

Estructura:

1. Shebang de formato.
2. Header key-value.
3. Línea en blanco.
4. Bloques nombrados `---<nombre>---`.

---

## 4. Header v1

Formato:

```text
key: value
```

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

- Claves desconocidas: warning o ignoradas en v1.
- `api_version` debe ser numérico y soportado por runtime.
- `kind` v1 permitido: `script`.
- `capabilities` es CSV: `providers,commands`.
- Espacios alrededor de comas en `capabilities` deben tolerarse.
- `enabled` default: `true`.
- `priority` default: `0`.

---

## 5. Bloques v1

Bloques delimitados por:

```text
---<block-name>---
```

### Bloques soportados

- `module.js` — obligatorio.
- `config.json` — opcional.
- `readme.md` — opcional.
- `signature` — opcional, reservado.

### Reglas

- `module.js` debe existir exactamente una vez.
- Bloques duplicados son error.
- Bloques desconocidos son warning o ignorados en v1.
- El contenido se toma literal hasta el siguiente delimitador o EOF.
- `config.json`, si existe, debe ser JSON válido.

---

## 6. Descriptor normalizado

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

## 7. Errores v1

Errores mínimos recomendados:

- `RMOD_E_INVALID_MAGIC`
- `RMOD_E_HEADER_MALFORMED`
- `RMOD_E_MISSING_REQUIRED_HEADER`
- `RMOD_E_INVALID_API_VERSION`
- `RMOD_E_DUPLICATE_BLOCK`
- `RMOD_E_MISSING_MODULE_JS`
- `RMOD_E_CONFIG_NOT_JSON`

El loader puede agregar contexto humano al error, pero el código estable debe mantenerse para debugging y tests.

---

## 8. Ejemplo mínimo válido

```txt
#!rmod/v1
name: hello-module
version: 0.1.0
api_version: 1
kind: script
capabilities: providers

---module.js---
export default function createModule() {
  return {
    provideItems(query, ctx) {
      if (!query.startsWith("hello")) return [];
      return [{
        id: "hello",
        title: "Hello from module",
        subtitle: "Example provider item",
        source: "hello-module",
        target: "notepad.exe",
        badge: "demo"
      }];
    }
  };
}
```

---

## 9. Ejemplo con config y readme

```txt
#!rmod/v1
name: configured-module
version: 0.1.0
api_version: 1
kind: script
capabilities: providers,decorate-items
priority: 10
description: Example configured module

---config.json---
{
  "prefix": "cfg"
}

---readme.md---
# configured-module

Example module with config.

---module.js---
export default function createModule() {
  return {
    provideItems(query, ctx) {
      return [];
    }
  };
}
```

---

## 10. Compatibilidad futura

- `.rmod/v1` queda estable.
- Cambios incompatibles requieren nuevo magic, por ejemplo `#!rmod/v2`.
- Nuevos bloques opcionales pueden agregarse si son ignorables por runtimes v1.
- El formato debe seguir siendo legible como texto plano.
