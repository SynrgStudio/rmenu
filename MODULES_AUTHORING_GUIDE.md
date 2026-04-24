# MODULES AUTHORING GUIDE v1

Estado: Frozen v1

Guía oficial para crear módulos `rmenu` en formato `.rmod` y formato carpeta.

---

## 1. Leer primero

Orden recomendado:

1. `MODULES_ARCHITECTURE.md`
2. `MODULES_QUICKSTART.md`
3. `MODULES_API_SPEC_V1.md`
4. `RMOD_SPEC_V1.md` o `MANIFEST_SPEC_V1.md`
5. `MODULES_CAPABILITIES_MATRIX.md`
6. `MODULES_OPERATIONS_GUIDE.md`

Regla central:

> Módulo declara intención; core valida, ejecuta y renderiza realidad.

---

## 2. Elegir formato

### `.rmod` — recomendado para distribución

- Un solo archivo.
- Fácil de compartir.
- Fácil de versionar.
- No requiere carpeta auxiliar.

### Carpeta — recomendado para desarrollo

- Edición cómoda de `module.js`.
- Manifest separado.
- `config.json` y `README.md` simples.
- Mejor para hot reload durante iteración.

---

## 3. Capabilities

Declarar solo lo necesario.

Capabilities v1:

- `providers`
- `commands`
- `decorate-items`
- `input-accessory`
- `keys`

Si un módulo intenta usar una operación sin capability, el runtime bloquea la operación y registra `permission_denied`.

---

## 4. Estructura mínima `.rmod`

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
        id: "hello-item",
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

Bloques opcionales:

- `---config.json---`
- `---readme.md---`

---

## 5. Estructura mínima en carpeta

```text
modules/
  hello-module/
    module.toml
    module.js
```

`module.toml`:

```toml
name = "hello-module"
version = "0.1.0"
api_version = 1
kind = "script"
entry = "module.js"
capabilities = ["providers"]
enabled = true
priority = 0
```

`module.js` usa el mismo patrón que el bloque `module.js` del `.rmod`.

---

## 6. Modelo de item público

```ts
type Item = {
  id: string
  title: string
  subtitle?: string
  source?: string
  target?: string
  quickSelectKey?: string
  badge?: string
  hint?: string
}
```

Reglas:

- `id` y `title` son obligatorios.
- `target` indica destino ejecutable/lanzable.
- Campos largos o inválidos pueden ser truncados o descartados.
- El core decide merge, dedupe, ranking y render final.

---

## 7. Hooks principales

### Provider

```js
provideItems(query, ctx) {
  return [];
}
```

Requiere `providers`.

### Decorator

```js
decorateItems(items, ctx) {
  return items;
}
```

Requiere `decorate-items`.

### Command

```js
onCommand(command, args, ctx) {
}
```

Requiere `commands`.

### Key hook

```js
onKey(event, ctx) {
}
```

Requiere `keys`.

---

## 8. Comandos y namespacing

Formato recomendado:

```text
/modulo::comando
```

Reglas:

- alias sin namespace se permite solo si no hay colisión,
- si hay colisión, usar namespace explícito,
- nombres vacíos o ambiguos se rechazan.

---

## 9. Quick select y decoraciones

`quickSelectKey` debe ser una tecla visible simple: `1` a `9` o `0`.

Reglas:

- el core resuelve conflictos,
- el primer item visible gana,
- duplicados posteriores pueden perder key/badge,
- `hint` y `badge` pueden truncarse por layout.

---

## 10. Reglas de diseño para autores

1. Hooks rápidos y deterministas.
2. No bloquear I/O sin timeout propio.
3. Providers deben devolver pocos items relevantes.
4. No asumir control de UI o píxeles.
5. Normalizar inputs internos antes de procesar.
6. Tratar errores como recuperables.
7. Loggear contexto útil.
8. No depender de internals del core.
9. No declarar capabilities “por si acaso”.
10. Probar con `--modules-debug`.

---

## 11. Validación y seguridad IPC

El runtime puede recortar, normalizar o descartar:

- campos obligatorios vacíos,
- strings fuera de límites,
- quick select inválido,
- payload excesivo,
- valores no válidos.

Autores no deben depender de payload sin filtrar.

---

## 12. Workflow recomendado

1. Crear módulo en carpeta.
2. Declarar metadata mínima.
3. Declarar capabilities mínimas.
4. Implementar un hook.
5. Ejecutar `rmenu.exe --modules-debug`.
6. Probar comportamiento en launcher.
7. Revisar errores recientes.
8. Iterar con `/modules.reload`.
9. Congelar versión.
10. Empaquetar como `.rmod` para distribuir.
