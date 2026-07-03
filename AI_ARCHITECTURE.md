# AI Architecture

## Purpose

This document defines the role of Artificial Intelligence inside Doctorado_UCAM.

It does not describe models, prompts, frameworks or providers.

Instead, it defines the intelligent layer that operates the Research Operating System: what it is responsible for, what it needs in order to operate, how autonomously it may act and where human judgement remains sovereign.

Every previous document describes a system that stores, relates and exposes knowledge. This document describes the layer that *keeps that system alive* — the layer that captures, curates, links, reasons and produces continuously, on behalf of the researcher.

It is, for the intelligent behaviour of the platform, what the Domain Model is for its structure.

---

# Relationship with the Architecture

The architectural documentation progresses from abstract concepts toward concrete behaviour.

```
Vision
    ↓
Principles
    ↓
System Model
    ↓
Operational Model
    ↓
Responsibilities
    ↓
Domain Map
    ↓
Domain Model
    ↓
Knowledge Model
    ↓
System Capabilities
    ↓
Use Cases
    ↓
AI Architecture        ← this document
    ↓
Software Architecture  (future)
    ↓
Infrastructure         (future)
```

The documents above answer three questions.

- Why the system exists.
- What exists inside it.
- What it is able to do.

This document answers a fourth.

> Who does the operational work, and how intelligently.

The prior documents describe a system that *can* be operated. This document describes the layer that *operates it*.

It sits above the future Software Architecture, which will define how this layer is implemented, and it deliberately stops short of one concern it depends on: the organization of memory. That concern is large enough to deserve its own document. This document establishes only *what* the intelligent layer needs from memory. A future **Memory Architecture** will define *how* memory is organized and used.

---

# The Central Idea

Every knowledge system eventually faces the same question: who keeps it organized?

The conventional answer is the user. The researcher files documents, tags notes, maintains links, resolves duplicates and curates the collection. The system stores; the human organizes. Over time the organizing becomes work, the work is skipped, and the collection decays into an archive nobody trusts.

Doctorado_UCAM answers differently.

> The researcher captures. The system organizes.

The intelligent layer is not an assistant the researcher occasionally consults. It is the **librarian** of the Research Operating System. The researcher's responsibility ends at capture — dropping in a paper, a note, a meeting, an idea. Everything after that — extraction, linking, consolidation, retrieval, maintenance — is the system's responsibility.

This reframing is the defining innovation of the platform.

Doctorado_UCAM is not a knowledge base that a researcher maintains.

It is a system of knowledge **continuously maintained by an intelligent layer** operating over a shared domain.

Everything in this document follows from that idea.

---

# Design Principles

The intelligent layer inherits every principle defined in System Principles. Principle 4 — *Intelligence Augments, Never Replaces* — is its constitution.

In addition, it observes the following principles specific to intelligent behaviour.

### 1. Operational, not intellectual

The AI performs operational work: capturing, organizing, linking, retrieving, drafting, maintaining.

The researcher performs intellectual work: judging, concluding, deciding, discovering.

The line between the two is never crossed. The AI prepares a decision. It never makes it.

### 2. The AI is the librarian, not the user

Organization is never the researcher's burden.

The researcher's only obligation is to capture. The system infers structure, builds relationships and maintains coherence.

A system that requires the researcher to organize has failed at its primary purpose.

### 3. Knowledge evolution is continuous

The intelligent layer maintains knowledge between interactions, not only when asked.

It links new information to old, detects contradictions, consolidates duplicates and flags decay — as a standing responsibility, not a command the researcher must remember to issue.

Knowledge that is only maintained on demand is knowledge that decays.

### 4. Context is constructed, never assumed

Every intelligent operation begins by constructing the context it needs.

The AI never assumes it already holds the relevant state. It assembles it — deliberately, per task — from the Unified System State.

The quality of any intelligent action is bounded by the quality of the context constructed for it.

### 5. The Domain Model is the single source of truth

This principle extends System Principle 6.

The intelligent layer operates over the domain entities defined in the Domain Model. It never maintains a private, parallel representation of the researcher's knowledge.

There is no shadow store. No document folder, index or generated wiki is ever the source of truth. Storage formats may change; the domain does not. The AI reasons over the domain, never over the storage.

### 6. Irreversible actions require human approval

The AI may act autonomously on reversible operations.

Irreversible actions — deleting knowledge, overwriting the researcher's work, sending outward-facing communication — always require explicit approval.

