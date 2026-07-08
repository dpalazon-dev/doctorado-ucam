# Documentation Architecture

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 4 — Navigation, Delivery and Process |
| **Normative status** | Meta-canonical for documentation governance |
| **Authoritative for** | Document levels, ownership, precedence, dependency semantics, duplication rules and change-impact protocol. |
| **Not authoritative for** | Product scope, domain semantics, system architecture, software structure or implementation behavior. |
| **Required reading** | `VISION.md`, `SYSTEM_PRINCIPLES.md` |
| **Downstream documents** | Every Markdown document in the repository; especially `README.md`, `CLAUDE.md` and `IMPLEMENTATION_CONTEXTS.md`. |

> When two documents appear to conflict, this document identifies which one owns the disputed concern. It does not decide the concern itself.

---

## Purpose

ResearchOS is described through multiple architectural views. Those views are useful only when each has a clear scope, a single authority boundary and an explicit relationship to the rest of the repository.

This document defines the architecture of the documentation itself.

Its objective is to make the repository operable by both humans and coding agents without requiring every document to be loaded for every task. It establishes:

- four documentation levels;
- the authoritative owner of each major concern;
- the meaning of required and downstream reading;
- precedence when statements overlap;
- rules for normative language and duplication;
- the change-impact process;
- the protocol used to assemble task-specific implementation context.

The documentation is a governed graph, not a linear pile of Markdown files.

---

## Core Rule

> **Every concern has one authoritative owner. Other documents may summarize, apply or derive from that concern, but they do not redefine it.**

A repeated statement does not create a second source of truth.

When a document restates a rule owned elsewhere, the repetition is informative unless the document contract explicitly declares ownership of that concern.

---

# The Four Documentation Levels

The levels describe the role a document plays. They are not a blanket precedence hierarchy: scope ownership always determines authority.

```text
Level 1 — Canonical Foundations
        ↓ constrain
Level 2 — Specialized Models
        ↓ inform
Level 3 — Functional and Vertical Specifications
        ↓ drive
Level 4 — Navigation, Delivery and Process
        ↓ packages context for
Implementation
```

## Level 1 — Canonical Foundations

These documents define the smallest stable core that governs the whole system.

| Document | Canonical responsibility |
|---|---|
| `VISION.md` | Why ResearchOS exists, what problem it solves and the intended system outcome. |
| `SYSTEM_PRINCIPLES.md` | Permanent cross-cutting principles that constrain every later decision. |
| `SYSTEM_ARCHITECTURE.md` | System-wide authority, trust boundaries, dependency rules, operating cycles and architectural invariants. |
| `LOGICAL_DOMAIN_MODEL.md` | Canonical technology-neutral structure of the domain: entities, attributes, relationships, lifecycles and invariants. |
| `SOFTWARE_ARCHITECTURE.md` | Canonical implementation-neutral software structure: modules, ports, adapters, transactions, runtimes and conformance rules. |

Level 1 is deliberately small. A coding agent should not infer that every other document is optional; it should infer that global decisions must have a home in this core.

## Level 2 — Specialized Models

These documents own one bounded conceptual concern. They deepen the system without redefining global architecture.

| Document | Specialized responsibility |
|---|---|
| `SYSTEM_MODEL.md` | Highest-level conceptual composition of assets, capabilities, interfaces and shared state. |
| `RESEARCHER_OPERATIONAL_MODEL.md` | The operational reality, pressures and recurring work of the researcher. |
| `SYSTEM_RESPONSIBILITIES.md` | Permanent responsibilities the system derives from that operational reality. |
| `DOMAIN_MAP.md` | Conceptual responsibility domains and their relationships. |
| `DOMAIN_MODEL.md` | Meaning and relationships of the seven Core Entities. |
| `KNOWLEDGE_MODEL.md` | Semantics, forms and evolution of Knowledge. |
| `AI_ARCHITECTURE.md` | Responsibilities and behavior of the AI Operating Layer. |
| `MEMORY_MODEL.md` | Memory taxonomy, recall and consolidation semantics. |
| `CONTEXT_MODEL.md` | Context definition, construction inputs, lifecycle and boundaries. |
| `EVENT_MODEL.md` | Event semantics, taxonomy and reaction discipline. |
| `INTERACTION_MODEL.md` | Human-system collaboration, interaction channels, initiative and approval semantics. |

