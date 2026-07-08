# Event Model

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 2 — Specialized Model |
| **Normative status** | Canonical within event semantics |
| **Authoritative for** | What an event means, event categories, lifecycle, reaction discipline, causality and relationship to memory and context. |
| **Not authoritative for** | Broker selection, outbox/inbox mechanics, transport guarantees, transaction implementation or process deployment. |
| **Required reading** | `SYSTEM_PRINCIPLES.md`, `DOMAIN_MODEL.md`, `SYSTEM_CAPABILITIES.md`, `SYSTEM_ARCHITECTURE.md`. |
| **Downstream documents** | `AI_ARCHITECTURE.md`, `MEMORY_MODEL.md`, `CONTEXT_MODEL.md`, `SOFTWARE_ARCHITECTURE.md`, event runtime implementations. |

> Scope and conflict rules are defined in `DOCUMENTATION_ARCHITECTURE.md`.

---

## Purpose

This document adopts the event-after-commit and state-authority rules defined in `SYSTEM_ARCHITECTURE.md`. It owns event meaning and reaction semantics; delivery mechanics belong to `SOFTWARE_ARCHITECTURE.md`.

This document defines how ResearchOS reacts.

An event is a recorded change in the domain. Reactive behaviour is what the system does in response to those changes — the work it performs without being asked.

This document defines that behaviour: what an event is, where events come from, how reactions are triggered, how they cascade, and the limits every reaction obeys.

It does not describe message brokers, queues or event buses. Those are realizations, chosen later, in the Software Architecture. This document defines the concept of an event in a way that must survive every change beneath it.

It owns one concept: the domain Event — the recorded change from which all reactive behaviour derives.

It is the last of the three models of the cognitive substrate. Where Memory defines what persists and Context defines what is assembled, this document defines what changes — and what the system does about it.

---

# What an Event Is

A domain event is a recorded fact that the domain changed.

It has three properties.

- **It happened.** An event is past tense. It is never a request, an intention or a plan. It is a fact.
- **It is a change to the domain.** Something was created, updated, derived, linked or transitioned.
- **It is addressed to no one.** An event does not know who, if anyone, will react to it.

That last property is the important one.

> Events are facts, not commands.

A command says *do this* — future tense, addressed to someone. An event says *this happened* — past tense, addressed to no one. ResearchOS is built on the second, not the first. The system is not a queue of instructions being executed. It is a standing reaction to the changing state of a research world.

---

# The System Reacts to Change

A system that acts only when asked is a tool.

A system that acts when its world changes is alive.

This is the difference the Event Model makes real. It is what turns *Knowledge evolution is continuous* (Design Principle 3 of the AI Architecture) from an aspiration into a mechanism, and it is the reactive requirement the AI Architecture deferred here.

The researcher captures a paper. A change is recorded. The system reacts — extracting knowledge, linking it, curating duplicates, proposing a next task — without being asked for any of it.

The intelligence of the system is, in large part, the sum of its reactions to facts.

---

# Events Are Grounded in State Changes

The Event Model invents almost nothing. The events already exist — the Use Cases have been naming them all along.

Every use case declares its **State Changes**, grouped as *Created*, *Updated*, *Derived* and *Linked*, with lifecycle transitions made explicit. That section is described, in the Use Cases, as *"the direct bridge from behaviour to the Domain Model and to future system events."*

This document is that bridge, formalized. A domain event is the runtime record of exactly such a state change.

| Change type          | Example event                                             |
|----------------------|----------------------------------------------------------|
| Created              | a Document was captured; a Task was proposed              |
| Updated              | a Project's objective changed                             |
| Derived              | Knowledge was extracted from a Document                   |
| Linked               | a Hypothesis was linked to Evidence                      |
| Lifecycle transition | a Hypothesis moved from *experimenting* to *evidenced*   |

Because the events are the state changes the system already makes, the Event Model needs no new vocabulary of its own. It gives a name to something the architecture has produced since the first use case.

---

# Where Events Come From

Events are classified by the change they record. They are *sourced*, orthogonally, from four origins.

- **The researcher** — captured a document, made a decision, completed a task.
- **The intelligence** — the AI Operating Layer produced, curated or linked something. A reaction's own output is itself an event.
- **Time** — a schedule fired: a periodic audit is due, a review is scheduled. Temporal events are how the system performs standing maintenance without being prompted.
- **The external world** — an email arrived, a feed carries a new paper. External signals enter as domain changes once captured.

Whatever the source, an event enters the system as a change to the domain. The domain is the single surface through which every event flows — consistent with the single-source-of-truth principle.

## Events and Activities

An event is close to, but broader than, an Activity.

An Activity occurring (Domain Model → Activity) is one kind of event: work happened. But not every event is an Activity. Linking two pieces of Knowledge changes the domain without being work the researcher did.

Activities are one source of events. Events are the general category.

