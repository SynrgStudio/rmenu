# ERROR ISOLATION POLICY (Modules)

Estado: Draft v1  
Fecha: 2026-04-22

---

## 1) Objetivo

Evitar que errores de un módulo comprometan estabilidad del launcher.

---

## 2) Principio

**Fail module, not launcher.**

El fallo de un módulo debe aislarse y degradar funcionalidad opcional, no romper core.

---

## 3) Alcance de aislamiento

Aplicar aislamiento en:

- hooks,
- providers,
- commands,
- actions solicitadas por módulo,
- load/reload.

---

## 4) Estrategia de manejo de errores

1. Capturar error por boundary de módulo.
2. Registrar contexto (módulo, hook/provider, query, stack si existe).
3. Devolver estado seguro al core.
4. Continuar pipeline sin ese resultado.

---

## 5) Política de desactivación

Parámetros sugeridos:

- `max_consecutive_errors_per_module`
- `max_timeouts_per_module`

Si excede umbral:

- módulo pasa a estado `degraded` o `disabled`.
- se muestra aviso en debug/log.

---

## 6) Estados de módulo

- `loaded`
- `degraded`
- `disabled`
- `unloaded`

Transiciones deben ser explícitas y observables.

---

## 7) UX ante fallo

- launcher sigue operativo,
- no crashes visibles,
- mensajes de error sólo en modo debug/log,
- opcional: toast breve no intrusivo.

---

## 8) Recuperación

- reload manual o hot reload puede reactivar módulo.
- reset de contadores de error al reload exitoso.

---

## 9) Testing de aislamiento

Casos mínimos:

- hook lanza error,
- provider timeout,
- command inválido,
- action denegada por permisos,
- reload fallido.

Cada caso debe verificar:

- core sigue estable,
- error registrado,
- módulo entra al estado esperado.
