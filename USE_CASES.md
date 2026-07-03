# Use Cases

## Purpose

This document defines the canonical use cases of Doctorado_UCAM.

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

The catalogue is intentionally derived from the operational behaviour of the first working instance of Doctorado_UCAM rather than hypothetical functionality.

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
- *Created* — Document (Paper), initial state `pending`.
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
- *Created* — Activity (Experiment), initial state `designed`; Document (experimental design).
- *Updated* — Knowledge (Hypothesis): lifecycle state `developing → experimenting`.
- *Derived* — Resource requirements identified for the experiment.
- *Linked* — Experiment ↔ Hypothesis, Resource and Project.

**Capabilities.** Retrieve · Reason · Produce · Organize · Operate

**Domain Entities.** Knowledge (Hypothesis) · Activity (Experiment) · Resource · Project

**Related Use Cases.** UC-K02, UC-R02, UC-R03

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

**Context.** Doctorado_UCAM does not replace specialised literature search engines. Its role is to translate a knowledge gap into effective search strategies whose results are later brought back into the system.

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
- *Created* — Context (checkpoint of the suspended activity).
- *Updated* — Activity: state `active → suspended`; target Activity: state `→ active`.
- *Derived* — none.
- *Linked* — Checkpoint ↔ Activity and Project.

**Capabilities.** Acquire · Organize · Retrieve · Reason

**Domain Entities.** Project · Task · Activity · Context

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

**Context.** Work produced in a session — knowledge, documents, tasks, decisions, context — must be integrated before it disperses, so continuity is preserved across sessions.

**Operational Flow.**
1. Retrieve everything created or changed during the session. → *Retrieve*
2. Understand and consolidate it into the operational state. → *Understand → Organize*
3. Persist knowledge, documents, tasks, decisions and context coherently. → *Organize*
4. Produce a session summary that becomes the next session's starting point. → *Produce*

**Expected Outcome.** The session's output is fully integrated into the operational state, and a summary exists to seed the next session. Nothing produced is left unconsolidated.

**State Changes.**
- *Created* — Document (Session summary / Research Journal entry).
- *Updated* — Knowledge, Document and Task produced during the session, consolidated into the operational state.
- *Derived* — Knowledge (session summary).
- *Linked* — Session ↔ every entity produced during it.

**Capabilities.** Retrieve · Understand · Organize · Produce

**Domain Entities.** Knowledge · Document · Task · Activity · Context

**Related Use Cases.** UC-X01, UC-X03, UC-P03

---

# Evolution

This catalogue intentionally represents only the canonical operational situations of the Research Operating System.

New features should emerge by improving existing use cases before introducing new ones.

Likewise, new concepts should appear in the Domain Model only after repeated observation across multiple use cases.

Across this catalogue, **Hypothesis**, **Experiment** and **Evidence** recur as operative concepts; following the Domain Consistency principle, they have been promoted into the Domain Model as Derived Types.

**Interaction** (currently modelled as a specialization of Activity) is under observation: its centrality in UC-C03 and UC-C04, connecting People, Projects, Knowledge and Tasks, may justify promoting it to a first-class concept in the future. It is not promoted yet.

This keeps the architecture grounded in real research practice while allowing the platform to evolve incrementally.
