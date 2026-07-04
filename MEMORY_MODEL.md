# Memory Model

## Purpose

This document defines how ResearchOS remembers.

Memory is the system's capacity to retain state and recall it across time. It is what separates a system with continuity from a model that begins every interaction blank.

This document defines that capacity: what the system remembers, how memory divides, how it is written, consolidated, recalled and forgotten, and how it relates to the domain.

It does not describe databases, caches, vector stores or indexes. Those are realizations, chosen later, in the Software Architecture. This document defines the concept of memory in a way that must survive every change beneath it.

It owns one concept: Memory.

For memory as a system capability, see Domain Map → Memory and Domain Model → Memory. This document gives that capability its structure.

---

# What Memory Is

Memory is a faculty, not an entity.

Consistent with Domain Model → Memory, the system has no "Memory" object. Memory is the ability to retain and recall — a capability that operates over state the system already holds.

Its importance is easy to underestimate.

The intelligence of ResearchOS depends far less on the reasoning model it uses than on the memory it can bring to bear. A powerful model with no memory is a stranger every morning. A modest model with excellent memory accumulates understanding for years.

Memory is what makes the librarian a librarian rather than a search box.

---

# The Defining Division

Memory divides along a single question: **where does the truth live?**

The answer splits all of the system's memory into two kinds, and the split matters more than any individual memory type.

## World Memory

Retention of the world itself.

World memory is **durable and authoritative**. It is the source of truth. It is, quite literally, the domain — the Knowledge, the Activities and the decisions that compose the researcher's operational reality.

The Memory Model does not own world memory. The domain does. Memory contributes only the faculty of recall over it.

## Operating Memory

Retention the intelligence needs in order to function.

Operating memory is **disposable and never authoritative**. It is the intelligence's own working state and learned skill. It supports the act of operating on the world; it is not part of the world.

The asymmetry between the two is the whole point.

```
Delete all operating memory   →  the world is intact.
Delete world memory           →  knowledge is lost.
```

This is the separation principle of the AI Architecture — the durable world-model versus the disposable intelligence — applied to memory. World memory outlives every model, prompt and agent. Operating memory belongs to them and dies with them.

```
                        Memory
                          │
          ┌───────────────┴───────────────┐
          ▼                               ▼
     World Memory                   Operating Memory
  (durable · truth)              (disposable · support)
          │                               │
   ┌──────┴──────┐                 ┌──────┴───────┐
   ▼             ▼                 ▼              ▼
 Semantic     Episodic          Working      Procedural
(Knowledge)  (Activities)      (live op)    (learned how)
```

The four types below realize the five recalls the AI Operating Layer requires (see AI Architecture → What the Layer Requires): immediate and conversational recall are Working memory; semantic recall is Semantic memory; historical recall is Episodic memory; procedural recall is Procedural memory.

---

# World Memory

## Semantic Memory

Semantic memory is what the system knows.

It is Knowledge, exactly as defined in the Knowledge Model. This document does not redefine it.

Memory's contribution is recall: the faculty of surfacing the right knowledge when it bears on the moment. How knowledge is born, evolves and is represented belongs to the Knowledge Model; how it is *recalled* belongs here.

Semantic memory is authoritative. It is the closest thing the system has to what it "believes."

### Does NOT include

- Files — those are Documents.
- The construction of a task's view — that is Context.

## Episodic Memory

Episodic memory is what happened.

It is the accumulated, recallable record of Activities, decisions and their outcomes over time. Where semantic memory holds what is *true*, episodic memory holds what *occurred*.

Its raw material already exists in the domain. An Activity (Domain Model → Activity) is a single episode. Episodic memory is the faculty of recalling those episodes meaningfully — connecting a decision made months ago to the situation that now echoes it.

This is what Domain Map → Memory means by "transforming isolated events into long-term understanding."

Episodic memory is authoritative. What happened, happened.

---

# Operating Memory

## Working Memory

Working memory is the live state of the current operation.

It has two scopes: the immediate interaction (this turn, this reasoning step) and the active session (the ongoing exchange). It holds what must stay coherent while an operation is in flight.

It is volatile. When the operation ends, working memory is discarded. Nothing of value is lost, because anything worth keeping has, by then, been consolidated into world memory.

Working memory is never authoritative. It is a scratchpad.

## Procedural Memory

Procedural memory is how the system has learned to perform recurring work.

It is the shape of tasks done before: the routine for ingesting a paper, the pattern of a sound literature review, the steps a recurring audit follows. It improves *how* the system works, never *what* it knows.

It is refined over time by the intelligence itself — Audit and Curate observing which routines succeed and consolidating them.

### Does NOT include

The reasoning model's own trained knowledge. That knowledge lives in a model the system treats as a replaceable tool (see *What We Deliberately Do Not Model*). Procedural memory is the system's own learned routine, external to any model, and it survives the model being swapped.

---

# Preferences Are Not a Memory Type

Many agent systems introduce a separate "user" or "persona" memory for preferences.

ResearchOS does not.

