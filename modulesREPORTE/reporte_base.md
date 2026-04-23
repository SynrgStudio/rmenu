Reporte completo de arquitectura propuesta para módulos/extensiones en rmenu
Objetivo general

Diseñar una arquitectura de extensiones para rmenu donde:

el core siga siendo chico, estable y entendible
las funcionalidades opcionales no contaminen el núcleo
las extensiones se carguen como archivos o módulos externos “pegables en carpeta”
el sistema permita agregar comportamiento, contenido y cues visuales
las extensiones se sientan nativas, pero sin tener acceso arbitrario a internals del launcher
la UI pueda enriquecerse visualmente sin dejar que cada extensión “dibuje lo que quiera”

La idea central es:

las extensiones no tocan internals del core

las extensiones reaccionan a eventos, aportan metadata y piden acciones

el core sigue siendo la autoridad sobre estado, render e input

1. Filosofía arquitectónica
1.1 Separación fuerte entre core y extensiones

El core de rmenu debe seguir siendo responsable de:

ciclo de vida principal
input
query
filtrado / ranking base
selección
submit
render
estado interno
interacción base con items
contrato/API de extensiones

Las extensiones deben ser responsables de:

agregar comportamiento opcional
aportar items o providers
registrar comandos
enriquecer metadata visual/semántica de items
reaccionar a eventos del launcher
integrar módulos externos del ecosistema
1.2 Regla de oro

La regla más importante de toda la arquitectura es:

una extensión puede declarar intención y pedir cambios

pero no puede mutar internals arbitrariamente

Ejemplos sanos:

ctx.setSelection(3)
ctx.setQuery("blender")
ctx.registerCommand(...)
ctx.addItems(...)

Ejemplos que NO deberían existir:

acceso a state.selected_index
acceso al renderer Win32
modificar estructuras internas del ranking
interceptar el loop de UI
mutar directamente el cache interno
dibujar con GDI o APIs internas
2. Modelo general del sistema de extensiones

La arquitectura propuesta separa las extensiones en cuatro conceptos:

2.1 Hooks

Eventos emitidos por el core a los que una extensión puede reaccionar.

2.2 Context

Vista controlada, de solo lectura o mutación limitada, del estado actual del launcher.

2.3 Actions

Conjunto pequeño de operaciones autorizadas que el módulo puede pedirle al core.

2.4 Contributions

Aportes que el módulo puede registrar o devolver:

items
providers
commands
decorations
capabilities
3. Superficie pública mínima del sistema de módulos

La API pública del sistema debe ser deliberadamente chica.

3.1 Hooks mínimos propuestos
onLoad(ctx)

Se ejecuta cuando el módulo se carga.

Usos:

registrar comandos
registrar providers
inicializar estado interno del módulo
validar capabilities requeridas
onUnload(ctx)

Se ejecuta cuando el módulo se descarga o recarga.

Usos:

liberar recursos
limpiar registros
cerrar conexiones externas si aplica
onQueryChange(query, ctx)

Se ejecuta cuando cambia el texto actual del launcher.

Usos:

lógica de comportamiento
activar/desactivar cosas según query
preparar providers o respuestas contextuales
onSelectionChange(item, index, ctx)

Se ejecuta cuando cambia el item seleccionado.

Usos:

reaccionar a navegación
loggear
mostrar hints
preparar estado contextual
onKey(keyEvent, ctx)

Se ejecuta ante input de teclado.

Usos:

shortcuts opcionales
navegación adicional
comportamiento custom de módulos

Importante:

este hook no debería reemplazar el input del core salvo casos explícitos
idealmente, el core procesa primero las teclas esenciales y luego expone hooks complementarios
onSubmit(item, ctx)

Se ejecuta cuando se confirma un item.

Usos:

comportamiento custom post-submit
instrumentation
acciones secundarias controladas
onCommand(command, args, ctx)

Se ejecuta cuando se invoca un slash-command u otro comando del sistema.

Usos:

módulos estilo /scripts, /windows, /projects
4. Hooks adicionales recomendados para el caso visual

Para soportar el caso del badge/chip/shortcut visual, hace falta una categoría más:

4.1 Hooks de decoración / enriquecimiento
decorateItems(items, ctx) -> Item[]

Este hook permite recibir la lista de items ya construida y devolverla enriquecida con metadata adicional.

Usos:

agregar badges
agregar capabilities
agregar hints
agregar tags
marcar estados
enriquecer subtítulos
anotar items con metadata visual o semántica

Esto es clave porque permite que un módulo diga:

“este item tiene shortcut 1”

“este item está running”

“este item es favorito”

