UI_EXTENSION_PRIMITIVE — Input Accessory / Inline Query Preview
Overview

This document defines a UI extension primitive that allows modules to augment the input box with contextual visual feedback without modifying the core renderer.

The goal is to support features such as:

inline calculator (4+4 → 8)
command previews (/kill chrome → 3 processes)
contextual hints
conversions (15 usd → ARS …)
lightweight feedback tied to the current query

This is achieved through a controlled mechanism called:

Input Accessory (Inline Query Preview)

Design Goals
Preserve core rendering authority
Avoid exposing rendering internals (Win32/GDI/layout)
Allow contextual UI enrichment
Keep API minimal and stable
Enable multiple use cases with a single primitive
Prevent modules from breaking layout or UI consistency
Core Principle

Modules declare what should be shown
Core decides how it is rendered

Concept: Input Accessory

An Input Accessory is a small, contextual piece of UI displayed alongside the input field, typically aligned to the right.

It is:

ephemeral
query-dependent
visually secondary
controlled by the core
Visual Model

Conceptual layout:

[ user query ...................... accessory ]

Example:

[ 4+4                             8 ]
API Specification
Context Methods
Set accessory
ctx.setInputAccessory(accessory)
Clear accessory
ctx.clearInputAccessory()
Type Definition
type InputAccessory = {
  text: string
  kind?: "info" | "success" | "warning" | "error" | "hint"
  priority?: number
}
Behavior
Lifecycle
Accessory is updated on onQueryChange
Accessory is cleared when:
query becomes invalid
module explicitly clears it
launcher closes
new higher-priority accessory overrides it
Rendering Responsibility

The core is responsible for:

positioning (right-aligned)
spacing
truncation
color (based on kind)
typography
clipping
theme integration

Modules cannot control rendering details.

Supported Use Cases
1. Calculator
export function onQueryChange(query, ctx) {
  const result = tryEvaluate(query)

  if (result.ok) {
    ctx.setInputAccessory({
      text: String(result.value),
      kind: "success"
    })
  } else {
    ctx.clearInputAccessory()
  }
}
2. Command Preview
ctx.setInputAccessory({
  text: "3 processes found",
  kind: "info"
})
3. Unit Conversion
ctx.setInputAccessory({
  text: "≈ ARS 15,230",
  kind: "hint"
})
4. Contextual Hint
ctx.setInputAccessory({
  text: "Press Enter to run",
  kind: "info"
})
Conflict Resolution

Only one accessory should be visible at a time.

Strategy (v1)
Use priority
Highest priority wins
Default priority = 0

Example:

ctx.setInputAccessory({
  text: "8",
  kind: "success",
  priority: 10
})
Constraints

Modules must not:

draw directly to screen
modify input layout
control positioning
inject rendering callbacks
override inputbox painting
access renderer internals
Why This Exists

Without this primitive, supporting inline features like calculators would require:

exposing rendering internals
allowing modules to draw UI
breaking separation between logic and presentation

This primitive avoids all of that by providing:

a semantic UI slot instead of raw rendering access

Architectural Role

This is part of a broader concept:

UI Extension Primitives

Other primitives may include:

item decorations (badges, tags)
footer/status bar
inline hints
secondary panels

All follow the same rule:

modules declare → core renders

Design Tradeoffs
Chosen approach
limited power
high consistency
low coupling
stable API
Not chosen
direct render hooks
custom drawing APIs
layout overrides
widget injection
Future Extensions (Optional)

Not required for v1, but compatible:

multiple accessory slots
left + right accessories
animated transitions
richer formatting (icon + text)
structured accessory content
Summary

The Input Accessory system:

solves inline UI augmentation cleanly
avoids exposing rendering internals
supports multiple high-value features
maintains core simplicity and control
Core Rule

Modules describe UI intent.
Core renders UI reality.