# Use Cases

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 3 — Functional and Vertical Specification |
| **Normative status** | Canonical behavioral specification |
| **Authoritative for** | Recurring researcher intentions, operational flows, expected outcomes, state changes, capabilities and entity participation. |
| **Not authoritative for** | UI sequences, component choreography, data schemas, runtime topology or technology choices. |
| **Required reading** | `VISION.md`, `SYSTEM_RESPONSIBILITIES.md`, `DOMAIN_MODEL.md`, `KNOWLEDGE_MODEL.md`, `SYSTEM_CAPABILITIES.md`. |
| **Downstream documents** | `DOMAIN_VERTICALS.md`, all `*_VERTICAL.md` files, `LOGICAL_DOMAIN_MODEL.md`, `INTERACTION_MODEL.md`, `IMPLEMENTATION_CONTEXTS.md`, acceptance tests. |

> Scope and conflict rules are defined in `DOCUMENTATION_ARCHITECTURE.md`.

---

## Purpose

This document defines the canonical use cases of ResearchOS.

A use case represents a recurring research situation in which the Research Operating System assists the researcher to achieve an operational goal.

Use cases are not features.

They are not user interface interactions.

They are not implementation workflows.

Instead, each use case describes an intention expressed by the researcher, the operational context in which it occurs, and the composition of capabilities required to produce a meaningful outcome.

Together they define the behavioral specification of the platform.

---

## Relationship with the Architecture

The architectural documentation progresses from abstract concepts toward concrete researcher activities.

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
```

The previous documents answer:

- Why the system exists.
- What exists.
- What the system can do.

This document answers:

> How those capabilities are composed to solve real research problems.

---

## Design Principles

Every use case follows the same principles.

### Research-driven

Every use case originates from real research practice.

The catalogue is intentionally derived from the operational behaviour of the first working instance of ResearchOS rather than hypothetical functionality.

### Technology-independent

A use case describes intent.

It never depends on a specific implementation.

Whether a capability is implemented through an LLM, a database, an external service or a human decision is outside the scope of this document.

### Capability Composition

A use case never introduces new behaviour.

It composes existing capabilities.

```
Acquire
   ↓
Process
   ↓
Understand
   ↓
Organize
   ↓
Retrieve
   ↓
Reason
   ↓
Produce
   ↓