sin tocar el renderer ni el layout directamente.

5. Contexto (ctx) expuesto a los módulos

El contexto debe ser limitado, estable y aburrido.

La idea no es “darle acceso a rmenu”.
La idea es dar una vista controlada del estado más algunas acciones permitidas.

5.1 Lecturas mínimas
Query actual
ctx.query()
Items visibles actuales
ctx.items()
Item actualmente seleccionado
ctx.selectedItem()
Índice actualmente seleccionado
ctx.selectedIndex()
Modo actual del launcher
ctx.mode()

Ejemplos:

launcher
stdin
command
Capabilities activas del sistema
ctx.hasCapability(name)
Metadata del módulo / entorno

Opcional:

ctx.moduleInfo()
ctx.runtimeInfo()
5.2 Utilidades mínimas de diagnóstico
ctx.log(message)
ctx.toast(message)
6. Actions permitidas

Las acciones son el mecanismo por el cual un módulo pide cambios al core.

No deben ser demasiadas.

6.1 Actions mínimas propuestas
Cambiar query
ctx.setQuery(text)
Cambiar selección absoluta
ctx.setSelection(index)
Mover selección relativa
ctx.moveSelection(offset)
Enviar submit del item actual
ctx.submit()
Cerrar launcher
ctx.close()
Agregar items
ctx.addItems(items)
Reemplazar items

Solo si el pipeline lo permite y en contextos bien definidos:

ctx.replaceItems(items)
Registrar comando
ctx.registerCommand(commandDef)
Registrar provider
ctx.registerProvider(providerDef)
7. Providers

Los providers son el mecanismo preferido para que una extensión aporte contenido.

Esto es muy importante, porque evita que un módulo mutile la lista de items en cualquier momento.

7.1 Qué es un provider

Un provider es una fuente de items que:

recibe la query actual
decide si responder o no
devuelve items
puede activarse solo bajo ciertos comandos o contextos
7.2 Casos de uso ideales
scripts activos
ventanas abiertas
bookmarks
recent projects
aliases
commands internos
integración con daemons externos
7.3 Ejemplo conceptual
ctx.registerProvider({
  name: "automation",
  command: "/scripts",
  query: async ({ query }) => {
    const scripts = await getScripts(query)
    return scripts.map(script => ({
      id: `script:${script.id}`,
      title: script.name,
      subtitle: script.running ? "Running" : "Stopped",
      action: {
        type: "run_script",
        payload: { id: script.id }
      }
    }))
  }
})
7.4 Beneficios de este enfoque
mantiene la identidad del core
centraliza entrada de contenido
hace más fácil el ranking y el render
evita mutaciones arbitrarias
escala bien hacia ecosistema modular futuro
8. Separación de tipos de módulos

La arquitectura se beneficia de separar dos grandes clases de extensiones.

8.1 Módulos de comportamiento

Modifican o enriquecen la interacción.

Ejemplos:

shortcuts numéricos
navegación custom
submit alternativo
comandos especiales
atajos contextuales

Viven sobre hooks.

8.2 Módulos de contenido

Agregan cosas para mostrar.

Ejemplos:

scripts
bookmarks
projects
clipboard history
módulos del ecosistema

Viven sobre providers.

9. Soporte visual limpio: decorations y capabilities

Este es el punto central para el caso del “chip 1” o cue visual de shortcuts.

9.1 Principio clave

el módulo no dibuja

el módulo declara metadata

el core decide cómo renderizar esa metadata

Eso evita convertir módulos en mini renderers o en hacks sobre Win32/GDI.

9.2 Dos categorías distintas que el item puede exponer
A. Capabilities

Describen lo que el item puede hacer o cómo se comporta.

Ejemplo:

tiene quick shortcut
soporta acción especial
pertenece a cierta clase funcional
B. Decorations

Describen cómo el item debería sugerirse visualmente, dentro del sistema controlado del core.

Ejemplo:

badge
tag
status hint
visual cue
9.3 Estructura sugerida
type Item = {
  id: string
  title: string
  subtitle?: string
  source?: string
  action: Action

  capabilities?: {
    quickSelectKey?: string
    shortcuts?: string[]
  }

  decorations?: {
    badge?: string
    badgeKind?: "shortcut" | "status" | "tag"
    hint?: string
    icon?: string
  }
}
10. Caso puntual: shortcut visual para Blender 5.0
10.1 Requerimiento funcional

El usuario quiere que:

Ctrl+Shift+Space abra rmenu
si Blender 5.0 tiene asignado shortcut 1
en la lista aparezca visualmente un chip/badge con 1
al apretar 1, el launcher seleccione o ejecute ese item según la política definida
10.2 Diseño correcto

