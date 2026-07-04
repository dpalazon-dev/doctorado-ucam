# AI Architecture

## Purpose

This document defines the **AI Operating Layer** of ResearchOS: the intelligence that operates the Research Operating System.

It does not describe models, prompts, frameworks or providers.

It defines the layer that captures, curates, links, reasons and produces continuously on behalf of the researcher — and, crucially, how that layer relates to the world it operates over.

This document is the **hub of a four-part cognitive architecture**. It defines the actor. The substrate that actor depends on — what it remembers, what it focuses on, what it reacts to — is defined in three companion models:

- **Memory Model** — what the system remembers across time.
- **Context Model** — what the system assembles for a single task.
- **Event Model** — what the system reacts to.

This document establishes the layer and its relationship to those three. Each model is defined in its own document.

It is, for the intelligent behaviour of the platform, what the Domain Model is for its structure.

---

# Relationship with the Architecture

The architectural documentation progresses from abstract concepts toward concrete behaviour, and then from behaviour toward cognition.

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
┌─── Cognitive Architecture ──────────────────┐
│   AI Architecture      ← this document       │
│   Memory Model         (forthcoming)         │
│   Context Model        (forthcoming)         │
│   Event Model          (forthcoming)         │
└──────────────────────────────────────────────┘
    ↓
Software Architecture   (future)
    ↓
Infrastructure          (future)
```

The documents above the cognitive cluster answer three questions.

- Why the system exists.
- What exists inside it.
- What it is able to do.

The cognitive cluster answers a fourth.

> Who does the operational work, with what memory, on what focus, in reaction to what.

The prior documents modelled the *world* — its entities, its knowledge, its use cases. They did not model the system's *cognition* at the same depth. The cognitive architecture corrects that, applying to the intelligent behaviour of the platform the same rigour the Domain Model applied to its structure: model the concepts first, decide the implementation later.

This cluster sits above the future Software Architecture, which will define how it is implemented. It defines *what* the intelligence is and needs. It does not define *how* the intelligence runs.

---

# The Central Idea

Every knowledge system eventually faces the same question: who keeps it organized?

The conventional answer is the user. The researcher files documents, tags notes, maintains links, resolves duplicates and curates the collection. The system stores; the human organizes. Over time the organizing becomes work, the work is skipped, and the collection decays into an archive nobody trusts.

ResearchOS answers differently.

> The researcher captures. The system organizes.

The intelligent layer is not an assistant the researcher occasionally consults. It is the **librarian** of the Research Operating System. The researcher's responsibility ends at capture. Everything after that — extraction, linking, consolidation, retrieval, maintenance — is the system's responsibility.

But the deeper architectural move is this.

> ResearchOS separates the model of the world from the intelligence that operates over it.

The world-model — the Domain Model and the Knowledge Model — is **authoritative and durable**. It is the truth.

The intelligence — the AI Operating Layer — is **active and disposable**. It reads the world, acts on it and leaves it changed, but it holds no truth of its own.

Models will be replaced. Prompts will be rewritten. Agents will be redesigned. None of that touches the world-model. The intelligence is a tenant of the domain, never its owner.

This separation is the defining innovation of the platform. Everything in this document follows from it.

ResearchOS is not a knowledge base a researcher maintains.

It is a durable model of a research world, **operated continuously by a disposable intelligence**.

---

# The Cognitive Architecture

The intelligence does not operate in a vacuum. To act well, it needs three things: something to remember with, something to focus with, and something to react to.

These are not features of the AI layer. They are distinct concerns, each large enough to be its own conceptual model, and each kept deliberately orthogonal to the others.

| Concern      | Question it answers                          | Nature of the state      | Defined in       |
|--------------|----------------------------------------------|--------------------------|------------------|
| **Memory**   | What does the system remember over time?     | State that **persists**  | Memory Model     |
| **Context**  | What does the system assemble for one task?  | State that is **produced** (transient) | Context Model    |
| **Events**   | What does the system react to?               | State that **changes** (and triggers) | Event Model      |
| **AI Layer** | Who acts?                                     | The **actor**            | this document    |

Persist, produce, trigger, act.

These four concerns never overlap, and that is precisely what keeps them separable. Memory is not context: memory is everything the system holds; context is the small, task-shaped slice assembled from it. Context is not events: context is what an operation runs *on*; an event is what causes an operation to *begin*. The AI layer is none of them: it is the actor that reacts to events, builds context from memory and the domain, and operates on the domain.

The complete conceptual stack.

```
                        Researcher
                            │
                            ▼
                 Research Operating System
   ═══════════════════════════════════════════════════════
     World Model            (what exists — authoritative)
       Domain Model · Knowledge Model
   ═══════════════════════════════════════════════════════
     Cognitive Substrate    (what the intelligence needs)
       Memory Model · Context Model · Event Model
   ═══════════════════════════════════════════════════════
     AI Operating Layer     (the intelligence that acts)
       Capture · Curate · Link · Reason
       Plan  · Write  · Audit · Notify
   ═══════════════════════════════════════════════════════
     Shared Services        (how it runs — Software Architecture, future)
       Retrieval · Scheduling · Tool Access · Policy Enforcement
   ═══════════════════════════════════════════════════════
     Infrastructure         (future)
