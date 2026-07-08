# Interaction Model

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 2 — Specialized Model |
| **Normative status** | Canonical within interaction scope |
| **Authoritative for** | Workspace concept, interaction channels, human/system initiative, feedback, notification and approval semantics. |
| **Not authoritative for** | Visual design system, frontend framework, API shape, domain mutation internals or model-provider interaction. |
| **Required reading** | `VISION.md`, `SYSTEM_PRINCIPLES.md`, `SYSTEM_MODEL.md`, `USE_CASES.md`, `AI_ARCHITECTURE.md`, `SYSTEM_ARCHITECTURE.md`. |
| **Downstream documents** | `SOFTWARE_ARCHITECTURE.md`, `IMPLEMENTATION_CONTEXTS.md`, interface design and inbound adapters. |

> Scope and conflict rules are defined in `DOCUMENTATION_ARCHITECTURE.md`.

---

> **Status: Canonical.** This document defines how the researcher and the system collaborate.
>
> The Domain Model defines *what exists*.
> The AI Architecture defines *how the system reasons*.
> The Interaction Model defines *how work flows between the human and the system*.
>
> It is intentionally independent of any interface technology, implementation framework or visual design.

---

# Purpose

ResearchOS is not a chatbot.

Neither is it a traditional desktop application.

It is an **Operating System** whose intelligence continuously collaborates with the researcher while remaining largely invisible.

The objective of this document is to define that collaboration.

It answers questions such as:

- How does the researcher interact with the system?
- When does the AI participate?
- Which actions are purely software?
- Which actions require reasoning?
- When may the system initiate work itself?
- How do all interaction channels coexist?

This document specifies the interaction architecture, not the interface.

---

# Position in the Architecture

The conceptual architecture is built from three complementary perspectives.

```
                 WHAT EXISTS
                Domain Model
                     │
                     │
          HOW THE SYSTEM THINKS
            AI Architecture
                     │
                     │
       HOW HUMAN AND SYSTEM WORK
         Interaction Model
```

None replaces another.

Together they define the complete behaviour of the operating system.

---

# Core Principle

The researcher never interacts directly with:

- the Knowledge Graph
- Memory
- Context
- Events
- Agents
- LLMs

Those are internal mechanisms.

The researcher interacts with a **Workspace**.

The Workspace is the operational environment where human and system collaborate.

---

# The Workspace

The Workspace is the single operational surface of the system.

It exposes information, accepts actions and adapts continuously as work evolves.

```
                  Researcher
                       │
             interacts with
                       │
                ┌────────────┐
                │ Workspace  │
                └────────────┘
                       │
         ┌─────────────┼─────────────┐
         │             │             │
      UI Actions    AI Requests   Notifications
                       │
                 Operating System
```

The Workspace is not a screen.

It is an abstract interaction environment that may be realized through multiple interfaces.

---

# Interaction Channels

The Workspace supports multiple interaction channels simultaneously.

No channel is privileged.

Each exists because it is optimal for particular tasks.

| Channel | Purpose |
|----------|----------|
| Navigation | Move through projects, documents and knowledge |
| Forms | Structured editing |
| Dashboards | Continuous situational awareness |
| Search | Explicit information retrieval |
| Timeline | Operational history |
| Graphs | Relationship exploration |
| Tables | Structured management |
| Kanban | Task organization |
| Calendar | Time management |
| Commands | Fast expert interaction |
| Conversation | Natural language collaboration |
| Notifications | System-initiated communication |

These channels cooperate.

None replaces the others.

---

# Interaction Taxonomy

Every interaction belongs to one of five categories.

---

## 1. Direct Interaction

The researcher performs an explicit operation.

No reasoning is required.

Examples

- Create a task
- Edit a document
- Move a card
- Open a project
- Change a deadline
- Browse knowledge

This is traditional software behaviour.

The AI does not participate.

---

## 2. Assisted Interaction

The researcher performs the action.

The system contributes intelligence.

Examples

- Summarize a document
- Suggest tags
- Link related knowledge
- Detect duplicates
- Recommend references
- Propose task priorities

The researcher remains the decision maker.

The AI augments rather than replaces.

---

## 3. Conversational Interaction

Natural language becomes the interaction mechanism.

Examples

- Explain this concept.
- Compare these papers.
- Build a reading plan.
- What changed this week?
- Which hypotheses remain unsupported?

Conversation is one interaction channel among many.

It is never the operating system itself.

---

## 4. Autonomous Interaction

The system initiates work without being explicitly requested.

Examples

- New relevant publication detected.
- Contradictory evidence discovered.
- Project deadline approaching.
- Resource exhaustion predicted.
- Duplicate concepts identified.
- Weekly review prepared.

