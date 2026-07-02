# System Model

## Purpose

This document defines the conceptual model of Doctorado_UCAM.

It does not describe implementation details, technologies or software components.

Instead, it defines the fundamental concepts that compose the Research Operating System and the relationships between them.

Every architectural decision should be consistent with this model.

---

# The Research Operating System

Doctorado_UCAM is a personal Research Operating System (Research OS).

Unlike traditional research software, its purpose is not to solve isolated problems such as bibliography management, note taking or document storage.

Its purpose is to provide a unified operational environment where every capability operates over the same shared state.

The system is designed around one fundamental idea:

> The researcher should interact with a single coherent system rather than coordinating multiple disconnected tools.

---

# Conceptual Model

The system is composed of three fundamental layers.

```
                    Researcher
                         │
                         ▼
                  Research OS
                         │
        ┌────────────────┼────────────────┐
        │                │                │
        ▼                ▼                ▼
     Assets         Capabilities      Interfaces
        │                │                │
        └────────────────┼────────────────┘
                         ▼
                 Unified System State
                         │
                         ▼
                 Context for Reasoning
```

Every capability of the platform contributes to maintaining and operating over a single shared system state.

---

# Assets

Assets represent persistent knowledge and operational resources.

They define **what exists** inside the system.

Assets evolve over time and collectively describe the current state of the research activity.

Current asset categories include:

- Knowledge
- Projects
- Tasks
- Documentation
- Research Journal
- Bibliography
- Resources
- Configuration

These categories are expected to evolve as the platform matures.

---

## Knowledge

Knowledge is the primary intellectual asset of the platform.

Knowledge is not limited to scientific papers.

It includes every form of information that contributes to reasoning.

Examples include:

- Scientific literature
- Personal notes
- Skills
- Concepts
- Methodologies
- System knowledge
- Documentation
- Evidence
- Research hypotheses
- Lessons learned

Knowledge is intended to be connected, reusable and operational.

---

## Projects

Projects organize research activities around common objectives.

A project may contain:

- goals
- deadlines
- documentation
- bibliography
- tasks
- hypotheses
- datasets
- experiments
- related knowledge

Projects provide organizational structure but do not isolate knowledge.

Knowledge should remain reusable across projects.

---

## Tasks

Tasks represent actionable work.

Tasks may belong to different operational domains, including:

- Research
- Teaching
- Administration
- Software development
- Personal organization

The system should treat tasks as first-class operational entities.

---

## Documentation

Documentation represents structured information intentionally created for communication.

Examples include:

- Architecture
- Specifications
- ADRs
- Meeting notes
- Technical documentation
- Thesis chapters

Documentation differs from knowledge in that it communicates decisions rather than simply storing information.

---

# Capabilities

Capabilities describe **what the system is able to do**.

Unlike assets, capabilities do not exist independently.

They operate over the system state.

The primary capabilities are:

- Intelligence
- Memory
- Search
- Reasoning
- Automation
- Planning
- Recommendation
- Execution

---

## Intelligence

Intelligence is the cognitive engine of the platform.

Its responsibility is to operate on behalf of the researcher while preserving human control.

Intelligence should:

- understand context
- reason over knowledge
- recommend actions
- identify relationships
- detect patterns
- assist writing
- support scientific thinking

Intelligence augments the researcher rather than replacing them.

---

## Automation

Automation eliminates repetitive operational work.

Automations exist to reduce friction rather than increase complexity.

Whenever deterministic workflows can replace repetitive human effort, they should do so.

Typical automation entry points include:

- document ingestion
- email processing
- metadata extraction
- document organization
- scheduled maintenance

Automation should increasingly collaborate with Intelligence rather than exist as isolated rule-based workflows.

---

# Interfaces

Interfaces define how the researcher interacts with the system.

The interface layer should expose the system as a single coherent experience.

Examples include:

- Dashboard
- Conversational Interface
- Search
- Editors
- Knowledge Explorer
- Project Workspace

Interfaces should never expose internal implementation details.

---

# Unified System State

The Research Operating System maintains one coherent representation of the researcher's activity.

This unified state is continuously updated as assets evolve and capabilities operate.

The state is not an independent entity.

It emerges from the interaction between every component of the platform.

Every service, capability and interface should contribute to the same shared state.

---

# Context

Context is the operational view of the current system state.

It enables meaningful reasoning.

Rather than existing independently, context is continuously derived from:

- current projects
- active tasks
- recent activity
- relevant knowledge
- ongoing conversations
- historical decisions

The quality of the system depends directly on the quality of the context it can preserve.

---

# Architectural Implication

Doctorado_UCAM is not designed around tools.

It is designed around state.

Every capability operates over the same unified model.

Every interface exposes different perspectives of the same system.

Every piece of knowledge contributes to a continuously evolving understanding of the researcher's work.

Maintaining this coherence is the primary architectural responsibility of the platform.

---

# Future Evolution

This model intentionally avoids implementation details.

Technologies, databases, AI models and software architecture may evolve over time.

The conceptual model should remain stable.

Future work will refine:

- the internal asset model
- capability orchestration
- context generation
- interaction patterns
- operational workflows

without changing the fundamental principles described in this document.
