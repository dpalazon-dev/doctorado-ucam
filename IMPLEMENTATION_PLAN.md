# Implementation Plan

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 4 — Navigation, Delivery and Process |
| **Normative status** | Proving-plan specification |
| **Authoritative for** | The minimum proving experiments, hypotheses, acceptance criteria, metrics and falsification conditions. |
| **Not authoritative for** | Canonical architecture, domain semantics, technology selection, production implementation sequence or project status. |
| **Required reading** | `ROADMAP.md`, `SYSTEM_ARCHITECTURE.md`, `SOFTWARE_ARCHITECTURE.md`, `LOGICAL_DOMAIN_MODEL.md`, relevant specialized models. |
| **Downstream documents** | `SPEC_CATALOG.md`, proving-slice Specs, prototype tasks, experiment reports, ADRs and Technical Architecture stabilization evidence. |

> Scope and conflict rules are defined in `DOCUMENTATION_ARCHITECTURE.md`.

---

> **Status: Living · proving plan · first pass.** This is **not** a development roadmap. It is a plan for **validating architectural hypotheses**: the smallest set of experiments that must succeed — or fail informatively — before we trust that the conceptual architecture can become a real system without losing its properties.
>
> Each entry below is an experiment designed to **falsify** a specific hypothesis, not a feature to ship. A hypothesis that fails is a result, not a setback: it tells us what to revise before we build on it.
>
> This first pass specifies four experiments: a gating hypothesis about whether the Domain Model survives real use, and three that build on it. Their broader assumption catalogue and the decisions they may confirm, revise or supersede are named here only as experiment consequences. Current technology selections are imported from `TECHNICAL_ARCHITECTURE.md` and remain provisional until evidence stabilizes them.

## Purpose

ResearchOS is past discovering its domain. The conceptual architecture — seven entities, the knowledge graph, memory/context/events, the capability model — is mature on paper. What is unproven is the transition: **can that model become an executing system while keeping the properties that make it worth building?**

So this document does not ask *"what will we program?"* It asks:

> **What must we demonstrate to trust this architecture?**

Every experiment exists to answer that for one architectural claim. We implement the minimum needed to put the claim at risk, then measure whether it survives.

### The claim under test, above all others

The research behind this plan surveyed how comparable systems are organized, and one pattern was unmistakable: **each orbits a mechanism.**

```
AIOS            orbits  an agent operating system
IRE             orbits  Markdown + Git
LangGraph       orbits  execution graphs
Semantic Kernel orbits  plugins
LlamaIndex      orbits  indices
Haystack        orbits  pipelines
```

ResearchOS must orbit **none** of these. Its center is the **Domain Model**; memory, events, retrieval, agents, Git and indices are all *mechanisms in service of it* — each replaceable without touching the domain. That is the single most important thing the research validated, and it is the property these experiments are designed to protect. If any experiment can only be made to work by letting a mechanism become the center, the experiment has failed even if the feature works.

## Two kinds of architectural claim

Not everything this document scrutinizes is falsifiable in weeks, and pretending otherwise would be dishonest. There are two kinds of claim, evaluated differently — conflating them is how projects fool themselves into "validating" a decision that was never on the table.

**Imported architectural commitments.** The following decisions are owned by the canonical architecture documents, not by this plan. This document treats them as constraints whose consequences can be tested, but does not redefine them:

- **Domain-first** — the Domain Model is the center; everything else is a mechanism serving it.
- **Single Source of Truth** — adopt the state-authority model defined in `SYSTEM_ARCHITECTURE.md`; no derived mechanism becomes an authoritative shadow of domain facts.
- **Event-driven** — components coordinate through domain events, not by calling one another.
- **AI as an operational layer** — the intelligence operates over the domain; it never owns a private copy of it, nor controls the high-level flow.
- **Capabilities before tools** — the system speaks in abstract verbs; concrete tools are implementation detail.

These are not created or settled here; they are *imported commitments*. What an experiment can reveal is not whether a bet is "true," but whether **its consequences justify their cost**. Experiment 2 is exactly this: it does not prove Domain-first is correct — it measures whether the cost of the graph, memory and context (the consequences of Domain-first) buys more than a cheaper alternative. If the consequences never justify the cost, the bet was wrong even though no experiment "failed."

**Falsifiable hypotheses.** Everything in the numbered experiments below is a claim that can be put at risk and killed by a concrete observation within weeks. This is where the discipline of the document lives.