---

# Triggers and Reactions

A **trigger** connects an event to a reaction.

A trigger is a subscription: *when an event of this kind occurs, this responsibility engages.* The reacting responsibilities are those of the AI Operating Layer — Capture, Curate, Link, Reason, and the rest.

The connection is deliberately decoupled.

Events do not know their subscribers. Responsibilities subscribe to events. A reaction can be added or removed without touching the event; an event can be produced without knowing what, if anything, reacts to it.

This decoupling is the mechanism behind Agent Coordination (AI Architecture). Workers coordinate through events, never by calling one another. The event is the medium; the domain is the surface; no worker ever hands off to another.

---

# Reactive Flows

A reaction produces new state changes. Those changes are themselves events. Those events may trigger further reactions.

Behaviour cascades.

```
Document created
   ↓
Capture   →  Knowledge derived · linked
   ↓
Curate    →  duplicates merged · relationships strengthened
   ↓
Reason    →  Task proposed
   ↓
Notify    →  researcher informed
```

Each arrow is not a call. It is a new event that a subscribed responsibility reacts to. The flow is *emergent* — assembled from independent reactions — not hard-wired as a fixed pipeline.

This is the system keeping itself alive. One act of capture ripples outward into extraction, curation, suggestion and notification, none of it requested, all of it a consequence of a single recorded change.

---

# Reactions Are Bounded

A cascade must terminate.

Left undisciplined, events could trigger reactions that produce events that trigger reactions, without end. The model requires that reactive flows be **bounded**: a reaction that produces no meaningful new change ends the cascade, and loops are broken rather than followed.

The mechanism that enforces this is deferred to the Software Architecture. The discipline is fixed here.

A living system must not be a runaway one.

---

# Reactions Obey Human Control

Reactive does not mean uncontrolled.

Every reaction obeys the same autonomy levels and the same invariants as any other operation (AI Architecture → Autonomy Levels, Human Control). An event may cause the system to act, but if the reaction would take an irreversible or outward-facing action, it still requires approval.

Events lower the threshold for the system *to act*. They never lower the threshold for what it may do *without permission*.

Proposing a Task in response to a new paper is an autonomous reaction. Sending that paper to a collaborator is not.

---

# Events, Memory and Context

The Event Model sits between the other two substrates and binds them.

**Events drive Memory.** A change is what there is to remember. The Activity and decision events are what episodic memory records; other domain changes — a Linked or Derived fact — update semantic memory instead. Either way, an event is what triggers consolidation (Memory Model → Encode, Consolidate). Once an event is past, its retention belongs to memory. The Event Model owns the change *as it happens*; the Memory Model owns it *afterward*.

**Events feed Context.** Recent change is one of the sources a context is assembled from (Context Model → The Sources of Context). A single event can play two roles at once: it triggers an operation, and — as recent change — it informs the context that operation runs on.

The three fit together cleanly:

> Memory remembers what happened. Context attends to what changed. Events are what happened, as it happens.

---

# Events Are a Concept, Not a Mechanism

This model defines what an event is and how the system reacts — not how events are transported, queued or delivered.

There is no message broker here. No queue. No bus. Those are realizations, chosen later, to carry a concept defined at this level.

What is fixed here is the reactive discipline: events are factual records of domain change, decoupled from their reactions, composing into bounded cascades under human control.

> The Event Model must survive every change in the layers beneath it.

The delivery mechanism will change. The reactive discipline should not.

---

# Relationship to Other Documents

To avoid duplication, this document does not redefine shared concepts.

- The state changes events record — Use Cases → State Changes; Domain Model
- Their retention after they happen — Memory Model
- Their role as recent change in a task's view — Context Model
- The responsibilities that react to them — AI Architecture → Responsibilities of the AI Operating Layer; Agent Coordination
- The limits every reaction obeys — AI Architecture → Autonomy Levels; Human Control
- Time as a transversal dimension of every event — Domain Model → Time

The Event Model owns one concept: how the system reacts to change.

---

# Future Evolution

This model fixes the concept and defers everything mechanical.

Later work will define:

- **Delivery mechanism** — how events are transported, delivered and ordered, in the Software Architecture.
- **Bounding strategy** — the concrete rules that terminate cascades and break loops.
- **Scheduling** — how temporal events are defined, fired and managed.
- **Event as a domain record** — whether an Event becomes a first-class recorded concept in the Domain Model, related to Activity and to Time, so that the system's own reactive history becomes queryable. A candidate for promotion, to be recorded deliberately.

The concept defined here is expected to remain stable while every mechanism beneath it evolves.

---

With this document the **Cognitive Architecture** is complete: the AI Operating Layer that acts, and the three models of the substrate it operates over — Memory (what persists), Context (what is assembled) and Events (what changes). Together they define not what the Research Operating System *is*, but how it *thinks*.