A Level 2 document may be canonical within its declared scope. It may not override a Level 1 invariant.

## Level 3 — Functional and Vertical Specifications

These documents define behavior, vocabulary and domain breadth.

| Document | Specification responsibility |
|---|---|
| `SYSTEM_CAPABILITIES.md` | Stable vocabulary of what the system can do. |
| `USE_CASES.md` | Canonical recurring intentions, flows, outcomes and state changes. |
| `DOMAIN_VERTICALS.md` | Index and rules for specialization across operational verticals. |
| `RESEARCH_VERTICAL.md` | Research-specific derived types and use-case mapping. |
| `TEACHING_VERTICAL.md` | Teaching-specific derived types and use-case mapping. |
| `ADMINISTRATION_VERTICAL.md` | Administration-specific derived types and use-case mapping. |
| `ORGANIZATION_VERTICAL.md` | Organization-specific derived types and use-case mapping. |

These documents drive implementation slices but do not define software modules, storage or runtime topology.

## Level 4 — Navigation, Delivery and Process

These documents help humans and agents navigate, sequence and consume the specification.

| Document | Process responsibility |
|---|---|
| `README.md` | Repository front door and concise navigation. |
| `ROADMAP.md` | Plan of record: phases, order and current status. |
| `IMPLEMENTATION_PLAN.md` | Proving experiments, acceptance criteria and falsification strategy. |
| `CLAUDE.md` | Operating instructions for coding agents working in the repository. |
| `DOCUMENTATION_ARCHITECTURE.md` | Documentation ownership, precedence and evolution. |
| `IMPLEMENTATION_CONTEXTS.md` | Task-specific reading bundles for implementation agents. |

Level 4 documents cannot create system behavior or architecture by implication. If they need a new rule, that rule must first be introduced in the document that owns the concern.

---

# Authority Ownership Matrix

The following matrix identifies the owner of recurring concerns. The owner is the only place where the concern may be changed normatively.

| Concern | Authoritative owner | Common downstream applications |
|---|---|---|
| Project purpose and intended outcome | `VISION.md` | README, roadmap, principles |
| Cross-cutting system principles | `SYSTEM_PRINCIPLES.md` | Every model and architecture document |
| System-wide authority and trust boundaries | `SYSTEM_ARCHITECTURE.md` | AI, interaction, event and software architecture |
| Classes and ownership of system state | `SYSTEM_ARCHITECTURE.md` | Memory, context, persistence, runtime design |
| System dependency rules | `SYSTEM_ARCHITECTURE.md` | `SOFTWARE_ARCHITECTURE.md`, implementation tests |
| Software topology and module boundaries | `SOFTWARE_ARCHITECTURE.md` | Source tree, packages, components |
| Commands, queries, proposals and runtime contracts | `SOFTWARE_ARCHITECTURE.md` | Application code and tests |
| Transaction, outbox, inbox and process semantics | `SOFTWARE_ARCHITECTURE.md` | Persistence and event runtime implementation |
| Core Entity set and logical structure | `LOGICAL_DOMAIN_MODEL.md` | Domain modules, schemas, migrations |
| Conceptual meaning of Core Entities | `DOMAIN_MODEL.md` | Logical model, use cases, verticals |
| Conceptual domains and responsibility boundaries | `DOMAIN_MAP.md` | Module mapping and capability allocation |
| Knowledge semantics | `KNOWLEDGE_MODEL.md` | Memory, reasoning, research use cases |
| AI Operating Layer behavior | `AI_ARCHITECTURE.md` | Cognitive runtime implementation |
| Memory taxonomy and recall semantics | `MEMORY_MODEL.md` | Context builder, consolidation, retrieval |
| Context lifecycle and construction semantics | `CONTEXT_MODEL.md` | Context runtime and AI operations |
| Event meaning and reaction discipline | `EVENT_MODEL.md` | Event runtime and process managers |
| Human-system interaction semantics | `INTERACTION_MODEL.md` | UI, notifications, approval workflows |
| Permanent system responsibilities | `SYSTEM_RESPONSIBILITIES.md` | Capabilities, use cases and architecture evaluation |
| Capability vocabulary | `SYSTEM_CAPABILITIES.md` | Use cases, tools and application services |
| Behavioral scenarios and expected outcomes | `USE_CASES.md` | Implementation slices and acceptance tests |
| Vertical taxonomy and specialization rules | `DOMAIN_VERTICALS.md` | Individual vertical documents |
| Vertical-specific derived types and mappings | Corresponding `*_VERTICAL.md` | Use-case selection and domain validation |
| Phase ordering and project status | `ROADMAP.md` | README, CLAUDE, work planning |
| Proving experiments and falsification criteria | `IMPLEMENTATION_PLAN.md` | Prototype work and evaluation |
| Documentation governance | `DOCUMENTATION_ARCHITECTURE.md` | Every document |
| Agent context bundles | `IMPLEMENTATION_CONTEXTS.md` | Coding-agent task preparation |

