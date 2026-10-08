# Intent of the product

> **Publication note (8 October 2026):** this document retains the historical intention of Research Workbench. The published prototype is partial, paused and has no active maintenance due to lack of time of its author. The intention to complete v0.1 that follows does not describe an achieved result or an existing commitment. Workbench is an independent local Windows application, without AI; it does not implement ResearchOS or share its canonical model. For provenance and limits, see [the publication guide](../../docs/publication/README.md).

Research Workbench is a local Windows application for David, a researcher who transforms manual reading of papers into structured, recoverable and traceable knowledge. It is installed and opened as a desktop application; the end-user computer does not need development tools.

Paper is the unit of work; Concept organizes knowledge; KnowledgeItem captures a semantic unit; Evidence and Provenance maintain traceability; Relation expresses explicit relationships; Workflow guides intellectual work. The application retains human decisions, uncertainty and limits, without inventing evidence or converting hypotheses into results.

## Authorized result
Implement v0.1 in phases until you can import PDFs, read them, complete PRE / P1 / P2, capture typed elements and global concepts, record locators and relationships, search, export and recover a library from backups. Deliver NSIS Windows x64 installer with WebView2 offline, data outside the program directory and tests of the capabilities performed.

The first 0.0.1 pilot includes installation, library and reader. v0.1.0 adds workflow and knowledge, search, export, backup and safe restoration. Each phase is integrated after revision and validation; data integrity is not postponed to gain functionalities.

## Limits
A user, an active library and a writer; offline use; no login, HTTP server, mandatory cloud, synchronization, OCR or LLM integration. P3 / P4, fusion and final deletion will be designed later. File / restore is reversible. The program does not run the agents that develop it.

## Principles
Reliable persistence; explicit provenance; researcher control; verifiable contracts; clear English interface; minimum sufficient complexity; portability without programme dependence; recovery before scheme changes. Not to claim a capacity without proving it.

## Source and acceptance
The user requested architecture before implementing and, on October 1, 2026, authorized creating the environment, bringing all the files and fully implementing in phases with this chat as an orchestrator. The v0.2 architecture of `docs/architecture/` is adopted as a baseline of execution. The DOCX and the original plans are kept in full in `docs/session/`; its earlier formulations do not replace subsequent normative contracts.

Changing scope or decisions requires updating affected documents and recording reason, impact, migration and verification in an ADR. The acceptance of the design does not yet demonstrate the implementation and its performance.
