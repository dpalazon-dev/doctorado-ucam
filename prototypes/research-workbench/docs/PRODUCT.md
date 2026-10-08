# Research Workbench product overview

Research Workbench is an independent local Windows desktop prototype for manually reading scientific papers and retaining structured knowledge with provenance. The author has paused development and offers the source under MIT for anyone wishing to inspect or fork it.

This overview describes the current publication and its intended direction. The complete original 1 October 2026 product proposal remains [available unchanged in repository history](https://github.com/dpalazon-dev/doctorado-ucam/blob/c985b079d39ee5915c017c38c1f50b7a94526843/prototypes/research-workbench/docs/PRODUCT.md). It contains early alternatives and future roadmaps; the current normative [architecture](architecture/README.md), [domain](architecture/DOMAIN.md), [contracts](architecture/CONTRACTS.md) and [specifications](architecture/SPECS.md) govern the prototype.

## Purpose

Paper is the work unit, Concept organizes knowledge, and KnowledgeItem captures a semantic unit. Evidence and Provenance preserve traceability, Relation records explicit links, and Workflow guides manual intellectual work. A quotation, a reported result, an interpretation and a researcher hypothesis remain distinct. Completing a reading phase does not establish scientific truth.

A paper view preserves reading context; a concept view would combine contributions from multiple papers without duplicating shared concepts. The researcher controls decisions and retains uncertainty, conditions and limits. Coverage of a personal library never establishes completeness of the literature.

## Published prototype

The integrated source includes library management, native PDF selection/import, metadata, duplicate handling, an internal PDF reader, reading-position persistence, archive/restore and the PRE/P1 workflow backend. The pending workflow UI is [historical unapplied work](../../../docs/publication/pending-work/README.md). Later knowledge capture, P2 completion, search, export, backup/restore and configurable tutorial/ontology work require further implementation and validation; their contracts are not capability evidence.

The stack is Tauri 2, React/TypeScript, Rust and local SQLite. Canonical data lives in SQLite; managed PDFs live outside program binaries. Markdown/JSONL exports and graph views are proposed projections, not competing sources of truth. v0.1 excludes accounts, a server, required cloud, synchronization, OCR and LLM integration. ResearchOS at repository root is a separate future architecture with its own model.

## Intended development sequence

| Milestone | Intended scope |
| --- | --- |
| Pilot 0.0.1 | Installed desktop library/reader with clean-machine and offline verification. |
| v0.1 | PRE/P1/P2, typed knowledge, global concepts, provenance/relations, metadata/knowledge search, export, verified backup/restore and library switching. |
| Later work | P3/P4, reviewed merging/deletion, rules, graph exploration and optional assistance require separate contracts and evidence. |

The broader roadmap is a proposal, not a delivery commitment. The current [status](STATUS.md) distinguishes integrated code from pending work. [Graph/workspace](design/OBSIDIAN_WORKSPACE_AND_GRAPH.md) and [tutorial/configuration](design/TUTORIAL_AND_SCIENTIFIC_CONFIGURATION.md) proposals remain unimplemented.

## Integrity and verification

Use stable UUIDs, explicit provenance, immutable definition snapshots, optimistic revisions and atomic commits. Preserve shared knowledge when archiving sources. Explain a conflict or failed save, retain pending edits, and report success only after persistence. Use bounded native commands rather than frontend SQL or unrestricted filesystem access.

Tests use synthetic data. Frontend tests, Rust/integration tests, a packaged installer and installed QA establish different things. A generated bundle does not prove clean-machine operation. [Release criteria](release-checklist.md) and [historical execution records](reviews/README.md) keep those limits explicit.
