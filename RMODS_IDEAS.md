# rMods / rPacks / Companions ideas

Este archivo reúne ideas futuras para extender rMenu mediante `rmod`, `rpack`, resident helpers o companions nativos.

Ranking de importancia:

- **P0**: alto impacto inmediato; complementa directamente el uso diario de rMenu y los companions actuales.
- **P1**: muy útil; buen candidato después de los P0.
- **P2**: útil pero más específico o con más complejidad.
- **P3**: experimental, wow, o depende de más infraestructura.

## Packs útiles inmediatos

### Productividad

| Idea | Ranking | Por qué agregaría esto |
|---|---:|---|
| `clipboard-history` | P0 | Es una de las extensiones más naturales para un launcher residente. Complementa `shortcuts`, `RTasks` y futuros flujos de captura. Mucho uso diario. |
| `text-snippets` | P0 | Reemplaza parte del valor histórico de AHK/TextExpander sin meterlo en core. Puede empezar como rpack simple y evolucionar a helper residente. |
| `quick-notes` | P0 | Captura rápida a Markdown. Encaja con el objetivo de reducir fricción y puede ser puente hacia Anytype/Zettelkasten. |
| `scratchpad` | P1 | Útil para abrir/editar texto temporal sin pensar en archivos. Menor impacto que quick-notes, pero simple y práctico. |
| `timer` | P1 | Pomodoro/timers desde rMenu es fácil de usar y combina bien con daemon/resident notifications futuras. |
| `unit-converter` | P1 | Muy buen rmod puro: rápido, local, sin permisos raros, resultado inmediato en input/lista. |
| `timezone` | P2 | Útil si hay trabajo con equipos/horarios. Menor prioridad si no hay necesidad diaria. |
| `uuid-generator` | P1 | Dev/productividad inmediata. Muy simple de implementar y confiable. |
| `password-generator` | P1 | Alto valor, pero requiere cuidado UX/security. Local-only, sin guardar secretos. |
| `case-transform` | P1 | Muy útil para dev/escritura. Puede operar sobre clipboard o input y es fácil de testear. |

### Sistema Windows

| Idea | Ranking | Por qué agregaría esto |
|---|---:|---|
| `window-actions` | P0 | Recupera valor fuerte de AHK: mover/maximizar/centrar ventanas. Probablemente rpack/helper o companion futuro. |
| `always-on-top` | P0 | Feature clásica de AHK, simple, muy útil y con alto valor percibido. Buen resident/helper pack. |
| `audio-devices` | P0 | Complementa `taskbar-volume`. Cambiar output/input desde rMenu sería de uso diario. |
| `display-switch` | P2 | Potente pero más propenso a edge cases de hardware/monitores. |
| `power-menu` | P1 | Acciones del sistema centralizadas. Simple, pero debe tener confirmaciones para acciones destructivas. |
| `process-killer` | P1 | Muy útil para dev/Windows. Debe tener filtros y confirmación antes de matar procesos. |
| `services` | P2 | Bueno para usuarios técnicos, pero requiere permisos/admin y más cuidado. |
| `env-vars` | P1 | Muy buen rmod de lectura/copia. Útil para debug y dev. |
| `hosts-editor` | P2 | Útil, pero requiere admin y puede romper red. Mejor con backup/preview. |
| `path-manager` | P1 | Encaja con launcher/dev. Permite diagnosticar PATH sin abrir configuración de Windows. |

### Archivos

| Idea | Ranking | Por qué agregaría esto |
|---|---:|---|
| `recent-files` | P1 | Extiende launcher hacia documentos recientes. Mucho valor si se integra bien con ranking. |
| `project-opener` | P0 | De los mejores siguientes pasos: abrir repos/proyectos/folders con acciones contextuales. Complementa `pi-launcher` y dev workflows. |
| `folder-jump` | P1 | Simple y útil. Similar a shortcuts pero orientado a carpetas frecuentes. |
| `file-search` | P2 | Muy útil, pero puede necesitar indexación/companion (`RSearch`) para ser realmente rápido. |
| `duplicate-finder` | P3 | Útil pero menos launcher-like; puede ser herramienta puntual. |
| `zip-tools` | P2 | Práctico, pero no central al flujo diario. |
| `file-hash` | P1 | Dev/security utility simple, buena para rmod/rpack. |
| `rename-tools` | P2 | Útil, pero requiere UI/preview para no ser peligroso. |
| `watch-folder` | P3 | Más daemon/resident y menos inmediato. Bueno para automatización futura. |

### Dev