Autonomy is earned per behaviour and bounded by reversibility. See *Autonomy Levels*.

### 7. Behaviour is composed, not invented

The intelligent layer introduces no new behaviour.

Everything it does is a composition of the capabilities defined in System Capabilities. This keeps the behavioural vocabulary of the platform stable no matter how sophisticated the intelligent layer becomes.

---

# The AI Operating Layer

The intelligence of Doctorado_UCAM is not a collection of independent tools bolted onto the platform.

It is a single architectural layer — the **AI Operating Layer** — positioned between the conceptual foundation of the system and its software implementation.

```
                        Researcher
                            │
                            ▼
                 Research Operating System
   ═══════════════════════════════════════════════════════
     Conceptual Foundation
       Domain Model · Knowledge Model
       System Capabilities · System Responsibilities
   ═══════════════════════════════════════════════════════
     AI Operating Layer
       Capture · Curate · Link · Reason · Plan
       Write  · Audit  · Monitor · Construct Context
   ═══════════════════════════════════════════════════════
     Software Architecture        (future)
   ═══════════════════════════════════════════════════════
     Infrastructure               (future)
```

The AI Operating Layer operates *over* the Domain Model, never *around* it.

It reads and writes the same entities every other part of the system uses. It produces no private state. Its work is visible in the domain, as changes to Knowledge, Documents, Tasks, Activities and their relationships.

This is what makes the intelligence coherent rather than fragmented: it does not sit beside the system, it operates the system.

## What is an Agent

The platform will eventually decompose the AI Operating Layer into specialized units. It is tempting to call these units *bots* or *assistants*. Both terms mislead.

> An agent is a specialized responsibility of the AI Operating Layer that operates over the Domain Model to maintain, enrich or apply the system's knowledge.

An agent is defined by a responsibility, not by a conversation.

It is not a chatbot. It is not a personality. It is a coherent slice of the intelligent layer's work, scoped to a responsibility and bounded by an autonomy level.

We do not design bots.

We design behaviours of the system.

---

# Responsibilities of the AI Operating Layer

The AI Operating Layer carries a set of standing responsibilities.

They are not commands the researcher issues. They are continuous obligations the layer fulfils, in the same way System Responsibilities defines the permanent obligations of the platform as a whole.

Each responsibility is a *lens* on the intelligent layer's work. Each is realized by composing capabilities defined in System Capabilities.

| Responsibility        | Purpose                                                        | Primary Capabilities         |
|-----------------------|----------------------------------------------------------------|------------------------------|
| **Capture**           | Bring new information into the system on the researcher's behalf | Acquire, Process             |
| **Curate**            | Improve and maintain existing knowledge                         | Organize *(+ Curate¹)*       |
| **Link**              | Build and strengthen relationships between entities             | Organize (Relate)            |
| **Reason**            | Produce insight, comparison, synthesis and recommendation       | Reason                       |
| **Plan**              | Turn intent and state into scheduled, trackable work            | Operate (Plan, Schedule)     |
| **Write**             | Produce artifacts that communicate and document work            | Produce                      |
| **Audit**             | Inspect the knowledge base for inconsistency and decay          | Reason, Organize *(+ Curate¹)* |
| **Monitor**           | Observe changes in state and react to them                      | Operate (Track, Notify)      |
| **Construct Context** | Assemble the relevant operational view for a task               | Context *(cross-cutting)*    |

*¹ Curate is not yet part of the Capability Model. See below.*

Three of these responsibilities deserve clarification because they are easily confused.

- **Curate** is the *act* of improving knowledge — consolidating, deduplicating, refreshing.
- **Audit** is the *periodic inspection* that surfaces what needs curating.
- **Monitor** is the *continuous observation* of state that reacts to individual changes as they happen.

Audit is deep and periodic. Monitor is shallow and constant. Curate is the work both of them trigger.

## Curate — A Candidate Capability

System Capabilities defines eight capabilities: Acquire, Process, Understand, Organize, Retrieve, Reason, Produce, Operate.

None of them fully describes one recurring behaviour of the AI Operating Layer: the improvement of knowledge that already exists.

This behaviour is not Retrieve. It is not Search. It is not Reason — it produces no new knowledge. It operates on the knowledge already present and makes it better:

- reorganize knowledge
- detect and merge duplicates
- consolidate fragmented concepts
- strengthen and repair relationships
- update stale representations
- remove redundancy
- detect and resolve inconsistencies

