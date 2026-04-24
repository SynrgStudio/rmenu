# PROVIDER EXECUTION POLICY

Estado: Draft v1  
Fecha: 2026-04-22

---

## 1) Objetivo

Definir cómo se ejecutan providers para mantener latencia y estabilidad.

---

## 2) Principios

1. Providers no deben bloquear UI.
2. Query nueva invalida trabajo viejo.
3. Core controla budgets, timeouts y merge final.
4. Resultado determinista y reproducible.

---

## 3) Ciclo por query

1. Core recibe query.
2. Crea `query_epoch` incremental.
3. Dispara providers habilitados.
4. Recolecta respuestas dentro de presupuesto.
5. Descarta respuestas stale (`epoch` viejo).
6. Merge + dedupe + ranking + decorate.

---

## 4) Budget / timeout v1

Parámetros sugeridos (ajustables):

- `provider_timeout_ms` por provider
- `query_total_budget_ms` para conjunto de providers
- `max_items_per_provider`

Si un provider excede timeout:

- se cancela/descarta respuesta,
- se registra warning,
- no se bloquea render.

---

## 5) Merge/dedupe policy

- dedupe por key estable (`id` o target normalizado).
- prioridad de fuente configurable (core vs provider).
- conflictos se resuelven de manera determinista.

---

## 6) Orden de ejecución

- providers ordenados por prioridad + nombre.
- misma entrada => mismo orden de evaluación.
- evita “saltos” no deterministas entre queries.

---

## 7) Errores

- error de provider no rompe query global.
- se loggea con módulo/proveedor/latencia.
- policy opcional de auto-disable tras N fallos consecutivos.

---

## 8) Observabilidad mínima

Registrar por provider:

- invocaciones,
- latencia p50/p95,
- timeouts,
- errores,
- ítems devueltos.

Exposición sugerida:

- `--modules-debug`
- sección en `scripts/audit.ps1` (futuro)

---

## 9) Recomendaciones para autores de módulos

- responder incremental/rápido,
- cachear cuando aplique,
- evitar I/O bloqueante sin timeout,
- respetar `max_items` y filtros por query.