Operate
```

Almost every use case realises the same underlying shape — acquire or retrieve information, understand it, reason over it, produce a result and integrate it into the operational state. This recurrence is intentional: it is the signature of a coherent architecture.

### Domain Consistency

Every use case manipulates existing domain entities.

If multiple use cases repeatedly require a concept not represented by the Domain Model, that concept becomes a candidate for promotion into the model as a Derived Type.

In this way, the Domain Model evolves from observed behaviour rather than speculation.

---

## Use Case Structure

Every use case follows the same structure.

- **Intention** — the operational goal expressed by the researcher.
- **Context** — the situation and preconditions in which it occurs.
- **Operational Flow** — the composition of capabilities that produces the outcome.
- **Expected Outcome** — the meaningful change in operational state, described narratively.
- **State Changes** — the precise mutation of the operational state, grouped as *Created*, *Updated*, *Derived* and *Linked*, with lifecycle transitions made explicit. This section is the direct bridge from behaviour to the Domain Model and to future system events. Read-only use cases declare no persistent change.
- **Capabilities** — the capabilities exercised.
- **Domain Entities** — the entities read or written.
- **Related Use Cases** — neighbouring situations in the catalogue.

Entities written as `Entity (Specialization)` denote a specialization not yet formalized in the Domain Model.

Within State Changes:

- **Created** — new entities brought into existence.
- **Updated** — existing entities whose attributes or lifecycle state change.
- **Derived** — knowledge produced by interpretation or reasoning over other entities.
- **Linked** — new relationships established between entities.

---

# Canonical Use Cases

The catalogue is organised around the operational domains of the Research Operating System.

---

# Knowledge

Knowledge acquisition, creation, refinement and retrieval.

Part of the Research vertical — see [RESEARCH_VERTICAL.md](RESEARCH_VERTICAL.md).

### UC-K01 · Capture a Hypothesis

**Intention.** Record a newly generated hypothesis the moment it appears, before the surrounding reasoning is lost, and have it situated within existing knowledge.

**Context.** Hypotheses emerge unpredictably, frequently in the middle of another activity, so capture must be near-frictionless. Yet a raw statement stored in isolation is rarely recoverable or operable later; the value lies in connecting it to what the researcher already knows and in making explicit what is still missing.

**Operational Flow.**
1. Acquire the hypothesis statement with minimal friction. → *Acquire*
2. Retrieve related hypotheses, concepts and knowledge already present. → *Retrieve*
3. Relate the hypothesis to that surrounding knowledge and detect missing context. → *Reason (Discover Connections)*
4. When internal context is insufficient, prepare external literature search strategies to strengthen it. → *Produce*
5. Persist the hypothesis as a single authoritative unit of knowledge with its relationships. → *Organize*

**Expected Outcome.** A hypothesis exists as connected knowledge in an initial state, positioned in the knowledge graph, with gaps made explicit and, where needed, search strategies queued. No reasoning has been lost.

**State Changes.**
- *Created* — Knowledge (Hypothesis), initial state `captured`.
- *Updated* — none.
- *Derived* — Resource (search strategies), only when internal context is insufficient.
- *Linked* — Hypothesis ↔ related Knowledge and Concepts; Hypothesis → Knowledge Graph.

**Capabilities.** Acquire · Retrieve · Reason · Produce · Organize

**Domain Entities.** Knowledge (Hypothesis) · Document · Resource

**Related Use Cases.** UC-K02, UC-K03, UC-R04

---

### UC-K02 · Develop a Hypothesis

**Intention.** Turn an initial hypothesis into a mature, well-grounded research hypothesis ready for experimentation.

**Context.** A captured hypothesis is usually underspecified. Developing it requires structured thinking, connection to prior literature and concepts, and the surfacing of assumptions the researcher may not remember. This is a deliberate, iterative activity rather than a quick capture.

**Operational Flow.**
1. Retrieve the existing hypothesis and its current relationships. → *Retrieve*
2. Retrieve semantically related knowledge, concepts and processed literature. → *Retrieve*
3. Engage the researcher in structured dialogue, challenging assumptions and proposing connections. → *Reason (Critique, Discover Connections)*
4. Synthesize the refined formulation, its rationale and its open questions. → *Understand → Reason (Synthesize)*
5. Persist the matured hypothesis, updating its state and relationships. → *Organize*

**Expected Outcome.** The hypothesis advances from an initial intuition to a mature formulation with explicit assumptions, supporting knowledge and readiness for experimental design.

**State Changes.**
- *Created* — Knowledge (Concept), when new concepts crystallize during dialogue.
- *Updated* — Knowledge (Hypothesis): formulation and assumptions; lifecycle state `captured → developing`.
- *Derived* — Knowledge (rationale, open questions).
- *Linked* — Hypothesis ↔ Concepts and processed literature (new backlinks).

**Capabilities.** Retrieve · Understand · Reason · Organize

**Domain Entities.** Knowledge (Hypothesis) · Knowledge (Concept) · Document

**Related Use Cases.** UC-K01, UC-K03, UC-R01

---

### UC-K03 · Acquire Scientific Literature

**Intention.** Bring scientific literature into the system so it becomes operable knowledge rather than an inert file.

**Context.** Literature arrives continuously and in volume. A stored document is not knowledge; its value is realised only when its meaning is extracted, its provenance preserved, and it is connected to the projects, hypotheses and concepts it relates to.

**Operational Flow.**
1. Acquire the document and register its metadata and provenance. → *Acquire*
2. Process the raw document into machine-operable form: extract, normalize, index. → *Process*
3. Understand its content: classify, summarise and identify the concepts it introduces. → *Understand*
4. Organize it into the knowledge graph, relating it to existing concepts, projects and hypotheses. → *Organize*

**Expected Outcome.** A document exists with preserved provenance, and the knowledge it carries is connected and retrievable. The literature is now operable across other use cases.

**State Changes.**
- *Created* — Document (Paper), initial state `Registered`.
- *Updated* — Bibliography.
- *Derived* — Knowledge (Concepts) extracted from the document.
- *Linked* — Paper ↔ Project, Hypothesis and Knowledge Graph.

**Capabilities.** Acquire · Process · Understand · Organize

**Domain Entities.** Document (Paper) · Knowledge (Concept) · Project · Bibliography

**Related Use Cases.** UC-K01, UC-K02, UC-R04, UC-W03

---

### UC-K04 · Explore a Research Topic

**Intention.** Build understanding of an unfamiliar topic progressively, prioritising exploration over answering a single question.

**Context.** Early in a line of inquiry the researcher does not yet know the right questions. The system must support open-ended, iterative discovery that accumulates context rather than returning a single answer.

**Operational Flow.**
1. Retrieve existing internal knowledge related to the topic. → *Retrieve*
2. Identify the boundaries of current understanding and the gaps. → *Reason*
3. Where internal knowledge is thin, prepare search strategies and incorporate acquired literature. → *Produce*
4. Synthesise an evolving map of the topic: key concepts, relationships and open questions. → *Reason (Synthesize) → Understand*
5. Persist the growing topic map as connected knowledge. → *Organize*

**Expected Outcome.** The researcher holds a structured, evolving understanding of the topic, with concepts and open questions connected in the graph, ready to seed hypotheses.

**State Changes.**
- *Created* — Knowledge (Topic Map), Knowledge (Concept).
- *Updated* — Knowledge (Topic Map) across successive iterations.
- *Derived* — Knowledge (concept relationships, open questions).
- *Linked* — Concepts ↔ Documents ↔ Project within the Knowledge Graph.

**Capabilities.** Retrieve · Reason · Understand · Produce · Organize

**Domain Entities.** Knowledge (Concept) · Document · Project

**Related Use Cases.** UC-K03, UC-K05, UC-R04

---

### UC-K05 · Support a Research Question

**Intention.** Answer a specific, complex research question using the knowledge already in the system, without fabricated conclusions.

**Context.** Unlike exploration, this begins with a concrete question. The system must reason over existing knowledge and, critically, distinguish between what is supported by evidence and what is not.

**Operational Flow.**
1. Interpret the question and identify the knowledge it requires. → *Understand*
2. Retrieve relevant knowledge, evidence and literature. → *Retrieve*
3. Reason over the retrieved material to construct a grounded answer. → *Reason (Synthesize, Support Decision)*
4. When evidence is insufficient, identify what is missing and propose search strategies instead of inventing an answer. → *Reason → Produce*
5. Present the answer with explicit provenance and confidence. → *Produce*

**Expected Outcome.** The researcher receives a grounded answer traceable to its supporting knowledge, or an explicit statement of what evidence is missing and how to obtain it. The system never fabricates conclusions.

**State Changes.**
- *Created* — none.
- *Updated* — none.
- *Derived* — Knowledge (grounded answer, or explicit evidence gap).
- *Linked* — Answer ↔ supporting Knowledge, Evidence and Documents.
- *Read-only: the answer mutates no persistent state unless the researcher chooses to keep it.*

**Capabilities.** Understand · Retrieve · Reason · Produce

**Domain Entities.** Knowledge · Knowledge (Evidence) · Document

**Related Use Cases.** UC-K04, UC-R04, UC-R02

---

# Research

Activities directly related to scientific investigation.

Part of the Research vertical — see [RESEARCH_VERTICAL.md](RESEARCH_VERTICAL.md).

### UC-R01 · Design an Experiment

**Intention.** Turn one or more mature hypotheses into an executable experimental design.

**Context.** A hypothesis becomes testable only when expressed as an experiment with defined objectives, resources, expected observations and measurable outcomes. This is the bridge between knowledge and evidence.

**Operational Flow.**
1. Retrieve the mature hypothesis and its supporting knowledge. → *Retrieve*
2. Reason from hypothesis to a testable design: objectives, variables, expected observations, success criteria. → *Reason*
3. Identify the resources the experiment requires. → *Reason → Organize*
4. Produce the experimental design as a structured, canonical artifact. → *Produce*
5. Register the experiment and move the hypothesis into an experimenting state. → *Operate → Organize*

**Expected Outcome.** An experiment exists with objectives, required resources, expected observations and measurable outcomes, linked to the hypothesis it tests, which now carries an experimenting state.

**State Changes.**
- *Created* — Activity (Experiment), initial state `planned`; Document (experimental design).
- *Updated* — Knowledge (Hypothesis): lifecycle state `developing → experimenting`.
- *Derived* — Resource requirements identified for the experiment.
- *Linked* — Experiment ↔ Hypothesis, Resource and Project.

**Capabilities.** Retrieve · Reason · Produce · Organize · Operate

**Domain Entities.** Knowledge (Hypothesis) · Activity (Experiment) · Resource · Project

**Related Use Cases.** UC-K02, UC-R02, UC-R03, UC-OR02

---

### UC-R02 · Analyse Experimental Results

**Intention.** Interpret experimental evidence and prepare it for scientific conclusions.

**Context.** Raw experimental output is not evidence until interpreted. Analysis compares observations against expectations and existing knowledge, and produces evidence in an operable form.

**Operational Flow.**
1. Retrieve the experiment, its design and its raw observations. → *Retrieve*
2. Process and normalise the observations into comparable form. → *Process*
3. Interpret the results and compare them with expectations and prior knowledge. → *Understand → Reason (Compare)*
4. Produce structured evidence and a readable analysis. → *Produce*
5. Relate the evidence back to the hypothesis and the knowledge graph. → *Organize*

**Expected Outcome.** Interpreted evidence exists, connected to its experiment and hypothesis, ready to inform validation and scientific writing.

**State Changes.**
- *Created* — Knowledge (Evidence).
- *Updated* — Activity (Experiment): lifecycle state `running → analysed`.
- *Derived* — Knowledge (Evidence, readable analysis) from raw observations.
- *Linked* — Evidence ↔ Experiment ↔ Hypothesis; Evidence → Knowledge Graph.

**Capabilities.** Retrieve · Process · Understand · Reason · Produce · Organize

**Domain Entities.** Activity (Experiment) · Knowledge (Evidence) · Knowledge (Hypothesis)

**Related Use Cases.** UC-R01, UC-R03, UC-K05

---

### UC-R03 · Validate a Hypothesis

**Intention.** Determine whether accumulated evidence supports, falsifies or leaves a hypothesis unresolved.

**Context.** Over time, evidence accrues around a hypothesis. Validation is the decision point that updates the hypothesis's operational state while preserving traceability to the evidence behind that decision.

**Operational Flow.**
1. Retrieve the hypothesis and all evidence related to it. → *Retrieve*
2. Reason over the body of evidence to assess support, falsification or insufficiency. → *Reason (Support Decision)*
3. Determine the resulting state: evidenced, falsified or unresolved. → *Reason*
4. Update the hypothesis's operational state, preserving links to the supporting evidence. → *Organize → Operate*

**Expected Outcome.** The hypothesis carries a resolved state — evidenced, falsified or unresolved — fully traceable to the evidence that justified it. Nothing is overwritten silently.

**State Changes.**
- *Created* — none.
- *Updated* — Knowledge (Hypothesis): lifecycle state `experimenting → evidenced | falsified | unresolved`.
- *Derived* — Knowledge (validation rationale linking decision to evidence).
- *Linked* — Hypothesis ↔ the specific Evidence that justified the decision.

**Capabilities.** Retrieve · Reason · Organize · Operate

**Domain Entities.** Knowledge (Hypothesis) · Knowledge (Evidence) · Activity (Experiment)

**Related Use Cases.** UC-R02, UC-K02, UC-W01

---

### UC-R04 · Generate Literature Search Strategy

**Intention.** Produce high-quality, structured search strategies for specialised discovery platforms, rather than having the system pretend to be one.

**Context.** ResearchOS does not replace specialised literature search engines. Its role is to translate a knowledge gap into effective search strategies whose results are later brought back into the system.

**Operational Flow.**
1. Retrieve the hypothesis, question or topic and its current knowledge context. → *Retrieve*
2. Reason about what evidence is missing and where it is likely to be found. → *Reason*
3. Produce structured search prompts and retrieval strategies for external platforms. → *Produce*
4. Register the strategies so their future results can be acquired and connected. → *Organize*

**Expected Outcome.** A set of high-quality search strategies exists, tied to the gap that motivated them, ready to drive external discovery and subsequent acquisition.

**State Changes.**
- *Created* — Resource (search strategies).
- *Updated* — none.
- *Derived* — Knowledge (explicit statement of the evidence gap).
- *Linked* — Search strategies ↔ Hypothesis or Knowledge gap that motivated them.

**Capabilities.** Retrieve · Reason · Produce · Organize

**Domain Entities.** Knowledge · Knowledge (Hypothesis) · Resource

**Related Use Cases.** UC-K01, UC-K03, UC-K04, UC-K05

---

# Writing

Support scientific communication.

Part of the Research vertical — see [RESEARCH_VERTICAL.md](RESEARCH_VERTICAL.md).

### UC-W01 · Draft Scientific Content

**Intention.** Generate structured scientific documents from the knowledge already present in the system.

**Context.** Writing should not start from a blank page nor invent content. It should compose existing knowledge, evidence and literature into a structured draft the researcher then refines.

**Operational Flow.**
1. Retrieve the relevant knowledge, evidence and literature for the intended piece. → *Retrieve*
2. Reason about structure and argument, organising the material into a narrative. → *Reason (Synthesize)*
3. Produce a structured draft grounded in that knowledge, with citations. → *Produce (Write, Cite)*
4. Register the draft as a document linked to its sources. → *Organize*

**Expected Outcome.** A structured scientific draft exists, grounded in and traceable to the system's knowledge, ready for review.

**State Changes.**
- *Created* — Document (Chapter / Draft).
- *Updated* — none.
- *Derived* — Document (structured draft) composed from existing knowledge.
- *Linked* — Draft ↔ Knowledge, Evidence and Bibliography.

**Capabilities.** Retrieve · Reason · Produce · Organize

**Domain Entities.** Document (Chapter) · Knowledge · Knowledge (Evidence) · Bibliography

**Related Use Cases.** UC-W02, UC-W03, UC-R02

---

### UC-W02 · Review Scientific Writing

**Intention.** Evaluate the clarity, consistency, evidence coverage and scientific quality of an existing document.

**Context.** Drafts — the researcher's own or others' — need critical review that goes beyond style: whether claims are supported, references real, and arguments coherent.

**Operational Flow.**
1. Retrieve the document and the knowledge and evidence it relies on. → *Retrieve*
2. Understand the document's claims and structure. → *Understand*
3. Critique clarity, consistency, evidence coverage and scientific quality; verify references. → *Reason (Critique)*
4. Produce a structured review with actionable findings. → *Produce*

**Expected Outcome.** A review exists identifying strengths, gaps in evidence, inconsistencies and unsupported or unverifiable claims, enabling targeted revision.

**State Changes.**
- *Created* — Document (Review).
- *Updated* — none.
- *Derived* — Knowledge (findings: evidence gaps, inconsistencies, unverifiable claims).
- *Linked* — Review ↔ Document, Evidence and Bibliography.

**Capabilities.** Retrieve · Understand · Reason · Produce

**Domain Entities.** Document · Knowledge (Evidence) · Bibliography

**Related Use Cases.** UC-W01, UC-W03, UC-K05

---

### UC-W03 · Generate References

**Intention.** Produce citations and bibliographic material consistent with the content being written.

**Context.** Citations must correspond to real sources already known to the system and match the claims they support; fabricated or mismatched references undermine scientific integrity.

**Operational Flow.**
1. Retrieve the sources underlying the content from the bibliography and knowledge graph. → *Retrieve*
2. Reason about which claims require citation and to which source. → *Reason*
3. Produce citations and bibliographic entries in the required form. → *Produce (Cite)*
4. Relate the references to the document. → *Organize*

**Expected Outcome.** Accurate, verifiable references exist, bound to the claims they support and to real sources in the system.

**State Changes.**
- *Created* — Bibliography (citations / entries).
- *Updated* — Document (draft): citations inserted.
- *Derived* — none.
- *Linked* — References ↔ Document claims ↔ source Documents.

**Capabilities.** Retrieve · Reason · Produce · Organize

**Domain Entities.** Bibliography · Document · Knowledge

**Related Use Cases.** UC-W01, UC-W02, UC-K03

---

# Planning

Coordinate research execution.

### UC-P01 · Manage Research Tasks

**Intention.** Create, prioritise, update and review research tasks while keeping them consistent with projects and objectives.

**Context.** Research work generates many tasks across domains. Their value depends on remaining connected to the projects and objectives they serve, so priorities reflect research goals rather than noise.

**Operational Flow.**
1. Acquire or update the task from the researcher or from other use cases. → *Acquire*
2. Relate the task to its project, objectives and related knowledge. → *Organize*
3. Prioritise it against existing commitments and objectives. → *Reason (Prioritize)*
4. Track and review its state over time. → *Operate (Track, Review)*

**Expected Outcome.** Tasks are maintained as first-class entities, prioritised in line with research objectives and consistently linked to their projects.

**State Changes.**
- *Created* — Task, when new.
- *Updated* — Task: priority and lifecycle state.
- *Derived* — none.
- *Linked* — Task ↔ Project, objectives and related Knowledge.

**Capabilities.** Acquire · Organize · Reason · Operate

**Domain Entities.** Task · Project · Knowledge

**Related Use Cases.** UC-P02, UC-P03, UC-C01

---

### UC-P02 · Plan Research Activities

**Intention.** Prepare short- and medium-term research plans aligned with project milestones.

**Context.** Research is non-linear, but execution still needs coordination. Planning aligns upcoming activities with milestones and available capacity without imposing a rigid workflow.

**Operational Flow.**
1. Retrieve current projects, milestones, tasks and progress. → *Retrieve*
2. Reason about priorities, dependencies and capacity. → *Reason (Prioritize)*
3. Produce a short/medium-term plan aligned with milestones. → *Produce*
4. Register the plan and its tasks. → *Operate (Plan, Schedule)*

**Expected Outcome.** A coherent plan exists linking upcoming activities to project milestones, ready to guide execution and review.

**State Changes.**
- *Created* — Document (Plan); Task(s).
- *Updated* — Task(s): scheduling; Project.
- *Derived* — Knowledge (prioritisation and dependencies).
- *Linked* — Plan ↔ Project milestones ↔ Tasks.

**Capabilities.** Retrieve · Reason · Produce · Operate

**Domain Entities.** Project · Task · Activity

**Related Use Cases.** UC-P01, UC-P03, UC-X03

---

### UC-P03 · Review Research Progress

**Intention.** Summarise completed work, pending activities, blockers and research evolution over a defined period.

**Context.** Progress is easy to lose track of across long, interrupted research. A periodic, cross-cutting review restores the big picture and supports reporting.

**Operational Flow.**
1. Retrieve activity, tasks, knowledge growth and project state for the period. → *Retrieve*
2. Reason about what advanced, what stalled and why. → *Reason*
3. Produce a readable progress review. → *Produce*
4. Register it as part of the operational record. → *Operate (Review)*

**Expected Outcome.** A progress review exists covering completed work, pending activities, blockers and research evolution, usable for self-direction and reporting.

**State Changes.**
- *Created* — Document (Progress Review / Research Journal entry).
- *Updated* — none.
- *Derived* — Knowledge (progress synthesis: advances, blockers, evolution).
- *Linked* — Review ↔ Project, Task and Activity.

**Capabilities.** Retrieve · Reason · Produce · Operate

**Domain Entities.** Project · Task · Activity · Knowledge

**Related Use Cases.** UC-P02, UC-X03, UC-X04

---

# Communication

Capture operational information generated through interactions.

### UC-C01 · Process Incoming Email

**Intention.** Analyse incoming email and extract its operational content, rather than triaging it by hand.

**Context.** Email is a continuous source of new work, knowledge and noise. Left unmanaged it fragments attention; its value lies in the tasks, people, deadlines and knowledge buried inside it.

**Operational Flow.**
1. Acquire new messages since the last processing. → *Acquire*
2. Process and classify them by operational relevance. → *Process → Understand (Classify)*
3. Extract operational elements: tasks, deadlines, people, projects, documents, research ideas. → *Understand → Reason*
4. Produce proposed outputs, including draft responses for approval. → *Produce*
5. Relate the extracted elements to the system state. → *Organize*

**Expected Outcome.** Incoming email is reduced to structured, connected operational elements and proposed actions, with nothing important left buried.

**State Changes.**
- *Created* — Task, Person, Document (draft response), as extracted.
- *Updated* — Person or Project, when the message references an existing one.
- *Derived* — Knowledge (extracted research ideas); message classification.
- *Linked* — Extracted elements ↔ Project, Person and Task.

**Capabilities.** Acquire · Process · Understand · Reason · Produce · Organize

**Domain Entities.** Knowledge · Task · Person · Project · Document

**Related Use Cases.** UC-C02, UC-P01, UC-X01

---

### UC-C02 · Prepare Communication

**Intention.** Generate context-aware drafts for emails or other written communications, subject to researcher approval.

**Context.** Much communication is routine but context-dependent. The system can assemble the relevant context and draft a response, but the researcher retains authorship and final judgement.

**Operational Flow.**
1. Retrieve the relevant context: prior threads, people, projects, documents. → *Retrieve*
2. Understand the intent and required content of the communication. → *Understand*
3. Produce a context-aware draft for researcher approval. → *Produce (Communicate)*

**Expected Outcome.** A grounded draft exists, ready for the researcher to review, edit and send. The system never sends autonomously without approval.

**State Changes.**
- *Created* — Document (Draft communication), pending approval.
- *Updated* — none.
- *Derived* — Document (context-aware draft) assembled from existing context.
- *Linked* — Draft ↔ Person, Project and prior Documents.

**Capabilities.** Retrieve · Understand · Produce

**Domain Entities.** Person · Document · Project · Knowledge

**Related Use Cases.** UC-C01, UC-C03, UC-PE02

---

### UC-C03 · Prepare a Meeting

**Intention.** Assemble every piece of relevant context before an interaction with another person.

**Context.** Interactions are more productive when prior discussions, documents, tasks, decisions and related knowledge are at hand. Reassembling this from memory is costly and error-prone.

**Operational Flow.**
1. Retrieve the person and the interaction's subject. → *Retrieve*
2. Retrieve related prior discussions, documents, tasks, decisions, projects and knowledge. → *Retrieve*
3. Reason about what matters for this interaction and what remains open. → *Reason*
4. Produce a structured briefing separating settled context from open questions. → *Produce*

**Expected Outcome.** A meeting briefing exists gathering all relevant context and distinguishing the confirmed from the open, ready to make the interaction productive.

**State Changes.**
- *Created* — Document (Meeting briefing).
- *Updated* — none.
- *Derived* — Knowledge (briefing: confirmed context vs open questions).
- *Linked* — Briefing ↔ Person, Project, Document and Task.

**Capabilities.** Retrieve · Reason · Produce

**Domain Entities.** Person · Activity (Meeting) · Document · Task · Project · Knowledge

**Related Use Cases.** UC-PE02, UC-C04, UC-P01

---

### UC-C04 · Record an Interaction

**Intention.** Transform conversations, meetings or calls into structured knowledge.

**Context.** Interactions generate decisions, actions and knowledge that are lost if not captured. Recording converts an ephemeral event into durable operational state.

**Operational Flow.**
1. Acquire the interaction's content: notes, transcript or summary. → *Acquire*
2. Understand and extract decisions, actions, knowledge, people and project updates. → *Understand → Reason*
3. Produce structured records for each extracted element. → *Produce*
4. Relate them to the corresponding people, projects, tasks and knowledge. → *Organize*

**Expected Outcome.** The interaction persists as an Activity, with extracted decisions, tasks, knowledge and people updates connected to the operational state.

**State Changes.**
- *Created* — Activity (Meeting); Task(s); Knowledge (decisions).
- *Updated* — Person and Project, from updates surfaced in the interaction.
- *Derived* — Knowledge (decisions, action items) extracted from the conversation.
- *Linked* — Interaction ↔ Person, Project, Task and Knowledge.

**Capabilities.** Acquire · Understand · Reason · Produce · Organize

**Domain Entities.** Activity (Meeting) · Person · Task · Knowledge · Project

**Related Use Cases.** UC-C03, UC-PE03, UC-P01

---

# People

Maintain operational knowledge about collaborators.

### UC-PE01 · Register a Person

**Intention.** Create or update a person within the system while preserving their organisational roles, affiliations and research relationships.

**Context.** Research is relational. People recur across projects, documents and interactions; a single, consistent representation of each is needed to keep that context coherent.

**Operational Flow.**
1. Acquire the person's identifying information and attributes. → *Acquire*
2. Retrieve any existing representation to avoid duplication. → *Retrieve*
3. Reconcile and register roles, affiliations and relationships. → *Organize*

**Expected Outcome.** A single authoritative Person record exists, with roles, affiliations and relationships, ready to anchor interactions and collaborations.

**State Changes.**
- *Created* — Person, when new.
- *Updated* — Person: roles, affiliations and relationships, when already present.
- *Derived* — none.
- *Linked* — Person ↔ Project and other People (relationships).

**Capabilities.** Acquire · Retrieve · Organize

**Domain Entities.** Person · Project · Knowledge

**Related Use Cases.** UC-PE02, UC-PE03, UC-C04

---

### UC-PE02 · Understand a Person

**Intention.** Retrieve the complete operational context for a collaborator on demand.

**Context.** Before or during work with someone, the researcher needs a consolidated view of everything relevant to them, drawn from across the system: projects, interactions, documents, knowledge, responsibilities and research history.

**Operational Flow.**
1. Retrieve the Person and every entity related to them. → *Retrieve*
2. Reason to assemble a coherent operational profile across projects, interactions, documents, knowledge, responsibilities and history. → *Reason (Synthesize)*
3. Produce a consolidated view of the collaborator. → *Produce*

**Expected Outcome.** A complete, current operational profile of the person is available, spanning projects, interactions, documents, knowledge, responsibilities and history.

**State Changes.**
- *Created* — none.
- *Updated* — none.
- *Derived* — Knowledge (consolidated profile of the collaborator).
- *Linked* — none.
- *Read-only: the profile is a transient synthesis and mutates no persistent state.*

**Capabilities.** Retrieve · Reason · Produce

**Domain Entities.** Person · Project · Activity · Document · Knowledge · Task

**Related Use Cases.** UC-PE01, UC-PE03, UC-C03

---

### UC-PE03 · Track Collaborations

**Intention.** Understand how collaborations evolve over time and identify the significant relationships within the research ecosystem.

**Context.** Collaborations shift over the course of a doctorate. Seeing their evolution helps the researcher recognise the relationships that matter most to the research programme.

**Operational Flow.**
1. Retrieve the history of interactions and shared activities per person. → *Retrieve*
2. Reason about the evolution, intensity and significance of each collaboration. → *Reason (Compare, Discover Connections)*
3. Produce an overview of collaboration dynamics. → *Produce*

**Expected Outcome.** The researcher understands how key collaborations have evolved and which relationships are most significant to the research ecosystem.

**State Changes.**
- *Created* — none.
- *Updated* — none.
- *Derived* — Knowledge (collaboration dynamics and significant relationships).
- *Linked* — none.
- *Read-only: the overview is a transient synthesis and mutates no persistent state.*

**Capabilities.** Retrieve · Reason · Produce

**Domain Entities.** Person · Activity · Project

**Related Use Cases.** UC-PE02, UC-C04, UC-P03

---
# Teaching

Prepare, deliver and evaluate university teaching. Teaching reuses the same knowledge, documents and people as research rather than forming an isolated workflow: courses draw on the researcher's own knowledge, and student work feeds back as evaluated understanding.

See [TEACHING_VERTICAL.md](TEACHING_VERTICAL.md) for Derived Types and cross-vertical relationships.

### UC-TE01 · Prepare a Course

**Intention.** Set up a university course (*asignatura*) as an organised initiative — objectives, teaching guide, session plan and assessment scheme — reusing existing knowledge rather than starting from a blank syllabus.

**Context.** A course recurs each academic year and must align its learning outcomes with a calendar, an official teaching guide (*guía docente*) and an assessment scheme under the institution's rules. Much of its content already exists as the researcher's own knowledge, documents and prior editions; the work is to organise it into a teachable structure.

**Operational Flow.**
1. Acquire the course's framing: degree, credits, competencies, calendar and institutional rules. → *Acquire (Register)*
2. Retrieve existing knowledge, documents and prior editions relevant to the course's topics. → *Retrieve*
3. Reason from learning outcomes to a coherent session plan and assessment scheme. → *Reason (Synthesize, Recommend)*
4. Produce the teaching guide and the course structure. → *Produce (Write, Generate)*
5. Register the course, lay out its sessions along the calendar and schedule preparation. → *Organize → Operate (Plan, Schedule)*

**Expected Outcome.** A course exists as a Project with objectives, a planned sequence of sessions and an assessment scheme, linked to the knowledge and documents it draws on, ready to drive preparation and evaluation across the term.

**State Changes.**
- *Created* — Project (Course), initial state `Proposed`; Document (Teaching Guide); Task(s), one per planned session along the calendar, plus session preparation and assessment.
- *Updated* — none.
- *Derived* — Knowledge (mapping from learning outcomes to sessions and assessments).
- *Linked* — Course ↔ Knowledge, Documents and prior editions; planned-session Tasks and deliverables ↔ Course calendar.

**Capabilities.** Acquire · Retrieve · Reason · Produce · Organize · Operate

**Domain Entities.** Project (Course) · Document (Teaching Guide) · Knowledge · Task

**Related Use Cases.** UC-TE02, UC-TE03, UC-P02

---

### UC-TE02 · Prepare a Lecture (optional, not mandatory)

**Intention.** Turn a slot in the course plan into ready-to-deliver teaching material grounded in existing knowledge and literature.

**Context.** Each session needs material pitched at the cohort and consistent with the course's objectives and what earlier sessions covered. The underlying knowledge usually already exists in the system; preparation is composition and pedagogical framing rather than origination. Delivery of the prepared material is later recorded as a Teaching Session.

**Operational Flow.**
1. Retrieve the course, the session's topic and related knowledge, documents and previous lectures. → *Retrieve*
2. Understand what earlier sessions covered and the cohort's level. → *Understand*
3. Reason about how to sequence, pitch and structure the material for teaching. → *Reason (Synthesize, Recommend)*
4. Produce the lecture material with worked examples and citations to its sources. → *Produce (Write, Generate, Cite)*
5. Register the lecture against its planned session and schedule its delivery. → *Organize → Operate (Schedule)*

**Expected Outcome.** A lecture exists as teaching material, grounded in and traceable to the system's knowledge, linked to its session in the course plan and ready to deliver. Its delivery is recorded as a Teaching Session through UC-C04.

**State Changes.**
- *Created* — Document (Lecture).
- *Updated* — Task (the session's preparation task): lecture material linked; Project (Course): session-plan progress.
- *Derived* — Document (lecture material) composed from existing knowledge.
- *Linked* — Lecture ↔ Course, Knowledge, Bibliography and previous Lectures; Lecture ↔ its planned-session Task.

**Capabilities.** Retrieve · Understand · Reason · Produce · Organize · Operate

**Domain Entities.** Project (Course) · Document (Lecture) · Task · Knowledge · Bibliography

**Related Use Cases.** UC-TE01, UC-TE03, UC-W01, UC-C04

---

### UC-TE03 · Design an Assessment (optional, not mandatory)

**Intention.** Create an assessment instrument — an exam or a continuous-evaluation assignment — with a rubric aligned to the course's learning outcomes.

**Context.** Assessment must measure the declared learning outcomes fairly and defensibly, and under continuous evaluation (*evaluación continua*) it recurs several times per term. A rubric makes the resulting grades reproducible and justifiable to students and to the institution.

**Operational Flow.**
1. Retrieve the course, its learning outcomes and the sessions the assessment must cover. → *Retrieve*
2. Reason from outcomes to what to assess and at what cognitive level. → *Reason (Synthesize)*
3. Produce the assessment instrument and a rubric mapping criteria to outcomes and marks. → *Produce (Write, Generate)*
4. Register the assessment and rubric in the course's assessment scheme and schedule it. → *Organize → Operate (Schedule)*

**Expected Outcome.** An exam or assignment exists with a rubric tied to the course's learning outcomes and placed in the assessment scheme, ready to set to students and to grade against.

**State Changes.**
- *Created* — Document (Exam) or Document (Assignment); Document (Rubric).
- *Updated* — Project (Course): assessment scheme.
- *Derived* — Knowledge (mapping from learning outcomes to assessment criteria).
- *Linked* — Assessment ↔ Course and learning outcomes; Rubric ↔ Assessment, criteria and marks.

**Capabilities.** Retrieve · Reason · Produce · Organize · Operate

**Domain Entities.** Project (Course) · Document (Exam) · Document (Assignment) · Document (Rubric) · Knowledge

**Related Use Cases.** UC-TE01, UC-TE04, UC-R01

---

### UC-TE04 · Grade Student Work

**Intention.** Evaluate student submissions against a rubric, producing consistent grades with actionable feedback.

**Context.** Grading is high-volume, recurring and must be consistent across a cohort and defensible on appeal. Applying a rubric uniformly and giving each student useful feedback is exactly the work the system can assist while the teacher retains judgement and authorship of the final mark.

**Operational Flow.**
1. Acquire the student submissions for the assessment. → *Acquire (Import)*
2. Retrieve the assessment, its rubric and the learning outcomes it measures. → *Retrieve*
3. Process and interpret each submission against the rubric's criteria. → *Process → Understand (Interpret, Classify)*
4. Reason to a proposed mark per criterion, with justification and feedback. → *Reason (Compare, Critique, Support Decision)*
5. Produce grades and per-student feedback for the teacher's approval. → *Produce (Write)*
6. Record the approved evaluations and update the course's grade record. → *Organize → Operate (Track)*

**Expected Outcome.** Each submission carries a justified grade traceable to the rubric and useful feedback, consolidated into the course's grade record. The system proposes; the teacher approves the final mark.

**State Changes.**
- *Created* — Knowledge (Evaluation) per submission; Document (feedback) per student.
- *Updated* — Document (Submission): lifecycle state `Submitted → Graded`; Project (Course): grade record.
- *Derived* — Knowledge (grade justification linking each mark to rubric criteria and evidence in the submission).
- *Linked* — Evaluation ↔ Submission, Rubric, Student and Course.

**Capabilities.** Acquire · Process · Understand · Reason · Produce · Organize · Operate

**Domain Entities.** Document (Submission) · Document (Rubric) · Knowledge (Evaluation) · Person (Student) · Project (Course)

**Related Use Cases.** UC-TE03, UC-R02, UC-W02

---

### UC-TE05 · Supervise a Final Project

**Intention.** Direct a student's final degree or master's project (TFG/TFM) from proposal to defence, keeping its evolving state connected to the researcher's own knowledge and projects.

**Context.** Supervising a TFG/TFM is where teaching meets research: the student runs a small piece of research the supervisor must scope, steer and assess over months, through recurring meetings and draft reviews, toward a defended deliverable. It reuses the whole research machinery — knowledge, literature, writing review — applied to someone else's work. In the researcher's own case this typically arises as a co-direction alongside the thesis supervisor (see [TEACHING_VERTICAL.md](TEACHING_VERTICAL.md)).

**Operational Flow.**
1. Register the final project, the student and the milestones toward defence. → *Acquire (Register) → Organize*
2. Retrieve knowledge, literature and prior projects relevant to the topic to scope it. → *Retrieve*
3. Reason to a feasible scope, objective and milestone plan. → *Reason (Recommend, Support Decision)*
4. Across supervision meetings, review the student's drafts and evidence and steer the next step. → *Understand → Reason (Critique)*
5. Track progress against milestones, recording decisions and feedback. → *Operate (Track, Review) → Organize*
6. At defence, produce the final evaluation against the assessment criteria. → *Reason (Support Decision) → Produce*

**Expected Outcome.** The final project exists as a supervised initiative with a scoped objective, milestones and a running record of meetings, drafts, feedback and decisions, connected to the knowledge it draws on and resolved by a final evaluation at defence.

**State Changes.**
- *Created* — Project (Final Project), initial state `Proposed`; Person (Student), when new; Task(s) (milestones); Knowledge (Evaluation), at defence.
- *Updated* — Project (Final Project): lifecycle state `Proposed → In Progress → Submitted → Defended`.
- *Derived* — Knowledge (scoping and supervision decisions, feedback on drafts).
- *Linked* — Final Project ↔ Student, Supervisor, Knowledge, Bibliography and related research Projects; supervision Meetings ↔ Final Project.

**Capabilities.** Acquire · Retrieve · Reason · Understand · Operate · Organize · Produce

**Domain Entities.** Project (Final Project) · Person (Student) · Activity (Meeting) · Knowledge (Evaluation) · Task · Bibliography

**Related Use Cases.** UC-TE04, UC-W02, UC-C04, UC-PE02

---

### UC-TE06 · Track a Course

**Intention.** Monitor the ongoing delivery of a course throughout the academic term, maintaining awareness of teaching progress, student engagement, assessment status and pending work so that the course remains aligned with its objectives and schedule.

**Context.** Once a course begins, the work shifts from preparation to continuous operation. Sessions are delivered, assessments are scheduled and graded, attendance varies, incidents occur and preparation for future sessions continues. Rather than reviewing many independent documents and tasks, the researcher needs a single operational view of the course that highlights progress, deviations and recommended actions.

**Operational Flow.**
1. Retrieve the course, its calendar, planned sessions, assessments, tasks and recent teaching activities. → *Retrieve*
2. Process the current operational state of the course, including completed sessions, pending preparation, assessment progress and upcoming deadlines. → *Process*
3. Reason about deviations from the teaching plan, workload, assessment timing and potential risks. → *Reason (Compare, Recommend, Support Decision)*
4. Produce an operational summary highlighting completed work, upcoming commitments, pending actions and recommendations. → *Produce (Generate)*
5. Organize follow-up work by creating, updating or reprioritising preparation tasks, assessment activities and course milestones where appropriate. → *Organize*
6. Continue operating the course until its completion while maintaining a complete operational history. → *Operate (Track)*

**Expected Outcome.** The course has an always-current operational state that reflects teaching progress, assessment completion, upcoming activities and outstanding work. Deviations from the original plan are visible, and the researcher receives recommendations that support decision-making without replacing it.

**State Changes.**
- *Created* — Task(s) for upcoming preparation or follow-up actions; Activity entries recording operational reviews (optional).
- *Updated* — Project (Course): state `Proposed → Active` as the term begins, then progress indicators; Task(s): priorities, due dates and completion; Activity (Teaching Session): `Active → Completed` as sessions are delivered; assessment schedule as evaluations progress.
- *Derived* — Knowledge (course progress metrics, teaching workload summary, student engagement indicators, operational recommendations, upcoming risks and deadlines).
- *Linked* — Course ↔ Teaching Sessions, Assessments, Tasks and Knowledge generated during delivery.

**Capabilities.** Retrieve · Process · Reason · Produce · Organize · Operate

**Domain Entities.** Project (Course) · Activity (Teaching Session) · Task · Knowledge · Document (Assessment)

**Related Use Cases.** UC-TE01, UC-TE02, UC-TE03, UC-TE04

---

### UC-TE07 · Review a Course

**Intention.** Reflect on a completed course to consolidate teaching experience into reusable knowledge, identifying what worked, what did not and what should change for future editions.

**Context.** Teaching does not end when the final grades are submitted. Every edition of a course produces valuable experience: successful explanations, ineffective assessments, recurring student misconceptions, pacing issues and organisational decisions. Capturing these lessons prevents repeating mistakes and allows each new edition to begin from accumulated teaching knowledge rather than from memory alone.

**Operational Flow.**
1. Retrieve the completed course together with its teaching sessions, assessments, student feedback, grade distributions, operational history and teaching materials. → *Retrieve*
2. Process the course's execution, then reason over it to identify deviations from the original plan, recurring issues, successful practices and measurable outcomes. → *Process → Reason (Compare)*
3. Understand the causes behind successes and shortcomings by relating operational events, teaching decisions and observed outcomes. → *Understand*
4. Reason about improvements for future editions, recommending changes to the syllabus, session sequence, assessment strategy, workload or teaching materials. → *Reason (Critique, Synthesize, Recommend)*
5. Produce a Course Review summarising lessons learned, improvement actions and recommendations for the next academic year. → *Produce (Write, Generate)*
6. Register the review, linking the resulting knowledge to the course and making it available for future course preparation. → *Acquire (Register) → Organize*

**Expected Outcome.** The completed course generates explicit teaching knowledge that becomes part of the system's long-term memory. Future editions can reuse accumulated experience instead of relying on personal recollection, enabling continuous improvement across academic years.

**State Changes.**
- *Created* — Document (Course Review); Knowledge (Teaching Insight); Task(s) for improvements to implement before the next edition (optional).
- *Updated* — Project (Course): lifecycle `Completed → Archived`; Knowledge: new relationships to teaching materials, assessments and operational decisions.
- *Derived* — Knowledge (teaching effectiveness analysis, assessment quality analysis, student performance trends, improvement recommendations, best practices and recurring issues).
- *Linked* — Course Review ↔ Course; Teaching Insight ↔ Knowledge reused during the course, Teaching Sessions, Assessments, Teaching Materials and future Course editions.

**Capabilities.** Retrieve · Process · Understand · Reason · Produce · Acquire · Organize

**Domain Entities.** Project (Course) · Knowledge (Teaching Insight) · Document (Course Review) · Activity (Teaching Session) · Task

**Related Use Cases.** UC-TE01, UC-TE02, UC-TE03, UC-TE04, UC-TE06, UC-R04

---

# Administration

The institutional obligations that surround the doctorate without directly advancing it: annual progress reports, committee and supervisor approvals, mandatory training credits, bureaucratic procedures and the deadlines that govern them. This work produces no scientific knowledge, yet failing it can halt the thesis. Its friction is administrative overload — forms, portals, regulations and hard dates scattered across institutional platforms and email — the very friction the Operational Model names among the researcher's recurring costs. These use cases treat that overload as operational state to be tracked, reasoned over and discharged, so attention returns to research. They lean on the same spine as the rest of the catalogue — Documents, Tasks, Activities and Decisions — specialized here as institutional Reports, Procedures and Administrative Processes.

See [ADMINISTRATION_VERTICAL.md](ADMINISTRATION_VERTICAL.md) for Derived Types and cross-vertical relationships.

### UC-AD01 · Prepare the Annual Doctoral Progress Report

**Intention.** Assemble the year's research progress and doctoral activities into the formal report the institution requires for annual evaluation.

**Context.** Spanish doctoral programmes require an annual evaluation: the candidate submits a record of activities and the progress of the research plan, which the Academic Committee assesses to authorise continuation. The information already exists across the system — activities, tasks, training, publications, decisions — but is scattered across a year and must be compiled into a prescribed institutional format against a hard deadline.

**Operational Flow.**
1. Retrieve the year's activities, completed tasks, training, publications and research-plan progress. → *Retrieve*
2. Interpret the programme's reporting requirements and the report template. → *Understand (Interpret)*
3. Reason over the period to synthesise advances, blockers and plan evolution against the committee's evaluation criteria. → *Reason (Synthesize, Compare)*
4. Produce the report in the required institutional format. → *Produce (Write)*
5. Register it as the deliverable of the annual evaluation process and move it toward submission. → *Organize → Operate (Track)*

**Expected Outcome.** A formal progress report exists in the prescribed format, grounded in the year's real operational record, ready for the researcher to review and submit for committee evaluation.

**State Changes.**
- *Created* — Document (Progress Report), initial state `Registered`.
- *Updated* — Project (Administrative Process · annual evaluation): state `Open → In Progress`.
- *Derived* — Knowledge (synthesis of yearly advances, blockers and plan evolution).
- *Linked* — Progress Report ↔ Administrative Process, Activities, Tasks, Documents (publications, training records) and Project (Doctoral Thesis).

**Capabilities.** Retrieve · Understand · Reason · Produce · Organize · Operate

**Domain Entities.** Document (Progress Report) · Activity · Task · Project (Administrative Process) · Knowledge

**Related Use Cases.** UC-P03, UC-AD03, UC-AD04, UC-X03

---

### UC-AD02 · Manage an Institutional Procedure

**Intention.** Carry a multi-step institutional procedure — thesis deposit, ethics approval, an extension or a change of supervisor — from requirement to completion without missing a form or a deadline.

**Context.** Institutional procedures are defined by regulations and executed through forms, portals and approvals over weeks. The requirements are buried in normative documents; the steps, their order, their deadlines and their required documents must be extracted and then tracked to completion. A single missed step can invalidate the whole procedure.

**Operational Flow.**
1. Acquire the triggering requirement and retrieve the governing regulation and any prior comparable procedure. → *Acquire → Retrieve*
2. Interpret the regulation to identify the required steps, forms, documents, approvals and deadlines. → *Understand (Interpret)*
3. Reason the requirements into an ordered set of procedures with dependencies and due dates. → *Reason (Synthesize, Prioritize)*
4. Register the procedure as an administrative process decomposed into scheduled steps. → *Organize → Operate (Plan, Schedule)*
5. Track progress and approvals across the steps until the process is resolved. → *Operate (Track, Notify)*

**Expected Outcome.** An administrative process exists, decomposed into ordered, scheduled procedures with their required documents identified, tracked from initiation to resolution.

**State Changes.**
- *Created* — Project (Administrative Process), initial state `Open`; Task (Procedure) per step; Document (forms), as required.
- *Updated* — Task (Procedure): lifecycle `Todo → In Progress → Done` as steps advance.
- *Derived* — Knowledge (the extracted step, document and deadline structure of the procedure).
- *Linked* — Administrative Process ↔ Regulation, Procedures, Documents and Project (Doctoral Thesis).

**Capabilities.** Acquire · Retrieve · Understand · Reason · Organize · Operate

**Domain Entities.** Project (Administrative Process) · Task (Procedure) · Document (Regulation) · Document · Knowledge

**Related Use Cases.** UC-AD05, UC-AD04, UC-R01, UC-P01

---

### UC-AD03 · Track Mandatory Training Activities

**Intention.** Keep an accurate, evaluation-ready record of the mandatory doctoral training completed against the credits the programme requires.

**Context.** Doctoral programmes require transversal and specific training — courses, seminars, workshops — recorded in the doctoral activity document and counted toward mandatory credits. Completed training is easily forgotten and its certificates scattered. The record must stay current because both the annual evaluation and the final deposit draw on it.

**Operational Flow.**
1. Acquire each completed training activity and its certificate. → *Acquire (Register)*
2. Process the certificate to extract its metadata: title, hours, date, type. → *Process (Extract Metadata)*
3. Classify the activity against the programme's required training categories. → *Understand (Classify)*
4. Reason the accumulated training against the requirement to surface what remains. → *Reason (Compare)*
5. Relate the activity and its certificate to the doctoral activity record. → *Organize (Relate)*

**Expected Outcome.** The doctoral training record is current and mapped to the programme's requirements, showing completed activities, their evidence and the training still outstanding.

**State Changes.**
- *Created* — Activity (Training), with linked Document (certificate).
- *Updated* — Project (Doctoral Thesis): accumulated training record.
- *Derived* — Knowledge (training completed vs required; outstanding credits).
- *Linked* — Training Activity ↔ Document (certificate), Project (Doctoral Thesis) and Regulation (training requirements).

**Capabilities.** Acquire · Process · Understand · Reason · Organize

**Domain Entities.** Activity (Training) · Document · Project (Doctoral Thesis) · Document (Regulation) · Knowledge

**Related Use Cases.** UC-AD01, UC-AD05, UC-P03

---

### UC-AD04 · Record a Committee Decision

**Intention.** Capture an institutional verdict — an annual evaluation result, a supervisor or committee approval, an ethics authorisation — as a traceable decision that updates the affected process and thesis state.

**Context.** Doctoral progress is punctuated by binding institutional decisions: the Academic Committee's annual verdict, approval of the research plan, authorisation to deposit, an ethics resolution. They arrive through committee sessions, emails or the institutional platform and change what the researcher may do next. Left uncaptured, the reason and consequences of a verdict are lost.

**Operational Flow.**
1. Acquire the verdict from its source: committee session, resolution or notification. → *Acquire*
2. Understand its content, conditions and consequences. → *Understand (Interpret)*
3. Reason over its conditions and consequences and produce a structured decision recording the verdict and its rationale. → *Reason (Support Decision) → Produce*
4. Update the affected process and thesis state and derive any obligations the verdict imposes. → *Organize → Operate*

**Expected Outcome.** The verdict persists as a decision linked to the process and people that produced it, the affected states are updated, and any obligations it imposes exist as new procedures.

**State Changes.**
- *Created* — Knowledge (Decision · the verdict and its rationale); Task (Procedure), for any obligation the verdict imposes.
- *Updated* — Project (Administrative Process): state `Under Review → Resolved`; Project (Doctoral Thesis) or Document (Progress Report): an outcome attribute set from the verdict (e.g. research plan approval, evaluation result) — a recorded value, not a lifecycle-state transition.
- *Derived* — Knowledge (conditions and consequences of the decision).
- *Linked* — Decision ↔ Activity (Meeting · committee session), Person (Supervisor, Coordinator), Administrative Process and Project (Doctoral Thesis).

**Capabilities.** Acquire · Understand · Reason · Produce · Organize · Operate

**Domain Entities.** Knowledge (Decision) · Activity (Meeting) · Person · Project (Administrative Process) · Task (Procedure)

**Related Use Cases.** UC-C04, UC-T02, UC-AD01, UC-AD02

---

### UC-AD05 · Track Institutional Deadlines

**Intention.** Surface every mandatory institutional deadline in time to act on it, so no obligation is missed.

**Context.** Enrollment windows, report submission dates, evaluation calls, fee payments and procedure deadlines are dispersed across regulations, institutional notifications and email, each with a hard, non-negotiable date. Missing one can suspend enrollment or delay the thesis. The researcher needs a single, prioritised view of what is due and when, maintained continuously.

**Operational Flow.**
1. Acquire deadline-bearing information from regulations, institutional notifications and email. → *Acquire → Process (Extract Metadata)*
2. Interpret each source to identify the obligations and their dates. → *Understand*
3. Reason over the obligations to prioritise them against their deadlines and dependencies. → *Reason (Prioritize)*
4. Register each obligation as a dated procedure and schedule timely reminders. → *Operate (Schedule, Notify, Track)*

**Expected Outcome.** A current, prioritised view of institutional obligations exists, each carrying its deadline and reminders, so nothing mandatory is missed.

**State Changes.**
- *Created* — Task (Procedure), one per identified obligation, with its deadline.
- *Updated* — Task (Procedure): scheduling and reminders; existing obligations reprioritised.
- *Derived* — Knowledge (prioritised obligation calendar).
- *Linked* — Procedures ↔ Regulation, Administrative Process and the Documents or emails that announced them.

**Capabilities.** Acquire · Process · Understand · Reason · Operate

**Domain Entities.** Task (Procedure) · Document (Regulation) · Document · Knowledge

**Related Use Cases.** UC-C01, UC-AD02, UC-P01, UC-P02

---

# Organization

The shared infrastructure research runs on — compute and GPUs, datasets, software licenses, cloud and API credits, and the budgets and grants that pay for them. Where the research use cases *consume* Resources, these use cases *manage* them, exercising the Resource entity's capacity, cost and availability and the invariant that consumption never exceeds capacity. This is the operational counterpart to scientific work: the layer that provisions experiments, tracks spend and arbitrates finite capacity across competing projects.

See [ORGANIZATION_VERTICAL.md](ORGANIZATION_VERTICAL.md) for Derived Types and cross-vertical relationships.

### UC-OR01 · Register a Research Resource

**Intention.** Bring a resource — compute, a dataset, a license, cloud credits or a budget line — under management as a single authoritative record with its capacity, cost and availability.

**Context.** Research infrastructure is acquired piecemeal and tracked in scattered places: a cluster here, a credit balance there, a licence buried in an email. Contention and overspend begin with not having one authoritative record of what exists, what it costs and how much of it remains.

**Operational Flow.**
1. Acquire the resource's identifying attributes and terms — capacity, cost, provider, access. → *Acquire (Register)*
2. Retrieve any existing representation to avoid duplicating a resource already tracked. → *Retrieve*
3. Classify it by kind — Compute, GPU, Dataset, License or Budget. → *Understand (Classify)*
4. Persist it as an available resource under management. → *Organize*

**Expected Outcome.** A single authoritative Resource exists in an `Available` state, carrying its capacity, cost, provider and availability, ready to be provisioned, consumed and accounted for.

**State Changes.**
- *Created* — Resource (Compute / GPU / Dataset / License / Budget), initial availability `Available`.
- *Updated* — Resource, when already tracked: capacity, cost or terms reconciled.
- *Derived* — none.
- *Linked* — none at registration; consumption links form as Activities draw on the Resource (UC-OR02) and funding links as Projects allocate against it (UC-OR04).

**Capabilities.** Acquire · Retrieve · Understand · Organize

**Domain Entities.** Resource (Compute) · Resource (Dataset) · Resource (License) · Resource (Budget)

**Related Use Cases.** UC-OR02, UC-OR03, UC-PE01, UC-OR04

---

### UC-OR02 · Provision Infrastructure for an Experiment

**Intention.** Reserve and ready the compute, datasets and licenses an experiment requires so it can run, without exceeding capacity.

**Context.** UC-R01 identifies an experiment's resource requirements but does not secure them. Before an experiment runs, its GPUs, datasets and license seats must be reserved against finite capacity and made ready; left unmanaged, experiments collide over the same scarce resources.

**Operational Flow.**
1. Retrieve the experiment's identified resource requirements and the resources able to meet them. → *Retrieve*
2. Reason about availability against current commitments and remaining capacity, flagging contention. → *Reason (Support Decision)*
3. Reserve the required capacity and schedule the experiment's window. → *Operate (Schedule)*
4. Ready the environment, dataset access and licence seats for use. → *Operate*
5. Record the reserved consumption and bring the Resources into use. → *Organize*

**Expected Outcome.** The experiment holds the compute, data and licenses it needs, reserved within capacity and ready to run; consumption is recorded and contention surfaced before run time rather than at it.

**State Changes.**
- *Created* — none.
- *Updated* — Resource: availability `Available → In Use` for the reserved capacity; Activity (Experiment): its required Resources reserved and recorded as consumed, readiness established without advancing the experiment's state.
- *Derived* — Knowledge (allocation decision; contention flagged when demand exceeds capacity).
- *Linked* — Experiment ↔ the Resources it will consume and the Project it serves.

**Capabilities.** Retrieve · Reason · Operate · Organize

**Domain Entities.** Resource (GPU) · Resource (Compute) · Resource (Dataset) · Activity (Experiment) · Project

**Related Use Cases.** UC-R01, UC-OR01, UC-OR05

---

### UC-OR03 · Track Resource Consumption and Cost

**Intention.** Keep a live account of how much of each resource — compute hours, credits, seats, budget — has been consumed against its capacity and cost, and surface exhaustion or overspend before it halts work.

**Context.** Compute hours, API credits and grant money deplete continuously as activities run. Left untracked, a researcher discovers a drained credit balance, an exhausted quota or a blown budget only when work stops.

**Operational Flow.**
1. Retrieve each Resource, its capacity and cost, and the consumption recorded against it across activities. → *Retrieve*
2. Normalise consumption into comparable totals per Resource, project and period. → *Process (Normalize)*
3. Compare cumulative consumption and spend against capacity and budget. → *Reason (Compare)*
4. Produce a usage-and-spend overview, flagging near-exhaustion and cost overruns. → *Produce*
5. Notify the researcher and mark any exhausted or lapsed Resource. → *Operate (Notify, Track)*

**Expected Outcome.** A current account of consumption and spend per resource and project exists, with near-exhaustion and overruns flagged and exhausted resources marked, so capacity is topped up or reallocated before it blocks work.

**State Changes.**
- *Created* — none.
- *Updated* — Resource: availability `In Use → Depleted` when capacity is exhausted, or `→ Expired` when a licence or credit term lapses.
- *Derived* — Knowledge (usage-and-spend overview; near-exhaustion and overrun alerts).
- *Linked* — the overview ↔ the Resources and Projects it concerns.

**Capabilities.** Retrieve · Process · Reason · Produce · Operate

**Domain Entities.** Resource · Resource (License) · Project · Activity

**Related Use Cases.** UC-OR02, UC-OR04, UC-P03

---

### UC-OR04 · Manage Budget and Grant Spending

**Intention.** Allocate a grant or budget across projects and their resource needs, and keep committed and actual spend reconciled against the funds available.

**Context.** A doctorate draws on grants and budgets that pay for compute, data, licences and travel. Each funding source is a finite pool; keeping every project's commitments and actual spend within it — and traceable to what the money bought — is what separates a funded programme from an overrun one. The pool is a Resource; a project's claim on it is a `FundingAllocation`, and this is where the two meet.

**Operational Flow.**
1. Retrieve the funding pool, its amount, and the projects and resources drawing on it. → *Retrieve*
2. Reason how to distribute funds across projects and their resource needs within the pool. → *Reason (Prioritize, Support Decision)*
3. Record each project's share as a funding allocation drawn against the pool. → *Organize*
4. Reconcile committed and actual spend against the pool as Resources are consumed. → *Reason (Compare) → Operate (Track)*
5. Produce the budget position and flag any project at risk of overrun. → *Produce*

**Expected Outcome.** Each project carries a funding allocation drawn from the shared pool, committed and actual spend are reconciled against it, and overrun risk is visible early enough to act on.

**State Changes.**
- *Created* — none.
- *Updated* — Project: `funding` gains a FundingAllocation earmarking the pool; Resource (Budget): capacity drawn down as spend is committed, availability `→ Depleted` when the pool is exhausted.
- *Derived* — Knowledge (budget position; per-project overrun risk).
- *Linked* — FundingAllocation ↔ Resource (Budget) and Project.

**Capabilities.** Retrieve · Reason · Organize · Operate · Produce

**Domain Entities.** Resource (Budget) · Project · Activity

**Related Use Cases.** UC-OR01, UC-OR03, UC-P03

---

### UC-OR05 · Arbitrate Resource Contention across Projects

**Intention.** When competing projects demand more of a scarce resource than it can supply, allocate its finite capacity in line with research priorities.

**Context.** Shared infrastructure — a GPU cluster, a data licence with limited seats, a capped API — is finite. When several projects need it at once, first-come allocation starves high-priority work. Contention has to be arbitrated against programme priorities, not resolved by collision.

**Operational Flow.**
1. Retrieve the contended Resource, its capacity, and the competing demands from each project and experiment. → *Retrieve*
2. Compare the demands against remaining capacity and the projects' priorities and deadlines. → *Reason (Compare, Prioritize)*
3. Decide an allocation — shares, ordering or a schedule — within capacity. → *Reason (Support Decision)*
4. Reserve capacity to the chosen work and queue or defer the rest. → *Operate (Schedule) → Organize*
5. Notify the affected projects of what was granted and what was deferred. → *Operate (Notify)*

**Expected Outcome.** The scarce resource's capacity is allocated across competing projects by priority and within its limit; granted work proceeds and deferred work is queued with a reason, so contention is resolved deliberately rather than by accident.

**State Changes.**
- *Created* — none.
- *Updated* — Resource: availability `Available → In Use` for the granted share; Task(s): deferred demands `→ Blocked` pending capacity, or rescheduled.
- *Derived* — Knowledge (allocation decision and rationale; the deferral queue).
- *Linked* — the allocation decision ↔ the Resource and the competing Projects and Experiments.

**Capabilities.** Retrieve · Reason · Operate · Organize

**Domain Entities.** Resource (GPU) · Project · Activity (Experiment) · Task

**Related Use Cases.** UC-OR02, UC-OR03, UC-X03

---

# Personal

Everyday personal organisation, served by the same system rather than a separate one.

Personal life is the lightest vertical by design. It introduces almost nothing new: a personal task is a Task, a reminder is a Task, an appointment is a Task with a time, a personal contact is a Person, a jotting is a Note (Document), and a personal goal is a Project. The same capabilities that capture a hypothesis, schedule research work and review progress serve errands, appointments and goals unchanged — no personal-specific machinery, no parallel calendar or to-do app. These three use cases exist only to prove the system spans the researcher's whole life; the personal layer is deliberately declared, not elaborated.

### UC-PL01 · Capture a Personal Note or Reminder

**Intention.** Record a personal note, reminder or errand the moment it arises, with near-zero friction, and let the system route it rather than triaging it by hand.

**Context.** Personal items surface at any moment and are lost just as quickly — something to buy, a call to make, a fleeting idea. Capture must be effortless, but a raw jotting is only useful if what is actionable becomes work to do and what is reference stays findable. The same capture and organisation the system already provides for research serve personal life unchanged.

**Operational Flow.**
1. Acquire the note as free text with minimal friction. → *Acquire (Capture)*
2. Classify it as actionable or reference. → *Understand (Classify)*
3. When actionable, determine its due date and priority and relate it to any person, goal or project it touches. → *Reason (Prioritize, Discover Connections)*
4. Persist the outcome — a Task when actionable, a Note otherwise — filed where it will be found. → *Organize (Relate)*

**Expected Outcome.** The item is captured without breaking focus: an actionable item becomes a Task with a due date, a reference item is kept as a findable Note, each linked to whatever it concerns. Nothing is lost, and nothing required a personal-specific mechanism.

**State Changes.**
- *Created* — Task, when the item is actionable; Document (Note), when it is reference.
- *Updated* — none.
- *Derived* — the actionable-versus-reference classification.
- *Linked* — Note or Task ↔ related Person, Project (Personal Goal) or Task.

**Capabilities.** Acquire · Understand · Reason · Organize

**Domain Entities.** Document (Note) · Task · Person · Project (Personal Goal)

**Related Use Cases.** UC-K01, UC-C01, UC-PL02, UC-P01

---

### UC-PL02 · Schedule a Personal Appointment

**Intention.** Register a time-bound personal commitment — an appointment, a health visit, a trip — and reconcile it against existing commitments without a separate calendar.

**Context.** Personal life is full of scheduled commitments that compete with research time: a medical appointment, travel, a family event. A commitment made but not reconciled with everything else causes conflicts. A personal appointment is not a new kind of thing — it is a Task with a time, exactly like "meet the supervisor" — so the scheduling and planning the system already performs apply directly.

**Operational Flow.**
1. Acquire the commitment with its time, place and participants. → *Acquire*
2. Retrieve existing tasks, appointments and deadlines in the affected window. → *Retrieve*
3. Detect conflicts and reason about the commitment's priority against current obligations. → *Reason (Prioritize, Compare)*
4. Register the appointment as a scheduled Task and set its reminder. → *Operate (Schedule, Notify)*

**Expected Outcome.** A scheduled Task exists for the commitment, reconciled against existing obligations with any conflict surfaced and a reminder set — using the same Task and scheduling machinery as research work.

**State Changes.**
- *Created* — Task, carrying a scheduled time and a reminder.
- *Updated* — none.
- *Derived* — any scheduling conflicts detected against existing commitments.
- *Linked* — Task ↔ the People involved and any Project it serves.

**Capabilities.** Acquire · Retrieve · Reason · Operate

**Domain Entities.** Task · Person · Project

**Related Use Cases.** UC-P01, UC-P02, UC-C03, UC-PL01

---

### UC-PL03 · Track a Personal Goal

**Intention.** Define a personal goal and review its progress over time through the tasks and activities that advance it.

**Context.** Beyond research, the researcher pursues personal goals — a fitness target, learning a language, a reading habit. A goal is not a task; it is a small initiative with an objective, its own tasks and a sense of progress. That is precisely a Project, so a Personal Goal reuses the Project structure and the same progress review used for research — only lighter, with no milestones or deliverables required.

**Operational Flow.**
1. Retrieve the Personal Goal, its tasks and the activities logged against it. → *Retrieve*
2. Reason about progress: what advanced, what stalled, and whether the goal is on track. → *Reason (Synthesize, Compare)*
3. Produce a short progress readout. → *Produce*
4. Adjust or schedule the next tasks and record the review. → *Operate (Track, Review, Schedule)*

**Expected Outcome.** The Personal Goal carries an up-to-date sense of progress, its next tasks are set, and a light review record exists — reusing the research progress-review behaviour without its heavier apparatus.

**State Changes.**
- *Created* — Project (Personal Goal), when first defined; Task(s) for the next steps.
- *Updated* — Project (Personal Goal): progress and lifecycle state (e.g. `active → achieved`); Task(s): scheduling.
- *Derived* — Knowledge (progress synthesis for the goal).
- *Linked* — Personal Goal ↔ its Tasks and Activities.

**Capabilities.** Retrieve · Reason · Produce · Operate

**Domain Entities.** Project (Personal Goal) · Task · Activity · Knowledge

**Related Use Cases.** UC-P03, UC-P02, UC-PL01, UC-PL02

---

# Context

Preserve continuity across research sessions.

### UC-X01 · Resume Research Context

**Intention.** Recover the complete operational context after an interruption and resume productive work immediately.

**Context.** Research is constantly interrupted, sometimes for long periods. The cognitive cost of reconstructing "where was I" is a primary source of friction the system exists to remove.

**Operational Flow.**
1. Retrieve the most recent operational state: active projects, tasks, hypotheses, recent activity and open threads. → *Retrieve*
2. Reason to reconstruct the context relevant to what the researcher was doing. → *Reason (Synthesize)*
3. Produce a resumable context summary. → *Produce*

**Expected Outcome.** The researcher is presented with a coherent reconstruction of their working context, minimising switching cost and enabling immediate continuation.

**State Changes.**
- *Created* — none.
- *Updated* — none.
- *Derived* — Context (resumable summary of the working state).
- *Linked* — none.
- *Read-only: reconstruction mutates no persistent state.*

**Capabilities.** Retrieve · Reason · Produce

**Domain Entities.** Project · Task · Activity · Knowledge (Hypothesis)

**Related Use Cases.** UC-X02, UC-X03, UC-X04

---

### UC-X02 · Switch Research Context

**Intention.** Safely suspend one research activity and transition into another while preserving working state.

**Context.** Research alternates between projects and tasks throughout the day. Switching safely requires preserving the state of the activity being left so it can later be resumed.

**Operational Flow.**
1. Capture and consolidate the working state of the current activity. → *Acquire → Organize*
2. Persist it as a resumable checkpoint. → *Organize (Version)*
3. Retrieve and load the context of the target activity. → *Retrieve → Reason*

**Expected Outcome.** The suspended activity's state is preserved for later resumption, and the researcher enters the new activity with its context loaded.

**State Changes.**
- *Created* — none.
- *Updated* — Activity: working state checkpointed, lifecycle `active → suspended`; target Activity: state `→ active`.
- *Derived* — none.
- *Linked* — suspended and target Activity ↔ Project.

**Capabilities.** Acquire · Organize · Retrieve · Reason

**Domain Entities.** Project · Task · Activity

**Related Use Cases.** UC-X01, UC-X04, UC-P02

---

### UC-X03 · Review Operational State

**Intention.** Provide a global overview of the current state of the research programme.

**Context.** Beyond any single activity, the researcher periodically needs the whole picture to steer: what is active, what is progressing and what needs attention.

**Operational Flow.**
1. Retrieve the state of projects, hypotheses, experiments, tasks, knowledge growth and deadlines. → *Retrieve*
2. Reason to synthesise a coherent programme-level overview. → *Reason (Synthesize)*
3. Produce the overview. → *Produce*

**Expected Outcome.** A global, current overview of the research programme exists across projects, hypotheses, experiments, tasks, knowledge and deadlines, supporting direction-setting.

**State Changes.**
- *Created* — none.
- *Updated* — none.
- *Derived* — Knowledge (programme-level overview).
- *Linked* — none.
- *Read-only: the overview is a transient synthesis and mutates no persistent state.*

**Capabilities.** Retrieve · Reason · Produce

**Domain Entities.** Project · Knowledge (Hypothesis) · Activity (Experiment) · Task

**Related Use Cases.** UC-P03, UC-X01, UC-X04

---

### UC-X04 · Close a Working Session

**Intention.** Consolidate everything produced during the current session so the next session starts from a complete state.

**Context.** Work produced in a session — knowledge, documents, tasks, decisions — must be integrated before it disperses, so continuity is preserved across sessions.

**Operational Flow.**
1. Retrieve everything created or changed during the session. → *Retrieve*
2. Understand and consolidate it into the operational state. → *Understand → Organize*
3. Persist knowledge, documents, tasks and decisions coherently. → *Organize*
4. Produce a session summary that becomes the next session's starting point. → *Produce*

**Expected Outcome.** The session's output is fully integrated into the operational state, and a summary exists to seed the next session. Nothing produced is left unconsolidated.

**State Changes.**
- *Created* — Document (Session summary / Research Journal entry).
- *Updated* — Knowledge, Document and Task produced during the session, consolidated into the operational state.
- *Derived* — Knowledge (session summary).
- *Linked* — Session ↔ every entity produced during it.

**Capabilities.** Retrieve · Understand · Organize · Produce

**Domain Entities.** Knowledge · Document · Task · Activity

**Related Use Cases.** UC-X01, UC-X03, UC-P03

---

# Traceability

Guided reconstruction of the provenance and history of scientific work.

These use cases are what separate a Research Operating System from a document store: they do not merely retrieve items, they reconstruct how a scientific decision was reached. All of them are read-only.

They span six axes: provenance (backward), decisions, recall, impact (forward), evolution (temporal) and consistency. Forward tracing (UC-T04) additionally assumes that relationships in the knowledge graph are navigable in both directions — an implication the Domain Model must honour.

### UC-T01 · Reconstruct Provenance

**Intention.** Understand why a given hypothesis, conclusion or piece of knowledge exists — the chain of reasoning and evidence that produced it.

**Context.** In a Research OS, conclusions are the product of documents, hypotheses, experiments and evidence accumulated over time. Answering "why does this exist" or "how did we arrive at this" requires reconstructing a derivation path, not merely retrieving an item.

**Operational Flow.**
1. Retrieve the target: a hypothesis, conclusion or knowledge unit. → *Retrieve*
2. Traverse its `Derived` and `Linked` history backward through the knowledge graph: evidence → experiments → hypotheses → documents. → *Retrieve → Reason*
3. Reason to reconstruct the causal derivation chain and its decision points. → *Reason (Synthesize)*
4. Produce a provenance view showing how the target was reached. → *Produce*

**Expected Outcome.** A traceable derivation path exists, showing every entity and step that led to the target, with evidence links intact.

**State Changes.**
- *Created* — none.
- *Updated* — none.
- *Derived* — Knowledge (provenance view of the derivation chain).
- *Linked* — none.
- *Read-only: reconstruction reads existing relationships and mutates no persistent state.*

**Capabilities.** Retrieve · Reason · Produce

**Domain Entities.** Knowledge · Knowledge (Hypothesis) · Knowledge (Evidence) · Activity (Experiment) · Document

**Related Use Cases.** UC-R03, UC-K05, UC-X03

---

### UC-T02 · Retrieve Decision History

**Intention.** Find every decision related to a topic, entity or question, together with its rationale and context.

**Context.** Research accumulates decisions — validating a hypothesis, discarding a paper, choosing a method — scattered across time and activities. The researcher needs to recover all decisions bearing on a subject, with their reasoning, rather than searching by memory.

**Operational Flow.**
1. Interpret the scope: a topic, entity or question. → *Understand*
2. Retrieve every decision linked to that scope across projects, activities and knowledge. → *Retrieve*
3. Reason to order and relate them: what was decided, when, why, and what superseded what. → *Reason*
4. Produce a consolidated decision history. → *Produce*

**Expected Outcome.** An ordered decision history for the scope exists, each entry carrying its rationale and links to the evidence or context that justified it.

**State Changes.**
- *Created* — none.
- *Updated* — none.
- *Derived* — Knowledge (decision history for the scope).
- *Linked* — none.
- *Read-only: the history is a transient synthesis over existing decisions.*

**Capabilities.** Understand · Retrieve · Reason · Produce

**Domain Entities.** Knowledge (Decision) · Activity · Project · Document

**Related Use Cases.** UC-T01, UC-P03, UC-C04, UC-AD04

---

### UC-T03 · Recognise Prior Work

**Intention.** Surface past work relevant to the current situation — "I know I've solved this before" — so the researcher reuses rather than redoes.

**Context.** Over years, researchers re-encounter problems they have already addressed. Without recall, effort is duplicated. The system should recognise similarity between the current context and prior activity, knowledge or decisions, and surface it proactively.

**Operational Flow.**
1. Retrieve and represent the current context: the problem, hypothesis or task at hand. → *Retrieve*
2. Reason to find semantically similar prior knowledge, activities or decisions. → *Reason (Compare, Discover Connections)*
3. Produce the matches, each with why it is relevant and where it came from. → *Produce*

**Expected Outcome.** Relevant prior work is surfaced with its context, enabling reuse and preventing duplicated effort.

**State Changes.**
- *Created* — none.
- *Updated* — none.
- *Derived* — Knowledge (relevant prior-work matches with provenance).
- *Linked* — none.
- *Read-only: recognition reads existing state and mutates nothing.*

**Capabilities.** Retrieve · Reason · Produce

**Domain Entities.** Knowledge · Activity · Knowledge (Hypothesis) · Document

**Related Use Cases.** UC-K04, UC-K05, UC-X01

---

### UC-T04 · Trace Impact

**Intention.** Given an entity, find everything downstream that depends on it, so the researcher knows what is affected if it changes, is retracted or is falsified.

**Context.** Provenance (UC-T01) answers why something exists by looking backward; impact is its inverse. When a piece of evidence is retracted, a paper turns out to be flawed, or a hypothesis is falsified, the researcher must know which conclusions, drafts and experiments rest on it. This forward propagation is a safety behaviour a document store cannot provide.

**Operational Flow.**
1. Retrieve the target entity and its outgoing relationships. → *Retrieve*
2. Traverse the knowledge graph forward through `Linked` / `Derived` edges to every dependent entity: knowledge, conclusions, drafts, experiments. → *Retrieve → Reason*
3. Reason about the nature and severity of each dependency. → *Reason*
4. Produce an impact map of what would be affected. → *Produce*

**Expected Outcome.** A forward impact map exists, listing every entity that depends on the target and how, so a change or retraction can be assessed and propagated deliberately rather than discovered by accident.

**State Changes.**
- *Created* — none.
- *Updated* — none.
- *Derived* — Knowledge (impact map of downstream dependencies).
- *Linked* — none.
- *Read-only: impact analysis reads existing relationships and mutates no persistent state.*

**Capabilities.** Retrieve · Reason · Produce

**Domain Entities.** Knowledge · Knowledge (Evidence) · Knowledge (Hypothesis) · Document · Activity (Experiment)

**Related Use Cases.** UC-T01, UC-R03, UC-W02

---

### UC-T05 · Trace Evolution

**Intention.** Reconstruct how a hypothesis, concept or understanding has changed over time — its successive formulations, state transitions and the events that drove them.

**Context.** Provenance and decision history are structural: they span entities at a point in time. Evolution is temporal, following a single entity across time. Returning to a hypothesis after months, the researcher needs to see not only its current state but how it got there. This exercises the Memory capability and the lifecycle states made explicit in the Domain Model.

**Operational Flow.**
1. Retrieve the target entity and its historical states and versions. → *Retrieve*
2. Reason to order the changes chronologically and attach the events that caused them. → *Reason (Synthesize)*
3. Produce a timeline of the entity's evolution. → *Produce*

**Expected Outcome.** A chronological account exists of how the entity was formulated, revised and transitioned between states, with each change linked to the activity or evidence that caused it.

**State Changes.**
- *Created* — none.
- *Updated* — none.
- *Derived* — Knowledge (evolution timeline of the entity).
- *Linked* — none.
- *Read-only: the timeline is a transient reconstruction over historical state.*

**Capabilities.** Retrieve · Reason · Produce

**Domain Entities.** Knowledge (Hypothesis) · Knowledge (Concept) · Activity

**Related Use Cases.** UC-R03, UC-K02, UC-X03

---

### UC-T06 · Detect Contradictions

**Intention.** Given a hypothesis, claim or body of knowledge, surface where the evidence or sources contradict it or each other.

**Context.** Retrieval and derivation assume coherence, but a growing knowledge base inevitably accumulates tensions: evidence that conflicts, sources that disagree, conclusions that no longer hold. Surfacing these is cross-sectional consistency checking, and it is one of the behaviours that most distinguishes a Research Operating System from a document store. It feeds hypothesis validation and scientific writing.

**Operational Flow.**
1. Retrieve the target and the related body of evidence and knowledge. → *Retrieve*
2. Reason across it to detect conflicts, disagreements and unresolved tensions. → *Reason (Compare, Critique)*
3. Produce a report of the contradictions with their conflicting sources. → *Produce*

**Expected Outcome.** A set of detected contradictions exists, each linking the conflicting evidence or sources, enabling the researcher to resolve tensions deliberately rather than overlook them.

**State Changes.**
- *Created* — none.
- *Updated* — none.
- *Derived* — Knowledge (report of contradictions and their conflicting sources).
- *Linked* — none.
- *Read-only: detection reads existing evidence and mutates no persistent state.*

**Capabilities.** Retrieve · Reason · Produce

**Domain Entities.** Knowledge · Knowledge (Evidence) · Knowledge (Hypothesis) · Document

**Related Use Cases.** UC-R03, UC-W02, UC-K05

---

# Evolution

This catalogue intentionally represents only the canonical operational situations of the Research Operating System.

New features should emerge by improving existing use cases before introducing new ones.

Likewise, new concepts should appear in the Domain Model only after repeated observation across multiple use cases.

Across this catalogue, **Hypothesis**, **Experiment** and **Evidence** recur as operative concepts; following the Domain Consistency principle, they have been promoted into the Domain Model as Derived Types.

**Interaction** (currently modelled as a specialization of Activity) is under observation: its centrality in UC-C03 and UC-C04, connecting People, Projects, Knowledge and Tasks, may justify promoting it to a first-class concept in the future. Teaching Session (UC-TE01, UC-TE02, UC-TE06, UC-TE07) is a second witness to the same shape — "work that occurred with people" — recorded in a different vertical. It is not promoted yet.

**Decision** recurs across research, communication, administration and traceability and is now formalized as a Derived Type of Knowledge. Its use cases continue to supply the evidence needed to refine decision-specific attributes and lifecycle rules without creating a new root.

**Evaluation** (currently modelled as a specialization of Knowledge, in Teaching) is under observation: UC-TE04 and UC-TE05 both produce it as the interpreted, justified result of assessing a Submission against a Rubric — the same structural role Evidence plays for an Experiment in the Research Spine (Domain Model → The Research Spine). This assessment-spine parallel (Rubric/Exam/Assignment → Submission → Evaluation, mirroring Hypothesis → Experiment → Evidence) is noted but not promoted yet.

**Allocation** is a further concept under observation: UC-OR02 and UC-OR05 both reserve Resource capacity ahead of actual consumption — a forward commitment that neither `ResourceUse` (actual consumption) nor `FundingAllocation` (committed funding) captures. Its recurrence may justify a first-class Resource-side reservation concept. It is not promoted yet.

This keeps the architecture grounded in real research practice while allowing the platform to evolve incrementally.