The distinction governs how we react to a result. A failed hypothesis means *revise the design*. A bet whose consequences never justify their cost means *reconsider the bet* — a slower, heavier decision, recorded as an ADR, not a quick pivot. And note the relationship between the two: **Domain-first** is the bet that the domain should be the center; **Experiment 0** is the falsifiable test of whether the *specific* seven-entity model chosen under that bet actually survives real use. The first is committed; the second we actively try to break.

Each experiment below also illuminates the cost/benefit of one or more assumptions:

- Experiment 0 → **Domain-first** (does the chosen model even hold up under use?)
- Experiment 2 → **Domain-first + Single Source of Truth** (do the graph, memory and context earn their cost against plain RAG?)
- Experiment 3 → **Event-driven** (does coordination-through-events hold consistency in practice?)

## The mechanism map

Everything below is a servant of the domain, and each is swappable. The domain is truth; each mechanism is a projection. `TECHNICAL_ARCHITECTURE.md` now supplies the current reversible product choices; this map continues to fix only roles and therefore remains valid when an adapter is replaced.

| Concern | Mechanism (category) | What it serves |
|---|---|---|
| **Documents / Artifacts** | Versioned files (Git) | Human-editable content, history, provenance |
| **Knowledge Graph** | Domain store | Structured Knowledge and its typed relations |
| **Memory** | Memory infrastructure | Semantic · episodic · operating recall, for reasoning and continuity |
| **Events** | Event store | The system's own evolution, made queryable |
| **Search / Retrieval** | Specialized indices | Efficient recall under a relevance budget |

The reconciliation this map encodes, stated plainly:

> **Git is a versioning mechanism for human content. It is never the operating system's memory.** Git preserves what a human would call a *document*; it never holds the system's internal state — the graph, the memory, the events. Those live in stores built for them.

## How each experiment is specified

Every experiment carries the same six fields, so each stays a test of the architecture rather than a task list:

- **Architectural hypothesis** — the claim being put at risk.
- **Minimal implementation** — the least that must be built to test it, expressed in existing Capabilities and Domain entities.
- **Acceptance criteria** — what would let us trust the hypothesis.
- **Metrics** — what is measured to decide.
- **What would falsify it** — the concrete observation that kills the hypothesis.
- **Dependent decisions** — what we commit to, revise, or unlock depending on the outcome.

**Experiment 0 comes first and gates the rest.** Its method is the exception: it is falsified by structured real use, not by a benchmark or a baseline. If the domain does not survive use, the other three are premature — they would only measure how well we optimize an incorrect foundation.

---

## Experiment 0 — The Domain Model survives real use

**Architectural hypothesis.** The seven Core Entities and their relations are sufficient to represent the researcher's everyday operational reality **without significant friction** — without requiring new root entities, without breaking invariants, without inventing artificial relations to make things fit. This is the highest-risk claim in the project: if the domain does not survive real use, then memory, context and events — however well built — optimize an incorrect foundation.

**Minimal implementation.** No benchmark, and ideally no new code. Over several days, attempt real, everyday operations expressed *only* in the model's entities, relations and lifecycles, deliberately spanning verticals so the breadth claim is tested and not just the research path:

- import a paper (Research) · register a meeting (Daily Work) · create a task · associate Knowledge · prepare a lecture (Teaching) · record a decision (Administration) · relate people (People).

Keep a structured **friction log**: every time the reaction is *"I need a new entity"* or *"I don't know where this fits,"* record the operation, what was missing, and the workaround used.

**Acceptance criteria.** The everyday operations can be represented within the existing model, and the friction log shows exceptions are rare and superficial: no new root entity was required, no invariant had to be broken, and no artificial relation had to be invented. This is the Phase A claim — *seven entities, no new roots, covering all six verticals* — surviving contact with real use rather than holding only on paper.

**Metrics.** Frequency of "new entity needed" and "doesn't fit" events per operation; number of invariants strained or broken; number of forced or artificial relations introduced; share of operations completed with no modeling exception. Qualitative counts from real use, not benchmark scores.

**What would falsify it.** The friction recurs often: everyday operations repeatedly demand new entities, break invariants, or force artificial relations. Frequent friction is not an implementation problem — it is the signal that the domain itself is wrong.

**Dependent decisions.** This experiment gates the other three. If it holds, the foundation is trustworthy and Experiments 1–3 proceed. If it fails, the correct response is to revise the Domain Model — a deliberate change to the frozen conceptual layer, recorded as a decision — *before* investing in memory, context, events or AI on top of it. Nothing downstream is worth optimizing on a model that does not survive use.