| Idea | Ranking | Por qué agregaría esto |
|---|---:|---|
| `git-repos` | P0 | Altísimo valor para rMenu como command center técnico. Abrir repo, status, pull, branch. |
| `git-branches` | P1 | Natural después de `git-repos`; más específico y requiere buen UX para no cambiar ramas accidentalmente. |
| `github-issues` | P1 | Buen puente con trabajo real. Puede abrir issues/PRs rápidamente; requiere auth opcional. |
| `docker-control` | P1 | Muy útil para dev. Requiere manejar errores si Docker no corre. |
| `ports` | P0 | Excelente utilidad diaria: ver quién usa un puerto y matar proceso. Muy complementario a dev. |
| `npm-scripts` | P1 | Detectar scripts del proyecto actual y ejecutarlos. Muy útil, pero depende de contexto/cwd/proyecto. |
| `cargo-tasks` | P1 | Especialmente útil en este ecosistema. Check/test/run/build específicos desde rMenu. |
| `log-tail` | P2 | Útil con proyectos, pero puede requerir panel/streaming o abrir terminal. |
| `json-tools` | P1 | Muy buen rmod puro para clipboard/input: prettify, minify, query. |
| `regex-tester` | P2 | Útil, pero requiere UI suficiente para feedback. |
| `http-requester` | P2 | Mini REST client es útil, pero puede crecer demasiado. Mejor MVP simple. |

### Browser/web

| Idea | Ranking | Por qué agregaría esto |
|---|---:|---|
| `browser-tabs` | P1 | Muy potente si hay bridge real. Similar al viejo Thorium-tabs pero orientado a búsqueda/activación. |
| `bookmarks` | P1 | Útil y relativamente simple si se leen archivos de perfil. |
| `web-searches` | P1 | Ya hay módulos base, pero se puede consolidar como pack configurable. |
| `translate` | P2 | Útil; depende de API o motor local. |
| `read-later` | P1 | Buen flujo de captura de URLs, puede conectar con notes/Zettelkasten. |
| `wayback` | P2 | Utility web simple, bajo costo, uso ocasional. |
| `domain-tools` | P2 | Bueno para dev/sysadmin; whois/dns/ping/traceroute. |
| `download-video` | P2 | Muy útil si usás yt-dlp; requiere helper y cuidado legal/UX. |

### IA / agentes

| Idea | Ranking | Por qué agregaría esto |
|---|---:|---|
| `prompt-library` | P1 | Centraliza prompts reutilizables. Buen fit con rMenu y pi. |
| `pi-sessions` | P0 | Muy alineado con tu uso real. Abrir sesiones/proyectos pi desde rMenu sería muy valioso. |
| `agent-runner` | P2 | Potente, pero más complejo: ejecutar prompts sobre archivos requiere seguridad y contexto. |
| `summarize-clipboard` | P1 | Flujo rápido de IA sobre clipboard. Muy útil si se integra con provider local/remoto. |
| `explain-error` | P1 | Toma error del clipboard/log y abre prompt. Excelente para dev. |
| `commit-message` | P1 | Generar commit message desde diff; muy útil pero requiere Git context. |
| `zettel-capture` | P0 | Muy alineado con el sistema Zettelkasten/Anytype. Captura rápida con bajo rozamiento. |

## Resident helper packs

| Idea | Ranking | Por qué agregaría esto |
|---|---:|---|
| `always-on-top-helper` | P0 | AHK classic. Muy simple y de alto impacto. Ideal como resident helper. |
| `window-grid-helper` | P0 | Window management es de las migraciones AHK más valiosas. Puede crecer hacia `RWin`. |
| `text-expander` | P0 | Reemplazo directo de AHK TextExpander. Necesita resident helper por hooks globales. |
| `capslock-layer` | P1 | Muy potente para navegación, pero requiere diseño cuidadoso para no interferir. |
| `mouse-gestures` | P2 | Poderoso pero propenso a conflictos con apps. Mejor después de más experiencia con helpers. |
| `focus-follows-mouse` | P3 | Muy específico; puede incomodar si no está muy configurable. |
| `auto-dark-mode` | P2 | Útil y simple, pero no central al command center. |
| `battery-guard` | P2 | Bueno para laptops; no universal. |
| `mic-mute-indicator` | P1 | Muy buen resident helper visible y práctico para meetings. |
| `camera-mute-indicator` | P2 | Similar a mic, pero menos universal. |
| `meeting-mode` | P1 | Alto valor: audio/apps/status durante reuniones. Requiere varias integraciones. |
| `game-mode` | P2 | Útil para pausar hooks/helpers en fullscreen. Buen hardening futuro. |

## Companions posibles