La asignación de shortcut no debe implementarse:

ni tocando el renderer
ni tocando input interno arbitrario
ni dibujando desde el módulo

La forma limpia es:

el módulo declara
que cierto item tiene quickSelectKey = "1"
el core hace
render del badge de forma estándar
manejo del input de quick-select
layout del chip
colores según theme
spacing consistente
10.3 Ejemplo de item enriquecido
{
  "id": "app:blender-5.0",
  "title": "Blender 5.0",
  "action": {
    "type": "launch",
    "target": "C:\\Program Files\\Blender Foundation\\Blender 5.0\\blender.exe"
  },
  "capabilities": {
    "quickSelectKey": "1"
  },
  "decorations": {
    "badge": "1",
    "badgeKind": "shortcut"
  }
}
10.4 Mejor simplificación

Incluso podría evitarse duplicación si el core deriva el badge directamente desde quickSelectKey.

Es decir, bastaría con:

capabilities: {
  quickSelectKey: "1"
}

y el core automáticamente:

renderiza badge "1"
usa estilo visual de shortcut
procesa la tecla 1

Eso es aún más limpio.

10.5 Recomendación concreta

Haría que quickSelectKey sea una capability oficial del modelo de item, no un hack de módulo.

El módulo decide a qué item asignarlo.
El core resuelve toda la parte visual y funcional.

11. Pipeline recomendado para soportar enriquecimiento visual
11.1 Flujo interno sano
providers generan items base
core filtra/ranquea
módulos decorateItems() enriquecen metadata
core renderiza según capabilities/decorations
core procesa input y submit

Esto es importante porque:

separa contenido, ranking, enriquecimiento y render
permite cues visuales ricos sin abrir el renderer a extensiones
deja bien clara la autoridad del core
12. Por qué el core debe renderizar badges/chips y no el módulo
12.1 Beneficios
Consistencia visual

Todos los badges:

mismo padding
mismo estilo
mismo alineado
mismos tokens de color
misma integración con themes
Simplicidad

Los módulos no necesitan saber:

Win32
GDI
layout interno
offsets
clipping
fuentes
Estabilidad

La API de módulos no depende del renderer.

Evolutividad

Se puede cambiar el renderer del core sin romper módulos.

12.2 Regla propuesta

Los módulos pueden declarar:

badge
badgeKind
hint
quickSelectKey

Pero no:

posición exacta
tamaño
fuente
borde
color específico por pixel
draw callbacks
13. Soporte para themes y otras customizaciones visuales

Esto se conecta con la misma idea.

13.1 Themes no deberían ser módulos de scripting general

Para colores, tipografías, spacing, etc., conviene usar una forma declarativa.

Ejemplo:

[module]
name = "nord-theme"
type = "theme"

[theme]
bg = "#2E3440"
fg = "#ECEFF4"
selected_bg = "#5E81AC"
selected_fg = "#FFFFFF"
hint_fg = "#88C0D0"
13.2 Motivo

No tiene sentido obligar a usar scripting para:

colores
theme tokens
spacing simple
variantes de look

Separar:

módulos declarativos
módulos scriptados

hace todo más mantenible.

14. Formato y carga de extensiones
14.1 Estructura de carpeta sugerida
modules/
  numeric-shortcuts/
    module.toml
    index.js

  nord-theme/
    module.toml

  automation/
    module.toml
    index.js
14.2 Manifest sugerido (module.toml)
name = "numeric-shortcuts"
version = "0.1.0"
kind = "script"
entry = "index.js"
capabilities = ["keys", "decorate-items"]

Ejemplo para theme:

name = "nord-theme"
version = "0.1.0"
kind = "theme"
14.3 Hot reload v1

Cuando cambia un módulo:

detectar cambio en archivo
descargar/unload módulo
volver a cargarlo
re-registrar providers/comandos/hooks
loggear éxito o error
Importante

No intentar al principio:

preservar estado complejo
migración automática de runtime
live patching profundo
hot swap de código nativo

Que recargue limpio alcanza.

15. Qué NO debería exponer la API v1

Esto es crítico.

No deberían estar expuestos:

renderer Win32 / GDI
structs internas del core
acceso a estado mutable global
ranking engine interno
cache interno
hooks sin límites “before everything / after everything”
capacidad de cambiar event loop
acceso libre a layout
acceso a fuentes internas del launcher/history/path sin pasar por providers

Porque si la API v1 expone demasiado:

se vuelve imposible de estabilizar
los módulos terminan acoplados a detalles internos
el core deja de poder refactorizarse
16. Distinción importante: declarar intención vs ejecutar implementación

