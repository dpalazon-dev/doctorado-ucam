# Research Vertical

> Materializes the Research vertical named in [DOMAIN_VERTICALS.md](DOMAIN_VERTICALS.md).
>
> Scientific production: turning knowledge into validated, published understanding.

---

# Purpose

Research is the vertical the rest of the domain was modelled around first. It represents the scientific process itself — hypotheses formed, tested, validated or falsified, written up and published.

Its Use Cases are the most mature in the catalogue, and its Derived Types were the first specializations of the seven Core Entities to be defined.

---

# Scope

Research covers the production of science. It does not cover the institutional obligations that surround a doctorate — annual evaluations, committee approvals, training credits, deadlines — which belong to [ADMINISTRATION_VERTICAL.md](ADMINISTRATION_VERTICAL.md).

An earlier draft of the domain named a separate **Doctorate** vertical, responsible for "producing a thesis" as distinct from "producing science." In practice it never became its own group of Use Cases: producing a chapter is Writing (UC-W01), grounded in the same Knowledge, Evidence and Bibliography as any other scientific document. Thesis *content* — its chapters and drafts — is produced as Documents through Writing, composed from what Research already produces; the Doctoral Thesis as a *project* is tracked by [ADMINISTRATION_VERTICAL.md](ADMINISTRATION_VERTICAL.md), not by Research.

---

# Derived Types

## Projects
- Research Project
- Grant

## Documents
- Paper
- Chapter / Draft
- Review
- Protocol
- Bibliography

Dataset is a Resource, not a Document — see [ORGANIZATION_VERTICAL.md](ORGANIZATION_VERTICAL.md); Research consumes it, Organization owns it.

## Activities
- Experiment
- Reading
- Analysis
- Writing

## Knowledge
- Hypothesis
- Evidence
- Methodology
- Concept
- Insight

---

# Use Cases

Full specifications live in [USE_CASES.md](USE_CASES.md). This vertical claims three of its groups.

## Knowledge

| Use Case | Intention |
|----------|-----------|
| UC-K01 · Capture a Hypothesis | Record a hypothesis the moment it appears, situated within existing knowledge |
| UC-K02 · Develop a Hypothesis | Turn a captured hypothesis into a mature, testable formulation |
| UC-K03 · Acquire Scientific Literature | Bring literature into the system as operable, connected knowledge |
| UC-K04 · Explore a Research Topic | Build progressive understanding of an unfamiliar topic |
| UC-K05 · Support a Research Question | Answer a specific question from existing knowledge without fabrication |

## Research

| Use Case | Intention |
|----------|-----------|
| UC-R01 · Design an Experiment | Turn a mature hypothesis into an executable experimental design |
| UC-R02 · Analyse Experimental Results | Interpret experimental evidence and prepare it for conclusions |
| UC-R03 · Validate a Hypothesis | Decide whether evidence supports, falsifies or leaves a hypothesis unresolved |
| UC-R04 · Generate Literature Search Strategy | Translate a knowledge gap into effective external search strategies |

## Writing

| Use Case | Intention |
|----------|-----------|
| UC-W01 · Draft Scientific Content | Compose a structured document from existing knowledge and evidence |
| UC-W02 · Review Scientific Writing | Critique clarity, consistency and evidence coverage of a document |
| UC-W03 · Generate References | Produce citations consistent with the claims they support |

---

# Relationships

Research is the primary source of Knowledge consumed by every other vertical: Teaching draws on it to prepare courses and lectures, Administration reports on it in the annual progress review. In the other direction, Research depends on Organization for the infrastructure its experiments consume (UC-R01 ↔ UC-OR02).
