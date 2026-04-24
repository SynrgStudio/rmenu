# MODULES API SPEC V1

Estado: Frozen v1  
Fecha: 2026-04-24

---

## 1. Objetivo

Definir el contrato público mínimo para módulos de `rmenu`.

Regla:

> API chica, estable, sin exposición de internals.

La intención de v1 es permitir módulos útiles sin permitir que un módulo controle el core, el renderer o el event loop.

---

## 2. Versionado

- Versión actual: `api_version = 1`.
- Todo módulo debe declarar `api_version = 1` en `.rmod` o `module.toml`.
- Cambios breaking requieren una nueva versión de API.
- Extensiones aditivas pueden incorporarse a v1 solo si son opcionales e ignorables por módulos existentes.

---

## 3. Formas de distribución

Un módulo puede distribuirse como:

1. Carpeta de desarrollo con `module.toml` + entry JS.
2. Archivo single-file `.rmod`.

Ambos formatos se normalizan al mismo descriptor interno.

---

## 4. Hooks v1

Hooks públicos:

```ts
onLoad(ctx)
onUnload(ctx)
onQueryChange(query, ctx)
onSelectionChange(item, index, ctx)
onKey(event, ctx)
onSubmit(item, ctx)
onCommand(command, args, ctx)
provideItems(query, ctx) -> Item[]
decorateItems(items, ctx) -> Item[]
```

### Reglas

- Hooks deben ser rápidos y deterministas.
- Hooks no deben bloquear el loop de UI.
- Errores se aíslan por módulo.
- El core puede no invocar hooks que no correspondan a capabilities declaradas.
- El core puede descartar respuestas stale, lentas o inválidas.

---

## 5. Context (`ctx`) v1

`ctx` es una fachada controlada. No expone referencias mutables al estado interno.

### Lectura

```ts
ctx.query() -> string
ctx.items() -> Item[]
ctx.selectedItem() -> Item | null
ctx.selectedIndex() -> number
ctx.mode() -> "launcher" | "stdin" | "command"
ctx.hasCapability(name: string) -> boolean
```

### Utilidades

```ts
ctx.log(message: string)
ctx.toast(message: string)
```

### Mutación permitida mediante actions

```ts
ctx.setQuery(text)
ctx.setSelection(index)
ctx.moveSelection(offset)
ctx.submit()
ctx.close()
ctx.addItems(items)
ctx.replaceItems(items)
ctx.registerCommand(def)
ctx.registerProvider(def)
ctx.setInputAccessory(accessory)
ctx.clearInputAccessory()
```

### Reglas

- Toda mutación es una solicitud, no una modificación directa.
- El core valida estado, payload y permisos.
- El core puede rechazar una action y registrar error.
- La semántica detallada está en `CTX_ACTIONS_SPEC_V1.md`.

---

## 6. Item v1

Modelo conceptual:

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

### Campos

- `id`: identificador estable del item dentro de su fuente.
- `title`: texto principal visible.
- `subtitle`: detalle opcional.
- `source`: fuente visible o lógica.
- `target`: destino a ejecutar si el item representa launch directo.
- `quickSelectKey`: tecla rápida visible (`"1".."9"|"0"`).
- `badge`: texto corto lateral.
- `hint`: ayuda contextual.

### Normalización interna

El core puede convertir el item público a un modelo interno con action explícita.

Ejemplo conceptual:

```ts
target -> { action: "launchTarget", target }
```

### Reglas

- `id` y `title` son obligatorios.
- Campos inválidos pueden ser recortados, normalizados o descartados.
- Items sin acción ejecutable pueden comportarse como `noop`.
- El core decide render, ranking final y dedupe.

---

## 7. Input Accessory v1

```ts
type InputAccessory = {
  text: string
  kind?: "info" | "success" | "warning" | "error" | "hint"
  priority?: number
}
```

Reglas:

- Requiere capability `input-accessory`.
- Solo un accessory visible a la vez.
- Mayor prioridad gana.
- El core decide colores, posición, truncado y layout.

---

## 8. Commands v1

```ts
type CommandDef = {
  name: string
  description?: string
}
```

Reglas:

- Requiere capability `commands`.
- Nombre recomendado: `/modulo::comando`.
- Alias sin namespace se permite solo si no hay colisión.
- Colisiones se rechazan de forma determinista.

---

## 9. Providers v1

```ts
type ProviderDef = {
  name: string
  priority?: number
}
```

Reglas:

- Requiere capability `providers`.
- Respuestas sujetas a budget global por query.
- Respuestas sujetas a timeout por host/provider.
- Respuestas sujetas a cap máximo de items.
- El core hace merge, dedupe y ranking final.

---

## 10. Key Event v1

```ts
type KeyEvent = {
  key: string
  ctrl: boolean
  alt: boolean
  shift: boolean
  meta: boolean
}
```

Reglas:

- Requiere capability `keys`.
- No reemplaza el sistema de keybindings del core.
- No permite interceptar el event loop completo.

---

## 11. Capabilities v1

Capabilities oficiales:

- `providers`
- `commands`
- `decorate-items`
- `input-accessory`
- `keys`

El runtime debe denegar operaciones no declaradas.

Ver `MODULES_CAPABILITIES_MATRIX.md`.

---

## 12. Restricciones explícitas

Módulos no pueden:

- dibujar UI directamente,
- acceder a Win32/GDI,
- cambiar layout global,
- mutar estado interno arbitrario,
- modificar ranking internamente,
- interceptar event loop del core,
- saltarse capabilities,
- depender de internals del core.

---

## 13. Errores y aislamiento

- Error de módulo no debe tumbar launcher.
- Hook/provider defectuoso puede degradarse o deshabilitarse.
- Logs deben incluir identidad del módulo.
- Timeouts, errores y restarts se exponen en `--modules-debug`.

Ver `ERROR_ISOLATION_POLICY.md`.

---

## 14. Compatibilidad futura

- Extensiones de API deben ser aditivas cuando sea posible.
- Nuevas primitivas requieren evidencia desde módulos reales.
- Cambios breaking requieren nueva API y decisión documentada.
- La política general está en `MODULES_ARCHITECTURE.md`.
