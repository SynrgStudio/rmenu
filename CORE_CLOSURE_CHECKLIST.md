# CORE CLOSURE CHECKLIST — rmenu

Status: Operational draft  
Goal: close the `rmenu` core as a stable platform and move future feature work to modules.

---

## Guiding principle

The core is closed when it can support real modules without frequent structural changes.

Main rule:

> If a feature can be implemented as a module, it does not belong in the core.

Future core changes are accepted only for:

- critical bug,
- crash,
- security/isolation,
- performance,
- Windows compatibility,
- v1 contract correction,
- general need demonstrated by several real modules.

---

# Phase 1 — Documentation, vocabulary, and architecture boundary

Status: mostly complete.

## 1.1 Architecture constitution

- [x] Create `MODULES_ARCHITECTURE.md`.
- [x] Define what the `rmenu` core is.
- [x] Define what the core is not.
- [x] Define the core as authority over UI, ranking, state, execution, and policy.
- [x] Define that modules extend by composition, not mutation.
- [x] Define boundaries between core, module runtime, module host, and user modules.
- [x] Define v1 API stability policy.
- [x] Define future breaking-change policy.
- [x] Define criteria for accepting a new core primitive.
- [x] Define criteria for rejecting a feature and sending it to a module.

## 1.2 Official v1 vocabulary

- [x] Freeze official v1 module vocabulary.
- [x] Document Providers.
- [x] Document Commands.
- [x] Document Decorations.
- [x] Document Input Accessory.
- [x] Document Capabilities.
- [x] Document Key Hooks.
- [x] Document Runtime Actions.
- [x] State that no new v1 primitives are added without proven need.

## 1.3 Explicit module limits

- [x] Modules cannot draw UI directly.
- [x] Modules cannot access Win32/GDI.
- [x] Modules cannot modify global layout.
- [x] Modules cannot replace the ranking engine.
- [x] Modules cannot mutate arbitrary state.
- [x] Modules cannot bypass capabilities.
- [x] Modules cannot depend on core internals.
- [x] Modules must operate through public hooks, `ctx`, and actions.

## 1.4 Public specs freeze

- [x] Review `MODULES_API_SPEC_V1.md`.
- [x] Review `RMOD_SPEC_V1.md`.
- [x] Review `MANIFEST_SPEC_V1.md`.
- [x] Review `CTX_ACTIONS_SPEC_V1.md`.
- [x] Review `PROVIDER_EXECUTION_POLICY.md`.
- [x] Review `ERROR_ISOLATION_POLICY.md`.

## 1.5 Future evolution policy

- [x] Add Core Change Policy to `MODULES_ARCHITECTURE.md`.
- [x] Require new features to be attempted as modules first.
- [x] Clarify that a new capability does not imply a new primitive.
- [x] Require evidence from real modules for new primitives.
- [x] Define API v2 proposal process.
- [x] Define deprecation process.
- [x] Document future architecture decisions in `DECISIONS.md`.

## 1.6 Documentation cleanup

- [x] Keep official specs at repository root.
- [x] Move historical docs to `docs/historico/`.
- [x] Remove or move ambiguous duplicates.
- [x] Update `README.md` to present `rmenu` as launcher + module platform.
- [x] Add module documentation links.
- [x] Document diagnostic commands: `--metrics`, `--debug-ranking`, `--modules-debug`, `--reindex`.
- [x] Create install/develop/debug module guides.
- [x] Translate public root documentation/specs to English for GitHub readiness.

## 1.7 Project metadata

- [x] Review `Cargo.toml`.
- [x] Replace placeholder author.
- [x] Review package description, version, license, include list, and `.gitignore`.

---

# Phase 2 — Validation with real modules, without core-specific features

Goal: prove that the current core is enough to build useful extensions.

## 2.1 Real module: calculator

Status: manually validated.

Implementation:

- File: `modules/calculator.rmod`.
- Detects simple calculations typed directly, with no mandatory `=` prefix.
- Shows result as `=<result>` in the input bar through `InputAccessory` kind `success`.
- Uses `ctx.replaceItems([])` to clear fuzzy results while the calculation is valid.
- Declares only `input-accessory`.
- Contains no calculator logic in the core.

Friction found and resolved:

- External host action return path was enabled for `setInputAccessory`, `clearInputAccessory`, and `replaceItems`.
- Input accessory rendering no longer shows `[success]` prefix.
- Hooks without declared capability are not invoked, avoiding noisy `permission_denied` logs.
- The UI cycle respects `replaceItems([])`.

Checklist:

- [x] Create calculator module.
- [x] Detect calculation queries.
- [x] Show result through Input Accessory.
- [x] Decide that provider item is not needed for current UX.
- [ ] Optional: define copy/use-result UX through an official action.
- [x] Declare minimal capabilities.
- [x] Confirm no calculator-specific core logic.
- [x] Document friction.

## 2.2 Real module: scripts/commands

Status: validated with `modules/local-scripts.rmod`, except optional namespaced commands.

UX decision:

- `>` lists local scripts.
- `> term` filters local scripts.
- Without `>`, the global launcher is unchanged.
- The module uses `ctx.replaceItems(...)` for scoped mode and avoids competing with global fuzzy/core ranking.

Checklist:

- [x] Create module that lists local scripts/commands.
- [x] Use badges/hints/subtitles for metadata.
- [ ] Optional: register stable namespaced commands.
- [x] Execute action without direct access to internals.
- [x] Confirm dedupe/ranking is acceptable in scoped mode.
- [x] Manually confirm script execution with Enter.
- [x] Confirm no conceptual core change is required.
- [x] Document friction.

## 2.3 Real module: shortcuts / quick actions

Status: implemented with `modules/shortcuts.rmod`; pending manual Enter validation.

UX decision:

- exact shortcut aliases activate only when `input.trim()` equals a configured `key` or `alias`;
- `1` and `b` activate Blender in the default demo config;
- `bl`, `b foo`, and `1 foo` do not activate shortcuts and fall back to normal launcher search;
- the shortcut row shows the alias cue as a badge, for example `[b]`;
- Enter executes the selected target through the normal launcher path.

Checklist:

- [x] Create a shortcut/quick-action module.
- [x] Use `ctx.replaceItems([item])` for exact alias matches.
- [x] Show visual keybinding cue via item badge.
- [x] Declare capabilities required for binding flow (`input-accessory`, `commands`, `keys`).
- [x] Use external `ctx.selectedItem()` snapshot for `Ctrl+B` binding flow.
- [x] Use external `ctx.setQuery(...)` action to prefill `/shortcuts::bind `.
- [ ] Manually confirm Blender launches via `b` + Enter.
- [ ] Manually confirm Blender launches via `1` + Enter.
- [ ] Manually confirm non-exact inputs do not activate shortcuts.
- [x] Manually confirm plain text input remains immediate with `shortcuts` loaded.
- [ ] Manually confirm adding a new shortcut with `Ctrl+B` + `/shortcuts::bind <alias>`.
- [x] Confirm core changes are v1 contract corrections/general actions, not shortcut-specific behavior.
- [x] Document behavior.

## 2.4 Friction review

- [x] Classify current known frictions as bug, missing docs, ergonomics, feature-specific need, or real primitive gap.
- [x] Prefer docs fixes before code if ambiguity is the issue.
- [x] Reject core changes that only improve one isolated case.
- [x] Accept core changes only when they unblock several modules or correct the contract.

Known frictions:

1. External modules needed real action return path to core — resolved as a v1 contract bug.
2. Input accessory rendered `[success]` — resolved as UI primitive rendering bug.
3. Undeclared hooks generated noisy permission errors — resolved as capability routing bug.
4. Exact provider intent vs global fuzzy ranking — resolved as documentation/pattern issue for `local-scripts` v2 through explicit `>` prefix + `ctx.replaceItems(items)` scoped intent mode, without adding a new primitive.
5. Shortcut keybinding vs key hooks — resolved for v1 as exact search aliases using `ctx.replaceItems([item])`, plus a `Ctrl+B` binding flow that uses general external ctx snapshot and `setQuery` support. Plain text keys are not dispatched to module key hooks and hot query snapshots omit items to avoid input latency.

---

# Phase 3 — Functional hardening of the existing core

Pending:

- [ ] Confirm each external module runs isolated from the core.
- [ ] Confirm request timeout works.
- [ ] Confirm hung hosts are killed or degraded correctly.
- [ ] Confirm auto-restart respects backoff.
- [ ] Confirm auto-disable after thresholds.
- [ ] Confirm successful reload resets relevant counters.
- [ ] Confirm one module's errors do not break others.
- [ ] Confirm `--modules-debug` exposes state, errors, telemetry, and capabilities.
- [ ] Confirm mixed discovery of directory modules and `.rmod` files.
- [ ] Confirm clear errors for invalid `.rmod` and `module.toml`.
- [ ] Confirm disabled modules are not loaded.
- [ ] Confirm deterministic priority.
- [ ] Confirm hot reload and debounce behavior.
- [ ] Confirm capability enforcement for providers, commands, input accessory, and keys.
- [ ] Confirm provider budgets, timeouts, item caps, sanitization, dedupe, and command collisions.
- [ ] Confirm UI primitive rendering for badges, hints, subtitles, input accessory, quick select, and narrow widths.
- [ ] Confirm IPC payload limits and invalid payload handling.

---

# Phase 4 — Tests and verification

## 4.1 Automated tests to add

- [ ] Valid/invalid `.rmod` parser tests.
- [x] Duplicate/missing block tests.
- [ ] Valid/invalid `module.toml` tests.
- [x] Mixed loader tests.
- [ ] Capability allow/deny tests.
- [ ] Provider timeout and item cap tests.
- [ ] Dedupe tests.
- [ ] Command namespace/collision tests.
- [ ] IPC payload limit tests.
- [ ] Hot reload tests.
- [ ] Host restart/backoff tests.
- [ ] Auto-disable tests.
- [x] Duplicate quick-select tests.
- [x] Input accessory priority/kind tests.

## 4.2 Local verification commands

- [ ] `cargo fmt`
- [x] `cargo check`
- [x] `cargo test`
- [ ] `cargo clippy` if adopted by the project flow.
- [ ] `rmenu --metrics`
- [ ] `rmenu --debug-ranking <query>`
- [x] `rmenu --modules-debug`
- [ ] Manual launcher mode test.
- [ ] Manual stdin/script mode test.

## 4.3 Minimum performance targets

- [ ] Define startup budget.
- [ ] Define search p95 budget.
- [ ] Define time-to-window-visible budget.
- [ ] Define provider budget.
- [ ] Confirm slow modules do not severely degrade UI.
- [ ] Confirm index cache invalidation and `--reindex`.

---

# Phase 5 — Product/core polish

Pending:

- [ ] Validate base launcher UX without modules.
- [ ] Confirm no fuzzy/ranking regression.
- [ ] Confirm friendly app labels and executable-name search.
- [ ] Confirm ShellExecuteW launch and controlled fallback.
- [ ] Confirm persistent history.
- [ ] Review `config_example.ini`.
- [ ] Document `[Modules]` config fully.
- [ ] Confirm safe defaults and invalid-config behavior.
- [ ] Define standard module location, examples, naming, capabilities, command namespace, sharing, and versioning.

---

# Phase 6 — Freeze declaration

Do not start yet.

Blockers:

- need manual validation of the third real module (`shortcuts`),
- need hardening,
- need specific tests,
- need minimum performance validation.

## Final checklist before freeze

- [ ] `MODULES_ARCHITECTURE.md` exists and is approved.
- [ ] v1 specs are reviewed and frozen.
- [ ] README reflects current architecture.
- [ ] Duplicate/historical docs are ordered.
- [ ] Project metadata is clean.
- [ ] Three real modules work without touching core.
- [ ] Module frictions are classified.
- [ ] Blocking bugs are fixed.
- [ ] Tests/verifications are green.
- [ ] Minimum performance is validated.
- [ ] Future core-change policy is written.

## Formal declaration

- [ ] Create `CORE_FREEZE_V1.md` or equivalent section in `MODULES_ARCHITECTURE.md`.
- [ ] Declare freeze date.
- [ ] Declare frozen core scope.
- [ ] Declare frozen module API.
- [ ] Declare allowed future core changes.
- [ ] Declare that new features must be implemented as modules first.

---

# Definition of Done — Core Closed v1

The `rmenu` core is considered closed when:

- the core/module boundary is documented,
- v1 vocabulary is frozen,
- public specs do not contradict each other,
- real modules validate the API,
- the external runtime is robust against errors,
- capabilities are enforced,
- v1 UI primitives are sufficient for useful modules,
- main tests/checks pass,
- the repository is ordered,
- and explicit policy prevents accidental core expansion.

After that, primary work moves to modules, templates, docs, visual polish within existing primitives, bug fixes, performance, and controlled future evolution.