---

## Experiment 1 — Capture becomes Knowledge without a human organizing

**Architectural hypothesis.** The pipeline **Acquire → Process → Understand → Organize** can take a raw Paper and produce linked, provenance-bearing **Candidate** Knowledge in the graph, end to end — with the researcher's only obligation being *capture*. This is the "researcher captures, the system organizes" premise (AI Architecture) put at risk against real documents.

**Minimal implementation.**
- Ingest a real paper: preserve content under a `locator`, derive text and core metadata; the `Document` (Paper) walks `Registered → Processed → Available`.
- React to the *Available* event (no direct call): identify candidate concepts, create `Knowledge` nodes with `Provenance` to the source, and link them into the graph with typed `KnowledgeRelation` edges.
- All derived Knowledge rests at `Draft → Candidate`. Never `Validated`. Promotion is the researcher's act (Improvement Loop) — the human gate is present from the first line, not retrofitted.
- Composes the Capture and Curate/Link responsibilities. Maps to UC-K03, UC-K04.

**Acceptance criteria.**
- A real paper yields a persisted Document (content reachable via `locator`, immutable `Provenance`) and a set of Candidate Knowledge nodes linked into the graph.
- Relations are typed and traversable both ways (bidirectional navigability).
- No derived Knowledge is ever written as Validated; the researcher, not the pipeline, holds the gate.
- Extraction runs purely as a reaction to the ingestion event — the two stages share no direct coupling.

**Metrics.** Metadata correctness (≥ 90% titles on a test set); extraction quality vs. a manual ground truth (F1 ≥ 80%); count of nodes/edges integrated; share of the domain's entity/relation types actually populated; per-document processing time and failure rate.

**What would falsify it.** Organizing still demands substantial human structuring; extraction quality is so low the graph is noise rather than knowledge; or real papers routinely produce concepts and relations the domain model cannot hold without inventing new structure.

**Dependent decisions.** Whether the "capture, the system organizes" premise survives contact with real data; whether the Logical Domain Model needs additions (new Derived Types) it did not anticipate; whether extraction is trustworthy enough to feed later experiments.

---

## Experiment 2 — Domain-grounded context beats conventional RAG

**Architectural hypothesis.** Answers assembled through the **Context Builder** — inferring intent, anchoring in the domain, **expanding over the knowledge graph**, recalling from memory, then prioritizing and bounding — are **more useful, more grounded and more traceable** than a conventional flat-vector RAG baseline over the same corpus. This is the load-bearing hypothesis of the whole architecture: it is what justifies building a graph and a Context pipeline instead of plain retrieval.

**Minimal implementation.**
- The domain-grounded path: a question drives Context construction (Intent → Anchors → Expand → Recall → Recent Change → Constrain → Prioritize & Bound); reasoning answers over that Context only; the result is an **Artifact** carrying provenance (what produced it, from what Context, when). Maps to UC-K05.
- A **baseline** to beat: conventional RAG over the same documents — flat vector similarity, no graph, no intent modeling, no bounding discipline.
- Both answer the same evaluation question set. The comparison *is* the experiment.

**Acceptance criteria.**
- On the question set, the domain-grounded answers are measurably better than the RAG baseline on groundedness and traceability, and at least as good on correctness.
- Every domain-grounded answer cites specific Knowledge/Documents; nothing is asserted that its Context does not support (no fabrication — the intent of UC-K05).
- The advantage is attributable to the graph/context machinery, not to incidental prompt differences.

**Metrics.** Answer correctness (domain-grounded vs. baseline on the same set); citation/provenance fidelity; groundedness (share of claims traceable to a source); end-to-end latency (P50/P90); number of sources used.

**What would falsify it.** The domain-grounded answers are no better — or worse — than vanilla RAG on the test set. If so, the graph + Context pipeline is not yet justified, and the honest move is to start with plain RAG and *earn* the complexity later. This is the experiment that keeps the architecture from over-building on faith.

**Dependent decisions.** Whether to invest in the knowledge graph and the Context Builder now, or defer them behind a simpler retrieval baseline; where the relevance/budget effort is worth spending; how much of the Context Model's pipeline earns its place in the first real build.

---

## Experiment 3 — The domain evolves through events without losing consistency