We name this behaviour **Curate**.

Curate does not create knowledge. It increases the value of existing knowledge.

Because Curate appears across multiple responsibilities of the intelligent layer (it underlies both *Curate* and *Audit*), it satisfies the Domain Consistency test used throughout this project: a concept that recurs across behaviour is a candidate for promotion.

> **Curate is a candidate for promotion into System Capabilities.**

This document does not formalize it there — System Capabilities owns the capability vocabulary, and promotion is a deliberate act recorded through a decision. This document only identifies the gap and names it, exactly as the Domain Model treats recurring concepts as candidate Derived Types.

---

# Memory

The intelligent layer cannot operate on the present moment alone.

Reasoning, context construction and long-term continuity all depend on memory — the ability to recall prior state, prior decisions and prior reasoning.

This document establishes only *what* the intelligent layer needs from memory.

- **Immediate recall** — the state relevant to the current interaction.
- **Conversational recall** — what has been said and done within the active session.
- **Semantic recall** — the knowledge the system holds, as defined in the Knowledge Model.
- **Historical recall** — what happened before: past Activities, decisions and their outcomes.
- **Procedural recall** — how the system has learned to perform recurring work.

These are requirements, not a design.

*How* memory is organized — its structure, retention, retrieval and consolidation — is deliberately out of scope here. Memory is a cross-cutting concern large enough to distort this document if resolved inside it.

> The organization of memory is deferred to a future **Memory Architecture** document.

Until then, this document assumes only that memory exists and that the intelligent layer depends on it. See Domain Map → Memory and System Responsibilities → Responsibility 7.

---

# Context Construction

No intelligent operation acts on the entire system.

Acting on everything is both impossible at scale and counterproductive: relevance is destroyed by noise. Instead, every intelligent operation begins by constructing the *specific* context it requires.

Context is not stored. It is derived, per task, from the Unified System State (see System Model → Context).

The construction follows a deliberate progression from intent to relevant state.

```
Task
  ↓
Intent
  ↓
Relevant Entities
  ↓
Relationships
  ↓
Recent Activity
  ↓
Relevant Knowledge
  ↓
External Sources
  ↓
Context
```

The intelligent layer does not read the whole system.

It *selects*.

Given a task, it infers intent, identifies the entities the task concerns, follows their relationships, incorporates recent activity, retrieves the knowledge that bears on it and — only when necessary — reaches for external sources. The result is a context: a focused, task-shaped view of the operational state.

As the volume of knowledge grows across years, this selection problem becomes the central engineering challenge of the platform. It is not solved by a single retrieval strategy. Different tasks require different construction strategies, and the strategies themselves will evolve. This document fixes the *principle* — context is constructed, never assumed — and leaves the strategies to the Software Architecture and to future evolution.

---

# Event-Driven Behaviour

The intelligent layer does not wait to be asked.

A system whose intelligence activates only on request is a search box. A system whose intelligence activates on *change* is an operating layer.

Every meaningful change in the operational state is an event. Events drive the intelligent layer's continuous work.

A single capture illustrates the pattern.

```
New Document
     ↓
Extract Knowledge
     ↓
Link Entities
     ↓
Detect Project
     ↓
Suggest Task
```

The researcher drops a paper into the system — a single act of capture. From that one event, the intelligent layer extracts knowledge, links it to existing concepts and people, infers which project it belongs to, and proposes the next task. None of this was requested. All of it follows from the event.

This is how the librarian works while the researcher thinks.

Event-driven behaviour is what makes *Knowledge evolution is continuous* (Design Principle 3) operationally real rather than aspirational.

---

# Agent Coordination

The AI Operating Layer is not a single agent. Nor is it a set of independent agents acting in isolation.

It is a set of coordinated responsibilities.

When an event enters the system, the responsibilities engage in sequence, each consuming the output of the previous and each writing to the same shared domain.

```
Capture → Extract → Curate → Link → Audit → Notify
```

There is no private hand-off between agents, because there is no private state. Coordination happens *through the domain*: one responsibility writes an entity or relationship, and the next responsibility acts on the domain as it now stands.

This is a direct consequence of Design Principle 5. Because the Domain Model is the single source of truth, coordination requires no separate message-passing substrate. The domain is the medium of coordination.

Agents coordinate by operating on shared state, not by talking to each other.

---

# Autonomy Levels