```

Notice what is missing from this diagram: a layer of *agents*.

There is none. Agents are not an architectural layer. They are specialized behaviours of the AI Operating Layer. The architecture is memory, context, events and the layer that binds them — not a directory of bots.

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

The AI never assumes it already holds the relevant state. It assembles it — deliberately, per task — from memory and the domain.

The quality of any intelligent action is bounded by the quality of the context constructed for it. How that construction works is defined in the Context Model.

### 5. The world-model is the single source of truth

This principle extends System Principle 6.

The intelligent layer operates over the domain entities defined in the Domain Model. It never maintains a private, parallel representation of the researcher's knowledge.

There is no shadow store. No document folder, index, vector store or generated wiki is ever the source of truth. Any such structure is a *projection* of the domain — reconstructible from it, never authoritative over it. Storage formats may change; the domain does not. The AI reasons over the domain, never over the storage.

### 6. Agents do not know the world

An agent never navigates storage. It never thinks *"open this file"* or *"query that table."*

It expresses a need for meaning — *"I need the Knowledge related to X"* — and the system resolves how to satisfy it.

This decouples the intelligence from the representation entirely. The agent knows what it needs, not where it lives. Retrieval strategy, storage layout and indexing can change beneath it without the agent ever noticing. An intelligence that knows where things are stored is an intelligence coupled to today's implementation.

### 7. Irreversible actions require human approval

The AI may act autonomously on reversible operations.

Irreversible actions — deleting knowledge, overwriting the researcher's work, sending outward-facing communication — always require explicit approval.

Autonomy is earned per behaviour and bounded by reversibility. See *Autonomy Levels*.

### 8. Behaviour is composed, not invented

The intelligent layer introduces no new behaviour.

Everything it does is a composition of the capabilities defined in System Capabilities. This keeps the behavioural vocabulary of the platform stable no matter how sophisticated the intelligent layer becomes.

---

# The AI Operating Layer

The intelligence of ResearchOS is not a collection of independent tools bolted onto the platform.

It is a single architectural layer — the **AI Operating Layer** — positioned between the cognitive substrate (Memory, Context, Events) and the software implementation.

The AI Operating Layer operates *over* the Domain Model, never *around* it.

It reads and writes the same entities every other part of the system uses. It produces no private state. Its work is visible in the domain, as changes to Knowledge, Documents, Tasks, Activities and their relationships.

This is what makes the intelligence coherent rather than fragmented: it does not sit beside the system, it operates the system.

## What is an Agent

The platform will eventually decompose the AI Operating Layer into specialized units. It is tempting to call these units *bots* or *assistants*. Both terms mislead, and so does a third assumption — that an agent is something that knows its way around the system's storage.

> An agent is a specialized responsibility of the AI Operating Layer that operates over the Domain Model — by requesting meaning, never by navigating storage — to maintain, enrich or apply the system's knowledge.

An agent is defined by a responsibility, not by a conversation, and not by knowledge of where data lives.

It is not a chatbot. It is not a personality. It is not a file-system navigator. It is a coherent slice of the intelligent layer's work, scoped to a responsibility, bounded by an autonomy level, and blind to the physical shape of the world it operates on.

We do not design bots.

We design behaviours of the system.

---

# Responsibilities of the AI Operating Layer

The AI Operating Layer carries a set of standing responsibilities.

They are not commands the researcher issues. They are continuous obligations the layer fulfils, in the same way System Responsibilities defines the permanent obligations of the platform as a whole.

Each responsibility is a *lens* on the intelligent layer's work. Each is realized by composing capabilities defined in System Capabilities.

| Responsibility | Purpose                                                        | Primary Capabilities         |
|----------------|----------------------------------------------------------------|------------------------------|
| **Capture**    | Bring new information into the system on the researcher's behalf | Acquire, Process             |
| **Curate**     | Improve and maintain existing knowledge                         | Organize *(+ Curate¹)*       |
| **Link**       | Build and strengthen relationships between entities             | Organize (Relate)            |
| **Reason**     | Produce insight, comparison, synthesis and recommendation       | Reason                       |
| **Plan**       | Turn intent and state into scheduled, trackable work            | Operate (Plan, Schedule)     |
| **Write**      | Produce artifacts that communicate and document work            | Produce                      |
| **Audit**      | Inspect the knowledge base for inconsistency and decay          | Reason, Organize *(+ Curate¹)* |
| **Notify**     | Surface what the researcher needs to know                        | Operate (Notify)             |

*¹ Curate is not yet part of the Capability Model. See below.*

Two responsibilities that earlier drafts placed here have deliberately moved out of the layer, into the cognitive substrate:

- **Constructing context** is not a responsibility of the AI layer — it is a concern large enough to be its own model. The layer *uses* context; the Context Model defines how it is built.
- **Monitoring for change** is likewise not owned here — it belongs to the Event Model. The layer *reacts* to events; it does not own their detection.

The layer acts. The substrate remembers, focuses and detects. Keeping these apart is what the cognitive architecture is for.

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

# What the Layer Requires

The AI Operating Layer depends on the three models of the cognitive substrate. This document states *what* it requires from each. The models define *how* each requirement is met.

## From Memory

The layer cannot operate on the present moment alone. Reasoning, context construction and long-term continuity all depend on the ability to recall prior state, prior decisions and prior reasoning.

The layer requires, at minimum:

- **Immediate recall** — the state relevant to the current interaction.
- **Conversational recall** — what has been said and done within the active session.
- **Semantic recall** — the knowledge the system holds (see Knowledge Model).
- **Historical recall** — what happened before: past Activities, decisions and outcomes.
- **Procedural recall** — how the system has learned to perform recurring work.

These are requirements, not a design. How memory is structured, retained, retrieved and consolidated is defined in the **Memory Model**.

## From Context

No intelligent operation acts on the entire system. Acting on everything is impossible at scale and counterproductive — relevance is destroyed by noise.

The layer therefore requires that every operation run on a **constructed context**: a focused, task-shaped view assembled on demand from memory and the domain. Context is a *product*, not a copy of memory (Design Principle 4).

The layer's only obligation is to never assume context — to always request its construction. How context is constructed, prioritized and bounded is defined in the **Context Model**.

## From Events

A system whose intelligence activates only on request is a search box. A system whose intelligence activates on *change* is an operating layer.

The layer is therefore **reactive**: it acts in response to changes in the domain, not only in response to the researcher. This is what makes *Knowledge evolution is continuous* (Principle 3) operationally real rather than aspirational.

The layer requires a stream of domain events to react to. The event taxonomy, the triggers and the reactive flows are defined in the **Event Model**.

---

# Agent Coordination

The AI Operating Layer is not a single agent. Nor is it a chain of agents calling one another.

The naïve design — agent A invokes agent B invokes agent C — is explicitly rejected. It produces inconsistent context, duplicated work, runaway cost and brittle coupling. The moment one agent must know how to call the next, the intelligence is coupled to its own decomposition.

Coordination happens **through shared state and events**, never through direct calls.

```
Shared State (the Domain)
        +
