# MODULES QUICKSTART — rmenu

Estado: Frozen v1  
Propósito: guía corta para instalar, desarrollar y debuggear módulos.

---

## 1. Ubicación de módulos

Por defecto, `rmenu` descubre módulos en:

```text
modules/
```

Formatos soportados:

```text
modules/
  my-module.rmod
  another-module/
    module.toml
    module.js
    config.json       # opcional
    README.md         # opcional
```

La identidad del módulo la define el manifest (`name`), no el nombre del archivo o carpeta.

---

## 2. Instalar un módulo `.rmod`

1. Crear la carpeta si no existe:

```powershell
mkdir modules
```

2. Copiar el archivo:

```powershell
copy .\my-module.rmod .\modules\my-module.rmod
```

3. Verificar carga:

```powershell
rmenu.exe --modules-debug
```

4. Si `rmenu` ya está abierto y el módulo fue editado, usar:

```text
/modules.reload
```

---

## 3. Instalar un módulo en carpeta

Estructura mínima:

```text
modules/
  hello-module/
    module.toml
    module.js
```

`module.toml` mínimo:

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

Validar:

```powershell
rmenu.exe --modules-debug
```

---

## 4. Desarrollar un módulo

Flujo recomendado:

1. Empezar con formato carpeta.
2. Declarar capabilities mínimas.
3. Implementar un hook pequeño.
4. Ejecutar `rmenu.exe --modules-debug`.
5. Iterar con hot reload o `/modules.reload`.
6. Revisar `recent_errors` y telemetría.
7. Empaquetar a `.rmod` cuando esté estable.

---

## 5. Debug rápido

### Ver módulos cargados

```powershell
rmenu.exe --modules-debug
```

### Recargar módulos desde rmenu

```text
/modules.reload
```

### Listar módulos desde rmenu

```text
/modules.list
```

### Resetear telemetría

```text
/modules.telemetry.reset
```

---

## 6. Módulo incluido: calculator

`rmenu` incluye un módulo real de validación:

```text
modules/calculator.rmod
```

Comportamiento:

- reconoce cálculos simples escritos directamente en el input, por ejemplo `2+2`, `10/2`, `(2+3)*4`;
- no requiere prefijo `=`;
- muestra el resultado en la misma barra, alineado a la derecha, como `=4`;
- usa `InputAccessory` con kind `success`, por lo que el core decide el color;
- reemplaza la lista inferior con `ctx.replaceItems([])` mientras el cálculo sea válido, para evitar resultados fuzzy irrelevantes;
- declara solo `input-accessory` como capability.

El core no contiene lógica de calculadora. Solo expone actions genéricas (`setInputAccessory`, `clearInputAccessory`, `replaceItems`) que el `.rmod` usa.

---

## 7. Errores comunes

### `RMOD_E_INVALID_MAGIC`

El `.rmod` no empieza con:

```text
#!rmod/v1
```

### `RMOD_E_MISSING_MODULE_JS`

Falta el bloque obligatorio:

```text
---module.js---
```

### `permission_denied`

El módulo intentó usar una operación sin declarar capability.

Ejemplo:

```text
permission_denied module='calc' operation='provide_items' capability='providers'
```

Solución: agregar la capability requerida o dejar de usar esa operación.

### Host con timeout

El módulo tardó más que `provider_timeout_ms` o no respondió.

Solución:

- reducir trabajo por query,
- cachear resultados,
- evitar I/O bloqueante,
- revisar errores con `--modules-debug`.

---

## 8. Capabilities rápidas

| Quiero hacer | Capability |
|---|---|
| Aportar items | `providers` |
| Registrar/recibir comandos | `commands` |
| Decorar items | `decorate-items` |
| Mostrar estado junto al input | `input-accessory` |
| Recibir teclas | `keys` |

Declarar solo lo necesario.

---

## 9. Contratos relacionados

Leer en este orden:

1. `MODULES_ARCHITECTURE.md`
2. `MODULES_API_SPEC_V1.md`
3. `RMOD_SPEC_V1.md` o `MANIFEST_SPEC_V1.md`
4. `MODULES_CAPABILITIES_MATRIX.md`
5. `MODULES_AUTHORING_GUIDE.md`
6. `MODULES_OPERATIONS_GUIDE.md`