Not every behaviour should act with the same freedom.

Autonomy is granted per behaviour, and it is bounded by one question: is the action reversible?

| Level          | Behaviour                              | Human involvement                     |
|----------------|----------------------------------------|---------------------------------------|
| **Observe**    | Reads state. Never modifies anything.  | None required.                        |
| **Suggest**    | Proposes an action and waits.          | The researcher decides.               |
| **Assist**     | Executes after explicit approval.      | Approval required per action.         |
| **Autonomous** | Executes automatically.                | Notification only. Reversible actions only. |

A behaviour begins at a low autonomy level and earns higher autonomy as it proves reliable and as the researcher chooses to grant it.

The autonomy level is a property of the behaviour, set by the researcher — never a property the AI grants itself.

Autonomous execution is permitted only for reversible actions. An irreversible action never reaches the Autonomous level, regardless of how reliable the behaviour has become. This is not a limitation to be relaxed over time. It is a permanent boundary.

---

# Human Control

Augmentation without control is substitution in disguise.

The intelligent layer is powerful precisely because it acts continuously and often without being asked. That power requires explicit, permanent limits.

The AI Operating Layer **never**:

- makes scientific or intellectual judgements — it never concludes a hypothesis is true, decides what a result means, or determines a paper's contribution
- deletes Knowledge or Documents irreversibly without approval
- overwrites the researcher's own writing, decisions or conclusions
- sends outward-facing communication without approval
- presents generated content as validated knowledge without the researcher's acceptance
- acts autonomously on any irreversible operation

The following **always** remain the researcher's:

- final scientific judgement
- acceptance of generated knowledge (see *Generated Artifacts*)
- approval of irreversible and outward-facing actions
- the autonomy level granted to each behaviour

These are not configuration defaults. They are architectural invariants. No future capability may relax them.

---

# Generated Artifacts

The intelligent layer produces outputs: reports, briefings, literature reviews, summaries, drafts, proposals.

These outputs are neither Documents nor Knowledge.

- A **Document** is an imported or authored source (see Domain Model → Document).
- **Knowledge** is validated meaning the system holds (see Knowledge Model).
- A generated output is neither. It is a *product* of reasoning, not a source and not yet validated understanding.

This recurring third category — the generated product — is a distinct concept. Provisionally, it is named an **Artifact**.

> **Artifact is a candidate for promotion into the Domain Model.**

Whether it becomes a Derived Type of Document or a new Core Entity is a decision for the Domain Model, to be recorded deliberately. Until then, an Artifact is treated as a generated Document carrying explicit provenance: what produced it, from what context, when.

## The Improvement Loop

An Artifact is not a dead end.

A useful answer, once validated by the researcher, can re-enter the system as knowledge. This closes a loop that makes the system compound in value with use.

```
Question
   ↓
Reason
   ↓
Artifact  (answer · briefing · review)
   ↓
Knowledge Candidate
   ↓
Validate ── rejected ──▶ discarded
   ↓ accepted
Knowledge
```

The loop is guarded by a single, non-negotiable gate: **validation is the researcher's act**.

The intelligent layer may generate an Artifact and may propose it as a Knowledge candidate. It may never promote an Artifact into Knowledge on its own. Only the researcher's acceptance converts a generated product into validated knowledge (see *Human Control*).

This is what allows the system to improve continuously without allowing it to contaminate its own knowledge with unvalidated output. Every question the researcher asks, and accepts an answer to, can make the next answer better — but only through a human gate.

---

# Future Evolution

This document intentionally stops at the boundary of implementation.

It defines *what* the intelligent layer is, *what* it is responsible for, *what* it needs and *how far* it may act on its own. It does not define how any of this is built.

Later work will define:

- **Memory Architecture** — how memory is organized, retained, retrieved and consolidated.
- **Context strategies** — the concrete methods by which context is constructed for different classes of task as knowledge scales.
- **The formal status of Curate** — its promotion, or not, into System Capabilities.
- **The formal status of Artifact** — its promotion, or not, into the Domain Model.
- **Agent decomposition** — the specific responsibilities carved out as agents, and their coordination.
- **Software Architecture** — the services, models and pipelines that implement this layer.

The conceptual commitments made here are expected to remain stable.

Models will change. Providers will change. Retrieval strategies will change.

The intelligent layer's relationship to the researcher — augmentation, never substitution; librarian, never burden; operator of the domain, never owner of a shadow copy of it — should not.
