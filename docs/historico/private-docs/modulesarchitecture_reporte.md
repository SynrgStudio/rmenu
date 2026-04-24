La necesidad de una “constitución” viva: MODULES_ARCHITECTURE.md
Problema actual

El conocimiento fuerte de la arquitectura modular está repartido entre:

código real
plan de implementación
decisiones conversadas fuera de la repo

Eso es peligroso porque la capa modular ya dejó de ser experimental. El runtime tiene responsabilidades reales y ya forma parte del programa principal.

Evidencia

Hoy el repositorio ya contiene:

carga de módulos externos desde modules/
soporte para carpeta o .rmod
normalización a ModuleDescriptor
runtime policy
capabilities
hooks
providers
decorations
input accessory
debug de módulos
hot reload

Todo eso existe en código, no solo como idea.

Por qué ya hace falta el documento

Cuando una arquitectura alcanza este nivel de realidad, hace falta un documento corto y vivo que responda estas preguntas:

qué es core y qué no
qué puede hacer un módulo
qué no puede hacer un módulo
qué significa “soportado”
qué superficies son estables
qué partes son experimentales o sujetas a cambio
cuáles son las primitives visuales oficiales
cuál es la filosofía de extensión del proyecto
Qué debería contener ese documento

Un MODULES_ARCHITECTURE.md o equivalente debería congelar, como mínimo:

Identidad del sistema
rmenu como core autoritativo
módulos como extensiones por composición, no por mutación
Contrato del módulo
hooks expuestos
contexto expuesto
actions permitidas
contributions posibles
Primitives oficiales
providers
commands
decorate-items
input accessory
quick select capability
badges/hints
Fronteras explícitas
módulos no dibujan directamente
módulos no mutan internals
módulos no tocan renderer Win32/GDI
módulos no redefinen ranking interno arbitrario
Niveles de estabilidad

Idealmente:

Supported Module API
Experimental/Unsafe API, si más adelante existiera
Conclusión de este punto

El documento no es burocracia. Es protección contra deriva conceptual.

En el estado actual del repo, no tener ese documento ya es una deuda real.