Events (changes to it)
        +
Specialized Workers (the responsibilities)
```

A worker reacts to an event, reads context built from shared state, does its work, and writes the result back to the domain. That write is itself a change — a new event — which another worker may react to in turn. No worker hands off to another. No worker holds the state. The domain holds the state; events announce that it changed; workers react.

This is a direct consequence of Design Principle 5. Because the domain is the single source of truth, coordination needs no separate message-passing substrate between agents. The domain is the medium of coordination, and the Event Model is its nervous system.

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

It defines *what* the intelligent layer is, *what* it is responsible for, *what* it requires from the cognitive substrate and *how far* it may act on its own. It does not define how any of this is built.

The cognitive substrate is defined in three companion models, to be written next:

- **Memory Model** — the types of memory, how they persist, evolve and relate to the domain.
- **Context Model** — how context is constructed, prioritized and bounded for each task.
- **Event Model** — the domain-event taxonomy, triggers and reactive flows that keep the system alive.

Later work will also define:

- **The formal status of Curate** — its promotion, or not, into System Capabilities.
- **The formal status of Artifact** — its promotion, or not, into the Domain Model.
- **The status of Policy** — governance, access control and the constraints that bound autonomy, as a cross-cutting concern rather than a standalone model.
- **Agent decomposition** — the specific responsibilities carved out as agents, and their coordination.
- **Software Architecture** — the Shared Services (retrieval, scheduling, tool access, policy enforcement) and the systems that implement this layer.

The conceptual commitments made here are expected to remain stable.

Models will change. Providers will change. Retrieval strategies will change.

The intelligent layer's relationship to the researcher — augmentation, never substitution; librarian, never burden; operator of the domain, never owner of a shadow copy of it — should not.