| Idea | Ranking | Por qué agregaría esto |
|---|---:|---|
| `RClip` | P0 | Clipboard manager nativo sería companion perfecto: residente, rápido, con UI propia o bridge rMenu. |
| `RWin` | P0 | Window manager nativo para reemplazar AHK WindowManager sin meter lógica en core. |
| `RText` | P0 | Text expander nativo y robusto; mejor que un rpack si crece mucho. |
| `RNote` | P1 | Captura/notas rápidas local-first, puente a Anytype/Markdown. |
| `RSearch` | P1 | Indexador local rápido para archivos/proyectos. Evita hacer file-search pesado en rMenu core. |
| `RDev` | P1 | Daemon dev para Git/Docker/ports/contexto de proyectos. Mucho valor para trabajo técnico. |
| `RAudio` | P1 | Mixer/audio devices como companion si `audio-devices` crece. |
| `RBrowser` | P1 | Bridge tabs/bookmarks/history. Alto valor, requiere integración con navegadores. |
| `RMail` | P3 | Potente pero mucha superficie/auth/API. |
| `RCalendar` | P2 | Útil para agenda, pero depende de providers. |
| `RVault` | P2 | Secrets/passwords local-first; alto valor pero requiere diseño de seguridad serio. |

## Packs “wow”

| Idea | Ranking | Por qué agregaría esto |
|---|---:|---|
| `command-recorder` | P3 | Muy wow: grabar acciones y convertirlas en rmod. Requiere mucha infraestructura. |
| `contextual-actions` | P0 | De los mejores: si clipboard/input es URL/path/json/error, ofrecer acciones. Multiplica el valor de otros rmods. |
| `smart-open` | P1 | Similar a contextual-actions, orientado a abrir según tipo. Buena UX si es predecible. |
| `workspace-switcher` | P0 | Muy alto impacto: perfiles de apps/ventanas/audio/folders. Encaja con rMenu como command center. |
| `rituals` | P1 | Secuencias tipo “modo trabajo/grabar/stream”. Puede construirse sobre workspace-switcher. |
| `panic-button` | P2 | Útil y divertido, pero requiere cuidado con acciones destructivas/ocultar/cerrar. |
| `daily-brief` | P2 | Bueno si hay integraciones de tasks/calendar/notes. |
| `meeting-brief` | P2 | Potente con calendar/notes/personas; depende de integraciones. |
| `project-dashboard` | P0 | Excelente para dev: detecta repo y muestra acciones relevantes. Puede unir `git-repos`, `cargo`, `npm`, `pi`. |
| `universal-inbox` | P1 | Muy alineado con captura: tasks, notes, links, clipboard en una cola. Puede crecer mucho. |

## Mis favoritos para próximos pasos

1. `clipboard-history` — P0: altísimo uso diario y buen primer companion/helper.
2. `always-on-top` — P0: reemplazo AHK simple y visible.
3. `window-actions` — P0: valor inmediato para gestión de ventanas.
4. `text-expander` — P0: migración AHK natural y de mucho impacto.
5. `project-opener` — P0: acelera trabajo técnico diario.
6. `git-repos` — P0: base de workflows dev.
7. `audio-devices` — P0: complementa `taskbar-volume`.
8. `contextual-actions` — P0: transforma rMenu en acciones inteligentes según contexto.
9. `zettel-capture` — P0: captura sin fricción para Anytype/Zettelkasten.
10. `workspace-switcher` — P0: modo trabajo/grabación/proyecto con una sola acción.

## Ranking global sugerido

### P0 — construir primero

- `clipboard-history`
- `text-snippets` / `text-expander`
- `quick-notes`
- `window-actions`
- `always-on-top`
- `audio-devices`
- `project-opener`
- `git-repos`
- `ports`
- `pi-sessions`
- `zettel-capture`
- `always-on-top-helper`
- `window-grid-helper`
- `RClip`
- `RWin`
- `RText`
- `contextual-actions`
- `workspace-switcher`
- `project-dashboard`

### P1 — construir después

- `scratchpad`
- `timer`
- `unit-converter`
- `uuid-generator`
- `password-generator`
- `case-transform`
- `power-menu`
- `process-killer`
- `env-vars`
- `path-manager`
- `recent-files`
- `folder-jump`
- `file-hash`
- `github-issues`
- `docker-control`
- `npm-scripts`
- `cargo-tasks`
- `json-tools`
- `browser-tabs`
- `bookmarks`
- `web-searches`
- `read-later`
- `prompt-library`
- `summarize-clipboard`
- `explain-error`
- `commit-message`
- `capslock-layer`
- `mic-mute-indicator`
- `meeting-mode`
- `RNote`
- `RSearch`
- `RDev`
- `RAudio`
- `RBrowser`
- `smart-open`
- `rituals`
- `universal-inbox`

### P2/P3 — backlog experimental o específico

- `timezone`
- `display-switch`
- `services`
- `hosts-editor`
- `file-search`
- `duplicate-finder`
- `zip-tools`
- `rename-tools`
- `watch-folder`
- `git-branches`
- `log-tail`
- `regex-tester`
- `http-requester`
- `translate`
- `wayback`
- `domain-tools`
- `download-video`
- `agent-runner`
- `mouse-gestures`
- `focus-follows-mouse`
- `auto-dark-mode`
- `battery-guard`
- `camera-mute-indicator`
- `game-mode`
- `RMail`
- `RCalendar`
- `RVault`
- `command-recorder`
- `panic-button`
- `daily-brief`
- `meeting-brief`
