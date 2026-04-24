# CORE_FINALIZATION_REPORT — rmenu

## Overview

This document defines the current state and finalization strategy of the `rmenu` core after the introduction of modular runtime, IPC, and `.rmod` distribution.

The purpose is to:

- Clarify what now constitutes the **true core**
- Prevent uncontrolled architectural growth
- Establish a transition from **building → stabilizing**
- Define discipline for future evolution

---

## 1. Scope Expansion — Reframed

### Observation

The project has evolved from:

- a simple Win32 launcher

into a system including:

- launcher
- modular runtime
- module host (external process)
- IPC
- module descriptor system
- `.rmod` format
- hot reload
- policy engine

---

### Interpretation

This is **not scope creep**.

This is:

> The completion of the *actual core architecture*

The core is no longer just the executable, but the **minimal system required to support extensibility as a first-class concept**.

---

### Core Now Includes

- Module discovery
- Descriptor normalization
- Runtime execution model
- IPC boundaries
- Policy enforcement (timeouts, budgets, dedupe)
- UI extension primitives (decorations, input accessory)
- Hot reload mechanism

---

### Conclusion

> The system has reached a **core-complete state**

From this point forward:

- Growth must slow
- Structure must stabilize
- Vocabulary must freeze

---

## 2. Missing Piece: Living Architecture Document

### Problem

Architectural knowledge is currently split between:

- code
- implementation plan
- external reasoning (conversations, mental model)

This creates fragility.

---

### Requirement

A central document must exist:


MODULES_ARCHITECTURE.md


---

### Purpose

To act as a **constitution** for the modular system.

---

### Must Define

#### Core Identity
- Core is authoritative
- Modules extend via composition, not mutation

#### Module Contract
- Hooks
- Context
- Actions
- Contributions

#### Supported Primitives
- Providers
- Commands
- Decorations
- Capabilities
- Input accessory
- Key hooks

#### Explicit Boundaries
Modules MUST NOT:
- Render UI directly
- Access Win32/GDI
- Mutate core state arbitrarily
- Override ranking engine
- Modify layout

---

### Conclusion

> Without this document, the architecture exists in code but not in shared understanding.

---

## 3. Repository Maturity Signals

### Issue

`Cargo.toml` contains placeholder metadata:

```toml
authors = ["Tu Nombre <tu.email@ejemplo.com>"]
Why It Matters

The project now presents itself as:

structured
modular
extensible
performance-oriented

This inconsistency reduces perceived maturity.

Required Fixes
Replace placeholder author
Review description
Ensure metadata consistency
Align manifest with project identity
Conclusion

Small details now have large perception impact.

4. Modular API Expansion Risk
Current State

The system exposes multiple extension surfaces:

hooks
commands
providers
decorate-items
input accessory
key handling
IPC lifecycle
Risk

Any new feature can now be “fit somewhere”

This leads to:

uncontrolled API expansion
conceptual drift
increasing complexity
harder refactoring
Required Action

Freeze the conceptual vocabulary.

Define Official Primitives

The system should stabilize around:

Providers
Commands
Decorations
Capabilities
Input Accessory
Key Hooks
Runtime Actions
Rule

Before adding new primitives:

Can this be expressed with existing primitives?
Is this reusable or feature-specific?
Does this expand the language or just convenience?
Conclusion

The system is at the exact point where it must stop expanding and start stabilizing.

5. Strategic Direction (Next Phase)
STOP Doing
Adding new core features
Expanding API surfaces
Introducing new primitives casually
START Doing
1. Freeze Architecture

Lock in current concepts and naming.

2. Document

Create and maintain MODULES_ARCHITECTURE.md.

3. Validate with Real Modules

Build real modules using only existing API:

numeric shortcuts
calculator (input accessory)
simple provider (/scripts)
4. Observe Pain Points

Only after real usage:

adjust API
remove redundancies
refine boundaries
Goal

Move from:

"Flexible architecture"

to:

"Proven, minimal, stable system"

6. Final Principle

The core is no longer being built.
It is now being defined and protected.

7. Ultra Summary
The scope increase was necessary to define the real core
The system is now core-complete at an architectural level
A formal architecture document is required immediately
Minor repo details (like Cargo.toml metadata) must be fixed
API expansion must stop; vocabulary must freeze
Next phase is validation, not invention
Closing Statement

rmenu has transitioned from a tool into a system.

The next step is not to grow it —
but to make sure it does not grow uncontrollably.