---

# Document Contract

Every repository document begins with a compact contract containing:

| Field | Meaning |
|---|---|
| **Level** | Its role in the four-level architecture. |
| **Normative status** | Whether it is canonical, canonical within a bounded scope, behavioral, navigational or operational. |
| **Authoritative for** | Concerns this document alone may define or change. |
| **Not authoritative for** | Nearby concerns that belong elsewhere. |
| **Required reading** | Minimal prerequisites before changing this document. |
| **Downstream documents** | Documents that must be impact-checked when its contract changes. |

The contract is part of the specification. It should remain concise and should not become a second abstract for the document.

---

# Required Reading Semantics

`Required reading` means the minimum material a maintainer or agent must understand before making a normative change to the document.

It does not mean every listed document must be injected in full for every implementation task.

Rules:

1. Required documents are read in full when changing the document's contract or fundamental model.
2. For implementation, `IMPLEMENTATION_CONTEXTS.md` may narrow reading to named sections.
3. Transitive dependencies are not automatically loaded.
4. A document may be consulted without becoming part of the task's authoritative context.
5. When a required document conflicts with the current document outside the current document's scope, the concern owner prevails.

---

# Downstream Document Semantics

`Downstream documents` are not automatically edited whenever an upstream file changes.

They must be reviewed when the upstream change affects:

- terminology they reuse;
- assumptions they apply;
- interfaces they derive;
- behavior they specify;
- status or sequencing they report.

A purely editorial change does not require a downstream cascade.

A contract, invariant, entity, lifecycle, capability, use case or interface change does.

---

# Precedence and Conflict Resolution

When statements appear inconsistent, use this sequence:

1. **Identify the concern.** Do not compare entire documents abstractly.
2. **Find the owner** in the Authority Ownership Matrix.
3. **Apply the owner's statement** within that concern.
4. **Treat downstream restatements as derived guidance.**
5. **Open an explicit architectural decision** when the owner itself must change.

The documentation level alone does not settle every conflict.

For example:

- `MEMORY_MODEL.md` owns memory taxonomy, but cannot create a second global state authority because that belongs to `SYSTEM_ARCHITECTURE.md`.
- `SOFTWARE_ARCHITECTURE.md` owns outbox mechanics, but cannot redefine what a domain event means because that belongs to `EVENT_MODEL.md`.
- `ROADMAP.md` may sequence an implementation, but cannot create a new domain entity because that belongs to `LOGICAL_DOMAIN_MODEL.md`.

---

# Normative Language

The terms **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT** and **MAY** are normative only when used by the document that owns the concern, or by a downstream document translating that concern into a stricter implementation contract.

Rules:

- A specialized model may define normative rules inside its scope.
- A process document may not establish architectural invariants.
- A summary should use descriptive language and link to the owner.
- Repeated global rules should be reduced to a short adoption statement.
- Exact lists, schemas and use-case prose should not be copied between documents.

---

# Duplication Policy

Duplication is acceptable only when it improves local comprehension without creating a second definition.

## Allowed

- A one-paragraph summary of an upstream rule.
- A local consequence of an upstream invariant.
- A diagram showing the same concept from a different viewpoint.
- A short list of imported constraints, clearly attributed to the owner.

