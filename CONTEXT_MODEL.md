# Context Model

## Purpose

This document defines how Doctorado_UCAM focuses.

Context is the task-shaped view of the system's state, assembled for a single operation. It is what an intelligent operation actually runs on.

This document defines that view: what context is, why it must be constructed rather than assumed, how it is built from its sources, and how it is prioritized and bounded.

It does not describe retrieval algorithms, ranking functions or relevance scoring. Those are realizations, chosen later, in the Software Architecture. This document defines the concept of context in a way that must survive every change beneath it.

It owns one concept: Context.

Context is not an entity. It emerges from the relationships between entities (Domain Model → Context) and is the operational view of the current system state (System Model → Context). This document gives that emergent concept its construction.

---

# What Context Is

Context is the operational view assembled for one task.

It is produced, used once, and discarded. It is never stored, never authoritative, and never a source of truth.

Context is not memory.

Memory is everything the system could recall. Context is the small, deliberate selection assembled for this task. Memory is the well; context is the bucket drawn from it for a specific purpose (see Memory Model → Recall).

This distinction is the reason the two are separate models. Memory answers *what can be recalled*. Context answers *what to assemble, right now, for this*.

System Principle 2 names Context the Primary Asset. This document is how that asset is produced.

---

# Context Is Selection

Everything the system knows is available to every task.

That is the problem, not the solution.

An operation given everything is an operation given nothing: signal drowns in noise, and relevance is destroyed by volume. The larger the system's memory grows — across years of research — the more acute this becomes. A mature Research Operating System could, in principle, surface thousands of related items for any task. Almost none of them belong in the context.

Context is therefore a deliberate act of **exclusion**.

Its value lies as much in what it leaves out as in what it includes.

Memory and Context are complementary disciplines. Memory *retains* everything worth keeping. Context *excludes* everything not worth attending to now.

This is the central tension the model resolves, per task:

```
Too little context  →  the operation is uninformed.
Too much context    →  the operation is unfocused.
```

Relevance under a budget. Every constructed context is a resolution of that trade-off, and getting it right is the single strongest determinant of the quality of an intelligent action.

---

# Context Is Constructed, Never Assumed

This elaborates Design Principle 4 of the AI Architecture.

Every intelligent operation begins by constructing the context it needs. The layer never assumes it already holds the right state.

The reason is simple: the right context is different for every task. The context that served the last operation is, almost always, wrong for this one. A system that assumes context is a system that acts on stale or irrelevant state.

The AI Operating Layer's only obligation regarding context is therefore negative: it must never assume it, and must always request its construction, per task (see AI Architecture → What the Layer Requires).

---

# The Construction Pipeline

Context is built by a deliberate progression from a task to a bounded, focused view.

```
Task
  ↓
Intent             what is this operation really trying to achieve?
  ↓
Anchors            which domain entities does it concern?
  ↓
Expand             follow relationships to connected entities
  ↓
Recall             semantic · episodic · procedural memory
  ↓
Recent Change      what changed lately that bears on this
  ↓
Constrain          apply policy: access · sensitivity · autonomy
  ↓
Prioritize & Bound rank by relevance; fit the budget; drop the rest
  ↓
Context
```

Each stage has a distinct role.

- **Intent** — the operation's real goal, inferred. Relevance is impossible without it: the same anchor entity demands entirely different context depending on what the task intends to do with it. Intent is what makes selection possible.
- **Anchors** — the domain entities the task directly concerns: the Project, the Paper, the Hypothesis. The starting points.
- **Expand** — following relationships outward from the anchors to connected entities. This is where the intentionally highly-connected domain (Domain Model → Relationship Philosophy) pays off: connection *is* context.
- **Recall** — surfacing relevant memory: knowledge that bears on the task, prior episodes that echo it, learned routines for performing it. When the task needs source material rather than distilled meaning, recall reaches to the Documents themselves.
- **Recent Change** — incorporating what has changed lately that is relevant, drawn from the Event Model.
- **Constrain** — applying policy: removing what this operation may not see, on grounds of access, sensitivity or its own autonomy bounds.
- **Prioritize & Bound** — the hard step. Ranking everything gathered by relevance to the intent, keeping what fits the budget, and deliberately discarding the rest.