El patrón correcto para toda la arquitectura es:

El módulo declara intención

Ejemplos:

“este item tiene quickSelectKey 1”
“quiero registrar /scripts”
“quiero aportar items de automation”
“quiero mover la selección”
El core implementa esa intención

Ejemplos:

dibuja el badge
enruta input
ejecuta acciones
gestiona render
resuelve ranking
mantiene consistencia del estado

Esto es lo que permite que algo “se sienta nativo” sin vivir dentro del core.

17. Dos niveles de extensibilidad recomendados
17.1 Extensiones declarativas

Para:

themes
aliases
static items
reglas simples
shortcuts fijos
colores
17.2 Extensiones scriptadas

Para:

hooks
providers dinámicos
integración con daemons externos
slash commands
decoraciones contextuales
comportamiento custom

Esta división es sana porque evita convertir todo en scripting.

18. API mínima v1 recomendada

Si hubiese que reducirlo a una v1 realmente práctica, propondría:

18.1 Hooks expuestos
onLoad
onUnload
onQueryChange
onSelectionChange
onKey
onSubmit
onCommand
decorateItems
18.2 Lecturas de contexto
ctx.query()
ctx.items()
ctx.selectedItem()
ctx.selectedIndex()
ctx.mode()
ctx.hasCapability(name)
ctx.log(msg)
ctx.toast(msg)
18.3 Actions permitidas
ctx.setQuery(text)
ctx.setSelection(index)
ctx.moveSelection(offset)
ctx.submit()
ctx.close()
ctx.addItems(items)
ctx.replaceItems(items) en contextos controlados
ctx.registerCommand(def)
ctx.registerProvider(def)
18.4 Modelo de item con soporte visual/semántico
capabilities.quickSelectKey
decorations.badge
decorations.badgeKind
decorations.hint
19. Ejemplo concreto de módulo de shortcuts numéricos
19.1 Objetivo

Asignar teclas rápidas a items específicos y mostrar badge visual.

19.2 Config conceptual
[shortcuts]
"1" = "app:blender-5.0"
"2" = "app:thorium"
"3" = "app:vscode"
19.3 Decoración de items
export function decorateItems(items, ctx) {
  return items.map(item => {
    const quickKey = findShortcutForItem(item.id)
    if (!quickKey) return item

    return {
      ...item,
      capabilities: {
        ...item.capabilities,
        quickSelectKey: quickKey
      }
    }
  })
}
19.4 Responsabilidad del core
si un item tiene quickSelectKey, mostrar badge
al recibir tecla matching, seleccionar o ejecutar según política
mantener estilo visual consistente
19.5 Ventaja

Esto mantiene:

comportamiento nativo
UI consistente
API limpia
lógica de asignación desacoplada del renderer
20. Política recomendada para input de quick-select

Hay dos opciones:

Opción A

El módulo maneja la tecla y selecciona items.

Problema

Demasiado comportamiento queda delegado a módulos.

Opción B

El módulo solo declara quickSelectKey y el core resuelve input.

Recomendación

Elegir esta.

Porque convierte a quickSelectKey en una capability oficial y estable del launcher, en vez de un hack por hook de teclado.

21. Dirección futura compatible con ecosistema más grande

Esta arquitectura sirve hoy para rmenu y mañana para un ecosistema mayor.

Por ejemplo, un futuro módulo de automation podría:

registrar /scripts
exponer scripts activos como items
devolver badges de estado
integrarse con un daemon externo

Todo eso sin meter el daemon dentro del core.

22. Conclusión arquitectónica

La propuesta completa se resume en esta idea:

rmenu debe exponer una API chica, estable y controlada basada en hooks, contexto, actions, providers y metadata de items.

Los módulos no deben tocar internals, renderer ni estado arbitrario.

Los módulos deben reaccionar a eventos, aportar contenido y declarar metadata semántica/visual.

El core debe seguir siendo la autoridad sobre input, estado, ranking, submit y render.

Para cues visuales como shortcuts, badges o hints, el módulo declara metadata; el core materializa esa metadata de manera consistente y nativa.

23. Resumen ejecutivo ultra corto

Si querés una versión resumida para otro agente:

No exponer internals del core.
Exponer hooks + contexto controlado + actions permitidas.
Usar providers para contenido dinámico.
Usar decorateItems() para metadata visual/semántica.
El core renderiza; los módulos no dibujan.
quickSelectKey debería ser capability oficial del item.
El core maneja la tecla y muestra el badge automáticamente.
Separar módulos declarativos de módulos scriptados.
Hot reload simple, sin magia.
La autoridad siempre la conserva el core.