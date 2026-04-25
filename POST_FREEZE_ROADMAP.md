# POST-FREEZE ROADMAP — rmenu

Status: living roadmap  
Created: 2026-04-24  
Context: `rmenu` core v1 is frozen. Future product expansion happens through modules first.

---

## 1. North star

`rmenu` should become a fast local command surface for Windows:

- launcher first;
- modular by default;
- local-first;
- keyboard-native;
- scriptable;
- observable;
- safe when modules fail;
- extensible without growing the core.

Post-freeze rule:

> Product capabilities should be modules, templates, examples, docs, or external tooling unless they meet the core freeze policy.

In this document, “feature” means “module/product capability”, not “new core behavior”.

---

## 2. Work lanes

### Lane A — Module ecosystem

Build useful modules that prove the platform and make `rmenu` valuable day to day.

### Lane B — Module authoring experience

Make it easy to create, test, debug, package, and share modules.

### Lane C — Product polish

Improve the launcher experience without expanding the core contract unnecessarily.

### Lane D — Packaging and release

Make `rmenu` easy to install, update, and trust.

### Lane E — Observability and quality

Keep the frozen core healthy with metrics, diagnostics, regression tests, and compatibility checks.

### Lane F — Future API research

Collect friction for possible v2 primitives without rushing them into the core.

---

# Lane A — Module ecosystem

## A0. System actions module

Prefix ideas:

```text
sys
power
```

Behavior:

- lock screen;
- sleep;
- hibernate if available;
- restart/shutdown with confirmation;
- log off with confirmation;
- open power settings;
- toggle or open common Windows settings through documented commands/URIs.

Safety:

- destructive/session-ending actions require confirmation;
- no silent shutdown/restart/logoff;
- elevated actions must be explicit and clearly labeled;
- module should start with safe/open actions before adding destructive actions.

Examples:

```text
sys lock
sys sleep
sys power settings
sys restart
sys shutdown
```

Implementation note:

- use documented Windows commands/URIs from the module;
- do not add power-action-specific behavior to the core.

---

## A1. Clipboard module

Type/prefix ideas:

```text
clip
clip error
clip json
```

Capabilities:

- provider;
- command;
- input accessory if needed.

Possible behavior:

- list recent clipboard entries;
- search clipboard history;
- paste/copy selected entry;
- pin entries;
- clear sensitive entries;
- show badges for text/path/url/json/image metadata.

Acceptance:

- [ ] implemented as module only;
- [ ] no core clipboard logic;
- [ ] clear handling for sensitive/private entries;
- [ ] documented module config.

---

## A2. Recent files module

Prefix ideas:

```text
recent
rf
```

Possible sources:

- Windows recent items;
- user-configured directories;
- project workspaces;
- app-specific recent files when available.

Behavior:

- show recent files with app/source badges;
- open file through default handler;
- reveal in Explorer command;
- filter by extension.

---

## A3. Workspace/project launcher module

Prefix ideas:

```text
ws
proj
```

Behavior:

- list known workspaces;
- open in terminal/editor/file manager;
- run workspace commands;
- detect Git repos;
- badge dirty/clean branch state if fast enough.

Potential commands:

```text
/workspaces::reload
/workspaces::add
/workspaces::open-code
/workspaces::open-terminal
```

---

## A4. Git module

Prefix ideas:

```text
git
branch
pr
```

Behavior:

- show current repo status;
- list branches;
- checkout branch;
- open remote URL;
- copy current branch;
- run safe commands only;
- show status accessory while inside repo.

Hard rule:

- destructive commands require explicit confirmation or should be omitted.

---

## A5. Browser/bookmarks module

Prefix ideas:

```text
bm
web
```

Behavior:

- search browser bookmarks;
- open bookmark;
- search history if explicitly enabled;
- support Chrome/Edge/Firefox profiles;
- badges for browser/profile/folder.

Privacy:

- disabled by default for history;
- config must make data sources explicit.

---

## A6. Windows settings module

Prefix ideas:

```text
settings
win
```

Behavior:

- open common `ms-settings:` URIs;
- Control Panel shortcuts;
- network/bluetooth/display/sound/privacy pages;
- quick badges for category.

No core change needed: items target `ms-settings:` URIs.

---

## A7. Services/processes module

Prefix ideas:

```text
svc
proc
```

Behavior:

- list Windows services;
- start/stop/restart selected service only after confirmation;
- list user processes;
- kill process only after confirmation;
- show elevated-permission requirement clearly.

Security:

- no silent admin actions;
- commands must be explicit and namespaced.

---

## A8. Notes/Zettelkasten module

Prefix ideas:

```text
note
zettel
zk
```

Behavior:

- search local notes;
- create fleeting note;
- open Anytype/local note target;
- expose capture commands;
- show tags/badges;
- maybe integrate with existing Anytype workflow externally.

Important:

- Anytype-specific logic stays in module/external tooling, not core.

---

## A9. Command palette module

Prefix ideas:

```text
/
cmd
```

Behavior:

- list all registered module commands;
- show command descriptions;
- execute namespaced commands;
- provide examples/help.

Potential value:

- makes modules discoverable.

Risk:

- if this requires core command introspection not currently exposed, document friction before changing core.

---

## A10. Environment/tools module

Prefix ideas:

```text
tool
which
```

Behavior:

- search PATH tools with richer metadata;
- show version when cheap;
- open containing folder;
- copy path;
- explain duplicates/shadowing.

---

## A11. Snippets/text expansion module

Prefix ideas:

```text
snip
```

Behavior:

- search snippets;
- copy snippet;
- paste through target action if supported externally;
- template variables;
- badges for language/category.

---

## A12. Calculator v2 module

Keep calculator out of core, but improve module UX.

Ideas:

- copy result command;
- result history;
- unit conversion;
- date math;
- currency conversion if configured;
- expression explain mode.

Potential friction to monitor:

- official “copy text” action may become a general primitive candidate only if several modules need it.

---

## A13. Local scripts v3 module

Ideas:

- `/local-scripts::reload`;
- `/local-scripts::list`;
- categories;
- script metadata from comments/frontmatter;
- environment badges;
- dry-run preview;
- per-script confirmation flags.

---

## A14. Shortcuts v2 module

Ideas:

- list all shortcuts;
- edit/delete shortcuts;
- conflict detection;
- import/export shortcuts;
- multiple targets per alias with disambiguation;
- shortcut groups.

No core shortcut-specific behavior.

---

## A15. Timer/reminder module

Prefix ideas:

```text
timer 10m tea
remind 15m stand up
```

Behavior:

- create local timer/reminder;
- show active timers;
- cancel timer;
- notify through external mechanism.

May require external helper process if reminders outlive `rmenu`.

---

## A16. AI/local assistant module

Prefix ideas:

```text
ai
ask
```

Behavior:

- send selected prompt to configured local/remote model;
- summarize clipboard/text;
- generate command suggestions;
- copy output.

Hard boundaries:

- API keys/config live in module config;
- no AI logic in core;
- no automatic command execution without confirmation.

---

# Lane B — Module authoring experience

## B1. Module templates

Create examples/templates for common patterns:

```text
modules/templates/basic-provider.rmod
modules/templates/scoped-intent.rmod
modules/templates/input-accessory.rmod
modules/templates/command.rmod
modules/templates/key-binding-flow.rmod
modules/templates/decorator.rmod
```

Acceptance:

- [ ] each template runs;
- [ ] each template has minimal capabilities;
- [ ] each has README block or docs.

---

## B2. Module gallery

Document shipped and community modules:

```text
MODULES_GALLERY.md
```

For each module:

- purpose;
- prefix/commands;
- capabilities;
- install path;
- screenshots/gifs optional;
- safety notes.

---

## B3. Module test harness

External or dev-only helper to run module hooks without launching the UI.

Possible commands:

```text
rmenu-module-test modules/foo.rmod --query "abc"
rmenu-module-test modules/foo.rmod --key ctrl+b
```

Important:

- prefer external tooling first;
- only move into core if general need is proven.

---

## B4. Module pack/check tool

Tooling ideas:

```text
rmod check modules/foo.rmod
rmod pack modules/foo/
rmod explain modules/foo.rmod
```

Checks:

- required fields;
- capabilities used but not declared;
- invalid config JSON;
- oversized blocks;
- risky commands warning.

---

## B5. Better module debugging guide

Expand docs with:

- common `permission_denied` causes;
- timeout debugging;
- payload size debugging;
- hot reload workflow;
- `--modules-debug` interpretation;
- safe logging patterns.

---

## B6. Module versioning guide

Document:

- semver-like module versions;
- `api_version = 1` meaning;
- when to bump module minor/patch;
- migration notes;
- compatibility table.

---

## B7. Module distribution conventions

Define:

- recommended `.rmod` naming;
- signing/checksum ideas;
- README metadata;
- license field convention;
- source URL convention;
- trust model.

---

# Lane C — Product polish

## C1. Visual theme polish

Ideas:

- refined default colors;
- light/dark example configs;
- compact/comfortable presets;
- high contrast preset;
- screenshot-ready default.

Keep as config/docs unless a core bug blocks it.

---

## C2. Better screenshots/gifs

Create visual assets:

- launcher search;
- calculator module;
- local scripts scoped mode;
- shortcuts binding flow;
- `--modules-debug` output.

---

## C3. README product rewrite

Make README more compelling:

- one-line value prop;
- animated/demo section;
- “Why modules?” section;
- quickstart in 60 seconds;
- module examples.

---

## C4. Default config refinement

Review defaults after real usage:

- position;
- width;
- row count;
- colors;
- source boosts;
- blacklist entries;
- quick select mode.

Validation:

- manual UX check;
- metrics still under targets.

---

## C5. History UX review

Questions:

- is history boost right?
- should history show clearer label?
- are stale entries handled well?
- is persistence robust?

Only change core for bug/quality/performance, not product-specific behavior.

---

## C6. Launch failure UX

Improve user-facing feedback when launch fails:

- clear stderr;
- optional input accessory message;
- docs for common failure cases.

If UI feedback requires new primitive, document friction first.

---

# Lane D — Packaging and release

## D1. Release build checklist

Create:

```text
RELEASE_CHECKLIST.md
```

Include:

- `cargo fmt`;
- `cargo test`;
- `cargo check`;
- release build;
- smoke test;
- modules debug;
- metrics;
- checksum;
- changelog/release notes.

---

## D2. GitHub release workflow

Possible automation:

- build Windows binary;
- upload artifact;
- attach checksums;
- include `config_example.ini`;
- include docs bundle.

---

## D3. Installer/update story

Options:

- zip release first;
- Scoop manifest;
- winget later;
- PowerShell install script;
- no auto-updater until trust model is clear.

---

## D4. Binary signing research

Investigate:

- Windows SmartScreen implications;
- code signing cost;
- unsigned binary docs;
- checksum verification.

---

# Lane E — Observability and quality

## E1. Release-mode benchmark baseline

Run repeated release metrics:

```powershell
cargo build --release
1..10 | % { .\target\release\rmenu.exe --metrics }
.\target\release\rmenu.exe --reindex --metrics
```

Track:

- startup prepare;
- time to visible;
- time to input ready;
- search p95;
- dataset size.

---

## E2. Large dataset stress test

Validate with larger synthetic or real datasets:

- 5k items;
- 10k items;
- long labels;
- duplicate targets;
- Unicode names.

---

## E3. Windows compatibility pass

Validate on:

- Windows 10;
- Windows 11;
- PowerShell 5;
- PowerShell 7;
- different DPI scaling;
- multiple monitor setups.

---

## E4. Module failure chaos suite

Expand tests/fixtures for:

- host exits mid-request;
- invalid JSON bursts;
- slow response then recovery;
- repeated reload failure;
- oversized item fields;
- many modules failing at once.

Only if existing coverage proves insufficient or regressions appear.

---

## E5. Diagnostics snapshots

Add docs/examples for capturing:

```powershell
rmenu.exe --modules-debug
rmenu.exe --metrics
rmenu.exe --debug-ranking query
```

Useful for bug reports.

---

# Lane F — Future API research

These are not commitments. They are friction buckets. Anything here must go through `CORE_FREEZE_V1.md` policy.

## F1. Clipboard/copy action primitive

Potential need:

- calculator result copy;
- snippets copy;
- path copy;
- AI output copy;
- bookmark URL copy.

Research question:

- can modules handle this externally, or is a safe core `copyText` action justified?

---

## F2. Notification primitive

Potential need:

- timers;
- long-running modules;
- background tasks.

Research question:

- should modules use external OS notifications, or should core expose a tiny notification primitive?

---

## F3. Background module lifecycle

Potential need:

- timers;
- watchers;
- clipboard history;
- file indexing;
- workspace cache.

Risk:

- background lifecycle can complicate core stability.

Default answer:

- external helper process first.

---

## F4. Richer item actions

Potential need:

- open;
- copy;
- reveal;
- edit;
- delete;
- secondary actions.

Research question:

- can commands and scoped modes cover this, or is an item action primitive needed?

---

## F5. Module command discovery API

Potential need:

- command palette module;
- help UI;
- module gallery introspection.

Research question:

- can `--modules-debug`/docs cover it, or does runtime need a read-only command listing?

---

## F6. Module storage primitive

Potential need:

- shortcuts user config;
- clipboard pins;
- snippets;
- module settings.

Current pattern:

- modules write their own files.

Research question:

- is a safe per-module data directory primitive needed?

---

# Suggested implementation order

## Wave 0 — Packaging and release first

Status: completed locally in continuity session `CONT-2026-04-25-0858-wave0-packaging-release`; GitHub workflow execution remains external validation.

1. [x] `RELEASE_CHECKLIST.md`.
2. [x] Install/update documentation.
3. [x] Binary signing research.
4. [x] Changelog/release notes baseline.
5. [x] Safe module example packaging policy.
6. [x] GitHub release workflow.

## Wave 1 — First post-freeze modules

1. System actions module.
2. Clipboard module.
3. Recent files module.
4. Workspace/project launcher module.
5. Browser/bookmarks module.
6. Windows settings module.

## Wave 2 — Make the frozen platform shine

1. `MODULES_GALLERY.md`.
2. Module templates.
3. README/demo polish.
4. Release-mode benchmark baseline.

## Wave 3 — Power user modules

1. Git module.
2. Snippets module.
3. Services/processes module.
4. AI/local assistant module.

## Wave 4 — Authoring tooling

1. Module check tool.
2. Module test harness.
3. Module pack tooling.
4. Distribution conventions.

## Wave 5 — API research

1. Track friction from real modules.
2. Write decisions for repeated gaps.
3. Prototype externally first.
4. Only then consider v2 primitives.

---

# Immediate next candidates

Current preferred order:

1. **Release checklist** — closes the project operationally.
2. **GitHub release workflow** — makes releases repeatable.
3. **Installer/update story** — decides how users get and update `rmenu`.
4. **Binary signing research** — clarifies Windows trust/SmartScreen tradeoffs.
5. **System actions module** — first high-value post-freeze module.

Recommended first move:

```text
Verify the GitHub Actions release workflow on GitHub, then move to the System actions module.
```