A researcher's preferences are Knowledge about a Person (Domain Model → Person). They are recalled semantically and applied procedurally. Introducing a separate preference store would duplicate what the domain already represents and would violate the single-source-of-truth principle.

Preference is a use of memory, not a kind of it.

---

# The Memory Lifecycle

Memory is not static storage. It is a cycle of four conceptual operations, each independent of any technology that might implement it.

```
Encode  →  Consolidate  →  Recall  →  Forget
```

## Encode

State enters memory.

World memory is encoded through Capture, Curate and the occurrence of Activities. Operating memory is encoded by the intelligence as it works.

## Consolidate

Transient memory becomes durable; raw memory becomes summarized.

This is the step that prevents memory from degenerating into a verbatim transcript. Working memory worth keeping is consolidated into episodic or semantic memory — distilled, connected and stripped of noise. A session becomes a handful of remembered facts, not a full recording.

Consolidation is what makes memory grow in value rather than merely in size.

## Recall

Relevant retained state is surfaced.

Recall is the primitive the Context Model builds upon. Memory answers *what can be recalled*; Context decides *what to assemble for this task*. Memory is the well; context is the bucket drawn from it for a specific purpose.

This document owns recall. It does not own construction. That boundary is what keeps the Memory Model and the Context Model separate.

## Forget

Memory decays deliberately.

Operating memory is forgotten freely — that is its nature. World memory is never silently forgotten. Consistent with the Knowledge Model, knowledge is never silently overwritten; it is superseded, with its provenance preserved, so that past reasoning remains recoverable.

Two kinds of memory, two disciplines of forgetting: the operating kind is cleared; the world kind is only ever layered.

---

# Memory and the Domain

Most of the system's memory is not a separate store. It is the domain, recalled.

| Memory type | Belongs to       | Realized over          | Authoritative |
|-------------|------------------|------------------------|---------------|
| Semantic    | World memory     | Knowledge              | Yes           |
| Episodic    | World memory     | Activities · decisions | Yes           |
| Working     | Operating memory | the intelligence       | No            |
| Procedural  | Operating memory | the intelligence       | No            |

This table carries the model's most important consequence.

ResearchOS needs no bolted-on "memory database" at the conceptual level. Its long-term memory is the Unified System State (see System Model → Unified System State), recalled. The domain is not merely data the system stores — it is the system's memory.

This is why the world-model can be authoritative and the intelligence disposable: the memory that matters was never the intelligence's to begin with.

---

# Memory Is a Concept, Not a Store

This model names faculties, never mechanisms.

Working memory is not a cache. Semantic memory is not an index or a graph. Episodic memory is not a log. Each of those is a *realization* — a technology chosen later, in the Software Architecture, to serve a concept defined here.

The concepts are structured so that realization has a clean seam to plug into: each memory type has a defined role, a defined owner and a defined lifecycle, and could be served by one technology or by several without any concept changing.

This is a deliberate discipline, and it is the reason this document exists at the conceptual level at all.

> The Memory Model must survive every change in the layers beneath it.

When the storage changes — and it will — this document should not.

---

# What We Deliberately Do Not Model

The literature on agent memory offers many finer distinctions. Most are realizations, not concepts, and the model omits them on purpose.

- **No "vector memory" or "graph memory."** These are ways to realize recall, not kinds of memory. They belong to the Software Architecture.
- **No "resource vault."** Resources are domain entities (Domain Model → Resource). Sensitive-data handling is a matter of Policy, not of memory.
- **No persona store.** Preferences are Knowledge about a Person.
- **No parametric or model memory.** The reasoning model is a replaceable tool. Whatever it "knows" internally is not the system's memory and must never be relied upon as such.

Selecting deliberately is itself part of the model. A memory taxonomy that mirrored every mechanism would not survive those mechanisms changing.

---

# Relationship to Other Documents

To avoid duplication, this document does not redefine shared concepts.

- Semantic memory and knowledge evolution — Knowledge Model
- Recall versus construction — Context Model
- Writes and consolidation triggered by change — Event Model
- Memory as a capability and cross-cutting domain — Domain Map → Memory; Domain Model → Memory
- The intelligence that uses memory — AI Architecture

The Memory Model owns one concept: how the system remembers.

---

# Future Evolution

This model fixes the concepts and defers everything mechanical.

Later work will define:

- **Realization** — the stores, caches and indexes that implement each memory type, in the Software Architecture.
- **Consolidation policy** — when and how working memory is distilled into episodic and semantic memory.
- **Forgetting policy** — the retention and supersession rules for each memory type, including governance and the right to erasure.
- **The boundary of procedural memory** — how learned routines are captured and reused without hardening into rigid workflows.

One concept surfaced here is a candidate for the Domain Model. **Decision** recurs across episodic memory, Domain Map → Memory and System Responsibilities, yet is not a domain entity. If it continues to recur, it becomes a candidate for promotion — as Knowledge, as a specialization of Activity, or as a Derived Type of its own. That decision belongs to the Domain Model, recorded deliberately.

The concepts defined here are expected to remain stable while every mechanism beneath them evolves.
