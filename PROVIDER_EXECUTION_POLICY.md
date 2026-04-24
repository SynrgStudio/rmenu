# PROVIDER EXECUTION POLICY

Estado: Frozen v1  
Fecha: 2026-04-24

---

## 1. Objetivo

Definir cómo se ejecutan providers para mantener baja latencia, estabilidad y resultados deterministas.

---

## 2. Principios

1. Providers no deben bloquear UI.
2. Query nueva puede invalidar trabajo viejo.
3. Core controla budgets, timeouts y merge final.
4. Resultado debe ser reproducible para la misma entrada.
5. Error de provider no rompe query global.

---

## 3. Ciclo por query

1. Core recibe query.
2. Core evalúa providers habilitados.
3. Core aplica capability `providers`.
4. Core recolecta respuestas dentro de presupuesto.
5. Core descarta respuestas inválidas, tardías o stale si aplica.
6. Core sanitiza items.
7. Core aplica cap por provider host.
8. Core hace merge con items base.
9. Core aplica dedupe.
10. Core aplica ranking final.
11. Core permite decorations si corresponde.
12. Core renderiza.

---

## 4. Configuración v1

Parámetros en `[Modules]`:

```ini
provider_total_budget_ms = 35
provider_timeout_ms = 1500
max_items_per_provider_host = 24
dedupe_source_priority = core_first
host_restart_backoff_ms = 800
max_ipc_payload_bytes = 262144
```

### `provider_total_budget_ms`

Presupuesto global aproximado para recolectar providers por query.

Si se excede, el core deja de consultar providers restantes y continúa con resultados disponibles.

### `provider_timeout_ms`

Timeout por request hacia provider/host.

Si se excede:

- se descarta la respuesta,
- se registra timeout,
- puede reiniciarse el host,
- puede contribuir a degradación/desactivación.

### `max_items_per_provider_host`

Cap máximo de items aceptados por provider host.

Items excedentes se truncan antes de entrar al pipeline.

### `dedupe_source_priority`

Valores:

- `core_first`
- `provider_first`

Define prioridad al resolver duplicados entre items core y provider.

### `max_ipc_payload_bytes`

Límite máximo de request/response IPC.

Payloads mayores se rechazan para proteger estabilidad.

---

## 5. Merge y dedupe

Dedupe debe ser determinista.

Key sugerida:

1. `id` estable si aplica,
2. `target` normalizado si existe,
3. fallback controlado por core.

Políticas:

- `core_first`: ante duplicado, gana item core.
- `provider_first`: ante duplicado, gana item de provider.

Después de dedupe, el ranking final sigue siendo autoridad del core.

---

## 6. Orden de ejecución

Providers deben evaluarse en orden estable:

1. prioridad,
2. nombre de módulo/provider,
3. orden normalizado de descubrimiento si aplica.

Objetivo: misma entrada debe producir mismo orden observable.

---

## 7. Sanitización de items

El core debe validar:

- `id` obligatorio,
- `title` obligatorio,
- longitudes máximas,
- strings multiline donde no correspondan,
- `quickSelectKey` válido,
- campos desconocidos ignorables,
- payload total dentro de límite IPC.

Items inválidos se descartan o normalizan. No deben crashear el core.

---

## 8. Errores

Ante error de provider:

- la query global continúa,
- se registra módulo/provider/latencia/error,
- no se renderiza resultado parcial inválido,
- el host puede reiniciarse,
- errores consecutivos pueden degradar o deshabilitar el módulo.

Ver `ERROR_ISOLATION_POLICY.md`.

---

## 9. Observabilidad mínima

`--modules-debug` debe exponer al menos:

- módulos cargados,
- hosts corriendo,
- estado por host,
- request count,
- error count,
- timeout count,
- restart count,
- latencia promedio/máxima,
- últimos errores.

---

## 10. Recomendaciones para autores

- Responder rápido.
- Devolver pocos items relevantes.
- Filtrar por query temprano.
- Cachear cuando aplique.
- Evitar I/O bloqueante.
- No depender de recibir siempre todas las queries.
- No depender de orden no documentado.
- Manejar errores propios y loggear contexto.
