Overview

rmenu implements a modular architecture where the core remains minimal, stable, and deterministic, while optional functionality is layered through extensions ("modules").

The system is designed to achieve:

Minimal, stable core
Composable behavior
Extensible ecosystem
Consistent UX regardless of modules
Strong separation of concerns
Core Philosophy
1. Core is authoritative

The core is responsible for:

Input handling
Query state
Ranking/filtering
Selection
Submission
Rendering
Lifecycle
Module orchestration

Modules never override core internals.

2. Modules declare intent, not implementation

Modules:

React to events
Provide data
Request actions
Attach metadata

They do not:

Render UI directly
Mutate internal state arbitrarily
Override ranking engine
Hook into private structures
3. Visual consistency is enforced by the core

Modules can describe visual cues, but only the core renders them.

This ensures:

UI consistency
Stable API
Decoupling from rendering backend
Architecture Model

The extension system is built around four primitives:

1. Hooks
2. Context (ctx)
3. Actions
4. Contributions
Hooks

Modules subscribe to lifecycle and interaction events.

Available hooks (v1)
onLoad(ctx)
onUnload(ctx)

onQueryChange(query, ctx)
onSelectionChange(item, index, ctx)

onKey(event, ctx)
onSubmit(item, ctx)
onCommand(command, args, ctx)

// visual/data enrichment
decorateItems(items, ctx) => Item[]
Context (ctx)

A controlled interface to runtime state.

Read-only
ctx.query()
ctx.items()
ctx.selectedItem()
ctx.selectedIndex()
ctx.mode()
ctx.hasCapability(name)
Utilities
ctx.log(message)
ctx.toast(message)
Actions (allowed mutations)

Modules can request changes via actions:

ctx.setQuery(text)
ctx.setSelection(index)
ctx.moveSelection(offset)

ctx.submit()
ctx.close()

ctx.addItems(items)
ctx.replaceItems(items) // restricted contexts

ctx.registerCommand(commandDef)
ctx.registerProvider(providerDef)
Contributions

Modules contribute behavior and data in controlled ways.

1. Providers

Primary mechanism for injecting content.

ctx.registerProvider({
  name: "example",
  command: "/example",
  query: async ({ query }) => {
    return [...]
  }
})
2. Commands
ctx.registerCommand({
  name: "/scripts",
  title: "Show scripts"
})
3. Decorations

Visual metadata attached to items.

4. Capabilities

Behavioral metadata attached to items.

Item Model

All UI elements share a unified structure.

type Item = {
  id: string
  title: string
  subtitle?: string
  source?: string

  action: Action

  capabilities?: {
    quickSelectKey?: string
  }

  decorations?: {
    badge?: string
    badgeKind?: "shortcut" | "status" | "tag"
    hint?: string
    icon?: string
  }
}
Capabilities vs Decorations
Capabilities

Describe behavior.

Example:

quickSelectKey: "1"
Decorations

Describe presentation.

Example:

badge: "1"
badgeKind: "shortcut"
Critical Rule

Modules declare metadata.
Core decides behavior and rendering.

Visual System (Badges / Chips)
Example: Quick Shortcut

Module sets:

item.capabilities.quickSelectKey = "1"

Core automatically:

Renders badge 1
Applies theme styling
Handles keypress 1
Selects or executes item
Rendering Policy

Modules cannot:

Draw UI directly
Control layout
Define pixel positions
Use Win32/GDI directly

Core handles:

Layout
Styling
Rendering
Clipping
Theme integration
Enrichment Pipeline
Providers → Core filtering/ranking → Modules decorateItems → Core render → Input handling
Module Types
1. Behavior Modules

Modify interaction patterns.

Examples:

Numeric shortcuts
Custom navigation
Input behaviors
2. Content Modules

Provide items.

Examples:

Scripts
Bookmarks
Windows
Clipboard history
Declarative vs Scripted Modules
Declarative Modules

For simple configuration:

Themes
Static items
Aliases
Rules

Example:

[theme]
bg = "#111111"
fg = "#ffffff"
Scripted Modules

For logic:

Hooks
Providers
Commands
Integrations
Module Structure
modules/
  my-module/
    module.toml
    index.js
module.toml
name = "numeric-shortcuts"
version = "0.1.0"
kind = "script"
entry = "index.js"
capabilities = ["keys", "decorate-items"]
Hot Reload (v1)
File change detected
Module unloaded
Module reloaded
Commands/providers re-registered
Errors logged

No:

state migration
partial patching
complex lifecycle recovery
What Modules MUST NOT Do

Modules cannot:

Access internal state directly
Modify ranking engine
Hook into render loop
Override input system globally
Use internal APIs
Access cache directly
Break invariants
Quick Select (Reference Implementation)
Module
export function decorateItems(items, ctx) {
  return items.map(item => {
    if (item.id === "app:blender-5.0") {
      return {
        ...item,
        capabilities: {
          ...item.capabilities,
          quickSelectKey: "1"
        }
      }
    }
    return item
  })
}
Core
Detect quickSelectKey
Render badge
Handle keypress
Maintain consistency
Design Constraints
API must remain small
Behavior must remain predictable
Rendering must remain centralized
Modules must remain replaceable
Future Compatibility

This architecture enables:

External daemons (automation, scripting)
Ecosystem modules
Command-driven workflows
Multi-process integrations

Without modifying the core.

Final Principle

rmenu is not extended by mutation.
It is extended by composition.