The result is a context: a focused, bounded, task-shaped view, handed to the operation.

---

# The Sources of Context

Context draws on four sources and one constraint. It owns none of them.

| Source / constraint | Contributes                       | Owned by        |
|---------------------|-----------------------------------|-----------------|
| **Memory**          | Recall — knowledge, history, how-to | Memory Model  |
| **The Domain**      | Anchors and relationships          | Domain Model    |
| **Events**          | Recent, relevant change            | Event Model     |
| **Documents**       | Source material, when needed       | Document domain |
| **Policies**        | What may enter (a constraint)      | *cross-cutting* |

Context is, precisely, the act of drawing these together into one focused view. It contributes no new state of its own. It is composition, not creation.

---

# Policies Are an Input, Not a Model

Policy shapes what may enter a context: access rights, data sensitivity, and the autonomy bounds of the operation being served.

It is a gate applied during construction — not a memory type, and not a model of its own.

Where policy comes from is defined elsewhere: the autonomy bounds in AI Architecture → Autonomy Levels, and their eventual enforcement in the future Software Architecture (Policy Enforcement). This document treats policy only as a constraint the pipeline must honor when deciding what a given operation is permitted to see.

Policy is a fifth influence on context, deliberately *not* promoted to a fifth model. It is a cross-cutting concern realized across several documents, consistent with how the AI Architecture flagged it.

---

# Context and Retrieval Are Not the Same

The system can *retrieve* — locate and access information (System Capabilities → Retrieve; Domain Map → Search & Retrieval).

Context construction uses retrieval, but is far more than it.

Retrieval finds. Context infers intent, follows relationships, incorporates recent change, applies constraints, prioritizes and bounds. Retrieval answers *"where is information about X?"* Context answers *"what does this operation need to see, and what must it not?"*

A system with excellent retrieval and no context discipline floods every operation with results that are individually relevant and collectively useless.

Retrieval is a capability. Context is a discipline.

---

# Context Is Transient

Context is built per task and discarded after it.

For the life of an operation, the constructed context populates the working memory that operation runs on (Memory Model → Working Memory). When the operation ends, both are gone.

Context is never authoritative and never a source of truth. If a construction is ever cached to avoid rebuilding it, that cache is a projection — reconstructible, never authoritative — the same discipline the Memory Model applies to storage.

This also distinguishes context from an Artifact. An Artifact is a durable, researcher-facing product of reasoning (AI Architecture → Generated Artifacts). Context is ephemeral, internal scaffolding for a single operation. One is a deliverable; the other is never seen.

---

# Context Is a Concept, Not a Mechanism

This model defines what context is and how it is shaped — not how it is computed.

The relevance-under-a-budget problem has many possible solutions, and the choice among them will change as the system grows and as technology improves. That choice is deferred to the Software Architecture.

What is fixed here is the discipline: context is intent-driven, drawn from defined sources, constrained by policy, and always prioritized and bounded. These properties must hold no matter how relevance is scored or how the budget is enforced beneath them.

> The Context Model must survive every change in the layers beneath it.

The strategies will change. The discipline should not.

---

# Relationship to Other Documents

To avoid duplication, this document does not redefine shared concepts.

- The recall it draws on — Memory Model
- The recent change it incorporates — Event Model
- The entities and relationships it traverses — Domain Model; Domain Map → Context; System Model → Context
- Finding information, as a capability — System Capabilities → Retrieve
- The autonomy bounds that constrain it — AI Architecture → Autonomy Levels
- The layer that constructs and consumes it — AI Architecture

The Context Model owns one concept: how the system focuses.

---

# Future Evolution

This model fixes the concept and defers everything mechanical.

Later work will define:

- **Relevance strategy** — how relevance to intent is scored and selected, in the Software Architecture.
- **Budgeting strategy** — how the bound is set and enforced for different classes of task.
- **Context reuse** — whether, and how, a constructed context may be partially retained across closely related tasks without ever becoming a source of truth.
- **The formal status of Policy** — governance and access control as a cross-cutting concern rather than a standalone model.

The concept defined here is expected to remain stable while every mechanism beneath it evolves.