**Architectural hypothesis.** Domain change propagates through **events** (capture → derive → link → notify) as **bounded** reactive cascades, preserving consistency — immediate within an aggregate, eventual across aggregates — and full provenance, without event storms, loops, or corruption. This tests whether coordination-through-the-domain (Event Model; AI Architecture → Agent Coordination) is viable in practice, not just on paper.

**Minimal implementation.**
- Instrument the event flow already produced by Experiment 1 (Document created/available, Knowledge derived/linked).
- Inject concurrent and duplicate changes (the same paper twice; overlapping extractions; a contradicting piece of evidence).
- Observe cascade termination, cross-aggregate consistency, and whether every lifecycle transition retains its cause (*caused transitions*, *no silent overwrite*).

**Acceptance criteria.**
- Reactive cascades terminate; a reaction that produces no meaningful change ends the chain, and loops are broken rather than followed.
- After concurrent changes settle, cross-aggregate state is consistent and free of duplicate or orphaned nodes.
- Every change is reconstructible from provenance; no prior state is silently overwritten.
- Reactions obey autonomy limits — irreversible or outward-facing actions never fire without approval, even when event-triggered.

**Metrics.** Cascade depth and termination rate; incidence of loops/storms under injected load; cross-aggregate consistency after concurrent writes; provenance-reconstructibility rate; count of duplicate/orphaned entities produced.

**What would falsify it.** Injected concurrency produces event storms, non-terminating loops, inconsistent cross-aggregate state, or lost/incorrect provenance. Any of these means event-driven coordination needs a stronger bounding or consistency mechanism before it can be relied on.

**Dependent decisions.** Whether event-driven coordination (vs. explicit orchestration) is viable at all; the concrete bounding strategy for cascades; whether **Event** should be promoted to a first-class persisted record so the system's reactive history becomes queryable (a candidate promotion already flagged in the Event Model).

---

## What "trusting the architecture" means

Experiment 0 is the precondition: if the Domain Model does not survive real use, no result from the others is worth acting on. When Experiment 0 holds and the three that follow have been demonstrated — or falsified informatively — we can either stabilize the current provisional baseline or revise the model and adapters before expanding the product. The point is to make expensive stabilization and scaling decisions on evidence rather than on the elegance of the design.

Concretely, this provides the falsification programme for the ROADMAP's **M12 · Proving slice**, reframed from "a feature running end-to-end" to "the architecture's core claims, tested." One full turn of the operating cycle appears across the three experiments: capture changes the domain (E1), a question assembles context and produces a grounded answer (E2), and change propagates through events without breaking consistency (E3).

## Deferred to a later pass

Named so the boundary of this first pass is explicit; none is decided here.

- **The full assumption catalogue** — beyond the three tested here, the remaining architectural assumptions and their falsification methods.
- **Adopt / avoid decisions** — distilled into recorded ADRs (`DECISIONS.md`, reserved by the ROADMAP).
- **Technology replacement evidence** — measurements that would justify replacing the current embedded stores, model adapters, ingestion tools or process boundaries. Initial choices belong to `TECHNICAL_ARCHITECTURE.md` and ADR-0007 through ADR-0010; experiments may trigger later ADRs but do not silently rewrite them.

## Relationship to other documents

- **DOCUMENTATION_ARCHITECTURE** — owns this document's bounded authority and the rule that experiments cannot redefine architecture.
- **IMPLEMENTATION_CONTEXTS** — defines the context bundles used to implement each proving experiment.
- **ROADMAP** — owns the phase sequence and the Proving Milestone this document turns into experiments. It remains the plan of record.
- **Use Cases** — the experiments are minimal realizations of UC-K03, UC-K04, UC-K05; they invent no new use case.
- **System Capabilities** — the verbs each experiment composes.
- **Logical Domain Model** — the entities, lifecycles and invariants each experiment must preserve, including the `locator` for content and the Knowledge human gate.
- **Domain Verticals** — Experiment 0 stress-tests their founding claim, *seven Core Entities cover all six operational verticals*, against real everyday use rather than on paper.
- **Context Model** — the construction pipeline Experiment 2 puts to the test against a baseline.
- **Event Model** — the events and the bounded-cascade discipline Experiment 3 stresses.
- **AI Architecture** — the Artifact, the Improvement Loop, and the autonomy/human-control boundaries every experiment obeys.

This document owns one thing: the smallest set of experiments that decide whether the domain-centered architecture can become a real system without losing what makes it worth building.