Autonomous interaction is proactive rather than reactive.

Its behaviour is constrained by policies defined in the AI Architecture.

---

## 5. Ambient Interaction

The system continuously improves the Workspace without explicit interaction.

No notification is required.

No conversation occurs.

Examples

- Active memory changes.
- Context is rebuilt.
- Panels reorganize.
- Recommendations evolve.
- Search ranking adapts.
- Relevant documents appear automatically.

Ambient interaction is the characteristic behaviour of an intelligent operating system.

The researcher experiences a Workspace that continuously adapts to the current situation.

---

# Interaction Flow

Every interaction follows the same conceptual cycle.

```
Researcher
      │
      ▼
Interaction
      │
      ▼
Workspace
      │
      ▼
Operating System
      │
 ┌────┼────┐
 │    │    │
Domain Memory Context
 │    │    │
 └────┼────┘
      ▼
AI Operating Layer
      │
      ▼
Result
      │
      ▼
Workspace
      │
      ▼
Researcher
```

The intelligence operates beneath the Workspace.

The Workspace remains the stable interaction surface.

---

# Human-Initiated Flow

The traditional interaction flow.

```
Researcher

↓

Action

↓

Workspace

↓

Operating System

↓

Result
```

Examples include editing, searching, navigating or requesting assistance.

---

# System-Initiated Flow

The operating system may initiate interaction.

```
Domain Event

↓

Operating System

↓

Reasoning

↓

Recommendation
Notification
Prepared Action

↓

Researcher
```

The researcher may ignore, postpone or accept the proposal.

The operating system never assumes authority beyond its configured autonomy.

---

# Degrees of Autonomy

Not every intelligent behaviour has the same authority.

ResearchOS defines progressive autonomy levels.

| Level | Behaviour |
|--------|-----------|
| 0 | Observe only |
| 1 | Produce recommendations |
| 2 | Prepare actions awaiting confirmation |
| 3 | Execute approved automatic actions |
| 4 | Coordinate long-running operational plans within policy constraints |

Autonomy is a policy decision, not an implementation detail.

Different capabilities may operate at different levels.

---

# Interaction Principles

The interaction architecture follows these principles.

## Workspace First

The Workspace is the primary interaction surface.

The conversation is one tool inside it.

---

## Software Before AI

Operations that require no reasoning should not invoke reasoning.

Traditional software remains the fastest solution whenever sufficient.

---

## Intelligence When Valuable

The AI participates only where reasoning provides meaningful value.

Reasoning is a capability, not a dependency.

---

## Continuous Assistance

The system continuously improves the Workspace through background reasoning.

The researcher should benefit from intelligence without constantly requesting it.

---

## Human Authority

The researcher remains responsible for decisions.

The operating system recommends, prepares and automates only within explicit autonomy policies.

---

## Explainability

Every recommendation must be explainable.

The researcher must always be able to understand why the system acted.

---

## Progressive Disclosure

Complexity appears only when necessary.

The Workspace exposes the minimum information required for the current context.

---

## Context Adaptation

The Workspace adapts to the current operational context.

The researcher should not need to manually reconfigure the system for every task.

---

## Multiple Interaction Modalities

Every capability should be accessible through the interaction mechanism that best suits it.

Conversation is never mandatory.

---

# Relationship to AI Architecture

The AI Architecture defines how intelligence operates.

The Interaction Model defines how that intelligence becomes visible.

```
AI Architecture

↓

Reasoning

↓

Interaction Model

↓

Workspace Behaviour
```

This separation allows the intelligence to evolve independently from the user experience.

---

# Relationship to the Domain

Every interaction ultimately produces change in the domain.

```
Interaction

↓

Domain Change

↓

Domain Event

↓

Memory Update

↓

Context Update

↓

Future Interactions
```

Interaction is therefore part of the same operational cycle defined throughout the architecture.

---

# Scope

This document defines:

- interaction philosophy
- interaction channels
- collaboration model
- autonomy levels
- operational flows
- interaction principles

It intentionally does not define:

- visual design
- layouts
- screens
- widgets
- navigation trees
- component libraries
- implementation technologies

Those belong to the Software Architecture and the future User Experience documentation.

---

# Relationship to Other Documents

To avoid duplication, this document complements rather than replaces other models.

- **Domain Model** defines what users manipulate.
- **AI Architecture** defines how intelligence reasons.
- **Memory Model** defines what can be recalled.
- **Context Model** defines what becomes active.
- **Event Model** defines how change propagates.
- **System Capabilities** define what the system can do.
- **Use Cases** define concrete operational scenarios.

The Interaction Model defines only one thing:

> **How the researcher and the operating system collaborate while performing work.**

Everything visible to the researcher is ultimately an expression of this collaboration.