## Not allowed

- Maintaining the same entity schema in two files.
- Copying full use cases into vertical documents.
- Defining global authority independently in AI, memory, event or implementation documents.
- Maintaining separate roadmap status tables that disagree.
- Copying runtime contracts into conceptual models.

## Required wording for imported rules

Where confusion is possible, use a formulation such as:

> This document adopts the authority model defined in `SYSTEM_ARCHITECTURE.md`; the following section describes only its consequences for memory.

---

# Change-Impact Protocol

Every normative change follows this process:

```text
1. Identify the concern owner
        ↓
2. Change the owner document
        ↓
3. Review its declared downstream documents
        ↓
4. Update only affected summaries, mappings or derived contracts
        ↓
5. Run documentation conformance checks
        ↓
6. Record an ADR when the change is architecturally significant
```

A pull request or agent response that changes a normative concern should state:

- the concern changed;
- the authoritative owner;
- the downstream files reviewed;
- which downstream files changed and why;
- which downstream files were unaffected.

---

# Coding-Agent Consumption Protocol

A coding agent must not begin by loading every Markdown file.

It should:

1. Read `CLAUDE.md`.
2. Check the current phase in `ROADMAP.md`.
3. Select the matching bundle in `IMPLEMENTATION_CONTEXTS.md`.
4. Load the authoritative owner documents in full.
5. Load only the named sections of supporting documents.
6. State the invariants it must preserve before changing code.
7. Implement the smallest coherent slice.
8. Validate code and documentation against the conformance rules in `SYSTEM_ARCHITECTURE.md` and `SOFTWARE_ARCHITECTURE.md`.

A request such as “read all Markdown and implement ResearchOS” is invalid because it obscures authority and creates avoidable context competition.

---

# Independent Reviewability

Each document can be reviewed separately when the reviewer uses its contract.

A valid isolated review asks:

- Does the document stay inside `Authoritative for`?
- Does it defer concerns listed under `Not authoritative for`?
- Is it consistent with its required reading?
- Are downstream implications declared?
- Does it repeat a rule or genuinely specialize it?

An isolated review does not attempt to validate the entire platform from one file.

---

# Document Evolution Rules

## Add a document only when

- a concern has no existing owner;
- the concern is stable enough to deserve an independent lifecycle;
- separating it materially improves comprehension or change isolation;
- its authority and downstream relationships can be stated precisely.

## Do not add a document when

- it merely summarizes existing documents;
- it exists only to mirror a fashionable architecture template;
- it would own no unique concern;
- it would force agents to load another file without reducing ambiguity.

## Split a document when

- it owns two concerns that evolve independently;
- different implementation tasks consistently require disjoint halves;
- its required-reading set becomes incoherent.

## Merge or retire a document when

- it has no unique authority;
- its content is entirely duplicated elsewhere;
- its downstream consumers no longer exist.

Retired documents should be removed rather than left as competing historical specifications. Important historical decisions belong in ADRs or version control.

---

# Repository Layout

The repository may remain flat while it is documentation-only. Documentation levels are logical, not directories.

When source code is introduced, documents may be moved into directories only if links, agent instructions and context bundles are updated atomically.

Directory structure must not become a substitute for explicit authority.

---

# Conformance Checklist

The documentation architecture is conformant when:

1. Every Markdown document has a Document Contract.
2. Every normative concern has one owner.
3. Level 4 documents introduce no product or architecture rules.
4. Specialized models defer global authority and implementation details.
5. Functional specifications do not redefine domain structure.
6. Vertical documents reference use cases instead of copying them.
7. Required-reading sets are minimal and explicit.
8. Downstream impact can be traced.
9. `README.md`, `ROADMAP.md` and `CLAUDE.md` report the same project state.
10. Coding tasks can be prepared through a bounded context bundle.
11. Local links resolve.
12. Repeated statements cannot override their authoritative owner.

---

# Final Principle

ResearchOS should be implementable from precise, bounded context.

The documentation succeeds when a maintainer can answer three questions without loading the entire repository:

1. **Which document owns this decision?**
2. **What must I read to change or implement it safely?**
3. **What else must I review if it changes?**
