# Research Workbench Tutorial and Scientific Configuration

Date: October 7, 2026. Status: **design and research proposal. Not implemented and not yet a replacement for the current contracts.** Human request: a useful tutorial accessible from a button, plus configuration for publication venues, subject areas/domains, note symbols, and relationship ontology and meanings. General development authorization remains in effect. This extension requires contracts to be closed before its implementers are activated.

## 1. Objective and assumptions

The application should teach the actual research workflow and allow users to adapt their personal vocabulary without changing the meaning of older data. Retain local/offline Windows operation, one user, canonical SQLite, traceability, and no LLM. The tutorial does not assess scientific competence or certify truth.

Explicit assumption about “journal type”: cover **publication venue and article/review type as distinct fields**. This ambiguity was raised as a question. No answer was present in the recovered context. Both fields are independently useful and must not be conflated. The proposal can be corrected without changing existing data.

Verified state: product `5d35f03`, schema 2. Welcome onboarding, Library/Reader, and PRE/P1 exist. There is no repeatable tutorial, configurable semantic preference, or Knowledge/ontology implementation for 0003. DOMAIN defines twelve types and thirteen closed relationships and excludes a generic ontology editor. The new request expands that scope. Adding colors in Settings is not sufficient. The preflight is archived alongside this design.

## 2. Scientific interpretation and sources

These sources support distinctions. They do not scientifically validate our interface or establish a universal symbol vocabulary.

| Primary source consulted | Contribution | Limit on its use here |
|---|---|---|
| [W3C Web Annotation](https://www.w3.org/TR/annotation-model/) - principles and motivations | Separates body, annotated target, and intent, such as highlighting, commenting, classifying, or assessing. | Inspiration for separating a marker from content. Using these concepts does not establish JSON-LD conformance. |
| [W3C SKOS](https://www.w3.org/TR/skos-reference/) - labels, notations, notes, and relationships | Distinguishes conceptual identity, label, definition, and relationships. `related` is symmetric. | A UI icon is not automatically equivalent to `skos:notation`. `part_of` is not automatically equivalent to `broader`. Do not import external inferences. |
| [CiTO](https://sparontologies.github.io/cito/current/cito.html) - citesAsEvidence, citesAsPotentialSolution, citesAsRecommendedReading | Allows the role of a citation to be distinguished. | A citation relationship expresses how a source is used. It does not guarantee that the cited result is true. Our Evidence→Claim links are not automatically equivalent to CiTO. |
| [W3C PROV Overview](https://www.w3.org/TR/prov-overview/) | Provenance about entities, activities, and agents helps assess reliability. | Recording who asserts something, where it came from, and how it was transformed does not by itself prove validity. |
| [Cochrane Handbook, Chapter 14](https://www.cochrane.org/authors/handbooks-and-manuals/handbook/current/chapter-14) | Certainty is assessed over a body of evidence and an outcome. Consider risk of bias, inconsistency, indirectness, imprecision, and publication bias. | GRADE belongs to a specific methodological context. Do not transfer an automatic score to AI/engineering papers or equate personal confidence with GRADE. |
| [PRISMA 2020, original article](https://www.bmj.com/content/372/bmj.n71), [official site](https://www.prisma-statement.org/prisma-2020) | Reporting guideline for systematic reviews. The article warns that it is not a tool for assessing methodological quality. | Completing a checklist or workflow does not make a review correct or prove a claim. The article was retrieved through an indexed result. Full access to BMJ failed. |
| [W3C WAI, multi-page forms](https://www.w3.org/WAI/tutorials/forms/multi-page/) | Guidance, identifiable steps, and understandable progress. | Adapted to the tutorial as a design decision. It is not a study demonstrating the effectiveness of our tutorial. |

**Recommended decision:** keep the axes separate. Content type, origin, location/provenance, human confidence, and importance to the reader are not inferred from one another.

| Proposed configurable starter marker | Operational meaning | Recommended representation |
|---|---|---|
| `!` Important | Deserves attention for my reading objective. | Independent personal marker, compatible with any type. Does not increase confidence. |
| `REF` Reference | A source I want to cite, locate, or read. | Reference and its bibliographic identifier. It is not Evidence merely because a DOI exists. |
| `C` Claim | A proposition stated by a source or researcher. | Claim, explicit origin, and provenance when available. |
| `OBS` Observed/reported result | A measurement or result with method, conditions, and limits. | Evidence, distinguishing author-reported results from our own reproduction. |
| `FACT` Documented fact | A user's statement that needs context and a source. | Guided entry as Claim or Evidence depending on its content. Never automatically assigns “true” or sufficient confidence. Recommended visible label: “Reported fact.” |
| `H` Hypothesis | An original proposition awaiting testing. | Claim with `origin=researcher_hypothesis`. No incompatible new type. |
| `?` Question | Uncertainty or a research action. | Question. |
| `LIM` Limitation | An explicit constraint on a claim or method. | Limitation. |
| `IDEA` Interpretation/synthesis | An original reading that connects results. | Insight with researcher origin. |

The twelve core types remain available. This table is an abbreviated starter palette, not a replacement. Symbols are product conventions selected for usability, not universal scientific standards. Color alone is insufficient. Always provide a text label and accessible description.

**Fictional example:** “F1=0.94 on dataset X with temporal split Y” is captured as a reported result with page/table, conditions, and limitations. “The method generalizes to any plant” is a separate claim with support pending. `!` may mark both as relevant. A `supports` edge expresses interpreted support in a context. It does not turn the second statement into an established fact.

## 3. Executable tutorial

A **persistent `Tutorial` button in the shell**, also available from Help. It opens an index with an estimated duration, available modules, and local progress. Actions: Start, Resume, and Reset Progress. Do not depend on the first-run dialog. It works fully offline, with English text.

Two complementary modes:

1. **Guide in my workspace:** explains the current view, shows the next step, and lets the user return to the index. Pressing Next does not perform business mutations or change libraries. Closing or repeating the guide preserves drafts. For a real save/archive action, only the user's explicit action uses the normal workflow.
2. **Practice with an example:** a clearly labeled demonstration library, synthetic PDF, and separate entities. Use real services, not mocks that appear to persist data. Entry and exit go through the library coordinator. Save the previous context and allow it to be restored on restart. Enable this mode only after library switching/recovery and isolation have been implemented and tested. Do not copy or modify personal content for practice.

Modules in the complete delivery: (a) library and metadata, (b) PDF/locators, (c) PRE/P1 and save versus complete, (d) Claim/Evidence/Reference/importance, (e) shared concepts, relationships, and context, (f) P2 review and pending items, (g) search/export/backup/restore, and (h) customize profiles/symbols/definitions. A module can be opened directly.

Each lesson explains an objective, a concrete action, an observable result, a common error, and how to recover. Key exercise: create distinct claim and evidence records, locate the source, and link them with context while distinguishing a quotation from an original explanation. “Lesson completed” means the exercise was completed, never “paper validated.”

Functional specification TU-01…08:

- TU-01: available from a button after first run. Reopening it does not require restarting the app.
- TU-02: Previous/Next/Exit, index, and current step. Keyboard support, restored focus, screen reader, and zoom. No overlay covers the action it explains. A side-panel alternative is available.
- TU-03: progress keyed by `lessonId/tutorialVersion`, independent of `phases/answers`. Resetting the tutorial resets only that progress.
- TU-04: select lessons according to actual capabilities. Describe a missing feature as pending and do not use it to certify a completed exercise.
- TU-05: resolve targets by stable UI identifiers, not coordinates. If a target is missing, offer an explanation or return path without blocking the app.
- TU-06: saving/reopening confirms real data. Closing the guide preserves drafts. No automatic writes to the personal library.
- TU-07: practice is isolated and return is recoverable. Never delete an arbitrary root or reuse the personal library.
- TU-08: content and glossary are packaged and versioned. Examples use the selected profile and historically correct definitions.

## 4. Configurable metadata and profiles

Settings offers **Bibliography**, **Areas and Topics**, **Symbols**, **Types and Relationships**, and **Tutorial** sections. Use purpose-built forms, not a JSON or SQL editor.

Keep these concepts separate:

- **Venue:** journal, conference, or repository, with name and identifier when known. Editing the catalog does not invent quartile, indexing, or quality. Unknown values remain explicit.
- **Document/editorial type:** article, review, preprint, thesis, report, and so on. A local catalog with stable IDs, labels/definitions, and archiving. This is not the venue type.
- **Review type:** survey, SLR, mapping, and so on. Preserve the current ReviewType for PRE rules. A personal subtype must map explicitly to a canonical type or `other`. Do not infer that an experimental article satisfies a workflow designed for reviews. Describe this limitation in the UI and keep any future workflow expansion as a separate decision.
- **Domain and topics:** one primary domain plus multiple selectable topics. Provide definitions and aliases to avoid lexical duplicates. Example: water systems. Topics: anomaly detection, OT, time series. Topic tags do not automatically create scientific Concepts.

A **research profile** groups available vocabularies, symbols, ordering/favorites, and capture aids. The initial profile is general. An AI/water profile may adjust presentation and examples. Do not add Dataset/Metric/System types to the twelve types without their own contract.

When the profile changes, records retain their IDs and source definitions. New defaults affect future captures. Renaming labels does not recategorize papers. Changing a paper's classification is a real edit with revision/audit and invalidation where applicable. Archiving a term removes it from new selections while keeping historical uses readable.

## 5. Configurable ontology with history

| Option | Benefit | Cost/decision |
|---|---|---|
| Core icons and aliases only | Small change. | Insufficient for defining the requested new relationships and meanings. |
| Stable core plus versioned declarative extensions | Real customization, validation, and interpretable historical records. | **Recommended.** Requires extending contracts, persistence, export, and consumers. |
| OWL/RDF editor and general inference | Maximum expressiveness. | Complexity beyond the current need. Not recommended for v0.1. |

The relationship UI shows its name, definition, direction, allowed endpoints, valid example, counterexample, and whether it is symmetric. Users can order, hide, or favorite relationships, customize their presentation, and create their own relationships between core types. Example: `evaluates_in` Method→Condition, with context required by its definition. Do not allow executable code or free-form rules.

**Semantic identity:** `DefinitionRef=(namespace, code, version)`. Core retains `core: supports`, `contradicts`, and so on. A personal definition has a separate identity. Each saved relationship references exactly one version. A change in meaning, direction, or matrix creates a new version. Older records continue to reference the previous version. Users may write personal explanatory notes without editing the canonical definition. To change core semantics, create a personal variant with a descriptive link to the source term. Do not overwrite core.

Do not enable inferred transitivity or inversion by default. `similar_to`, `part_of`, `causes`, and `supports` are not interchangeable. A definition permits an edge but does not prove its content. Core retains its specific validations. Extensions do not automatically satisfy P2 requirements or receive confidence because their names resemble `supports`. They use generic human review until an explicit policy mapping is approved.

Local definition publication: draft → validate → publish an immutable version. Preview the change and count of uses without bulk rewriting. Applying a new definition to existing relationships requires a separate future operation with preview and audit. No silent conversion. Retiring a definition blocks new uses without breaking old ones.

## 6. Architecture and contracts to close

Scope, identity, and exchange/backup decisions are made concrete in §9 after independent review. This section lists responsibilities. Executable signatures, limits, and coordinated SQL migration remain pending.

No new server or harness. The Rust domain validates vocabulary. Transactional use cases coordinate optimistic revision checks, receipt/audit, and references. The SQLite adapter runs in the existing actor. Rust continues to generate TypeScript DTOs. The UI uses only typed commands and declared capabilities.

| Responsibility | Proposed data | Application contracts to define |
|---|---|---|
| Tutorial | `tutorialVersion`, `lessonId`, `status/lastStep`. Packaged content | Query/resume/reset progress. Practice entry/exit through the existing library mechanism. |
| Profile/presentation | `profileId/revision`, preferences. `MarkerDefinition` with code/label/glyph/help/semantic target | Read/edit profile, validate shortcut collisions, archive marker. Presets do not change confidence/origin without explicit selection. |
| Classification | Versioned terms with category `venueKind/documentKind/domain/topic`. Paper-term associations | Bounded CRUD, search, and assignment with CAS. Preserve legacy values without guessing equivalence. |
| Relationship definitions | `DefinitionRef`, definition, core matrix, symmetry, required context, status | List/view, save draft, validate/publish, retire. Create/read relationship with a versioned reference. |

`!` requires a marker-to-item association with stable identity. Do not embed it as a body prefix. A FACT preset guides capture but does not add a truth boolean. Citation and comment remain separate fields. Do not add a second canonical frontend store.

Concrete tables and ABI signatures are not closed by this document. T05a has not implemented 0003. Resolve the expanded schema before assigning it. Never edit published 0001/0002 migrations. Keep schema, ontology, tutorial, and export-format versions separate. Extending a DTO requires regeneration/verification at both ends and an explicit compatibility decision. Do not assume changing the envelope's `contractVersion` resolves semantic compatibility.

Export must include referenced historical semantic definitions and new associations. Review the allowlist and version before adding records. Visual preferences/tutorial progress are not mixed with shared scientific evidence. Local backup retains the relevant configuration needed to restore the experience. An importer that encounters an unknown definition must not reinterpret it as a known core definition. It should show an explicit limitation or reject atomically, as specified by the approved import contract. No generic import capability is claimed.

Data migration: preserve UUID, core code/type, text, and confidence. Retain existing domain/venue values. Create terms only when textual equivalence is explicit and meanings are not merged. Verify forward upgrade, reopen, backup/restore, and downgrade in diagnostic mode. Scientific history must not depend on the original visual profile remaining active.

## 7. Execution order and proposed ADR

**Proposed ADR-024:** configuration through profiles, independent markers, and a versioned declarative ontology. Repeatable tutorial separated from the scientific workflow. Affects ADR-008, DOMAIN, CONTRACTS, DATA, SPECS, QUALITY, TASK05_PORTS, TASK06_DECISIONS, and T07/T08/T09. Do not yet accept or implement incompatible interfaces based on this proposal.

1. Preserve and close T04c as a separate milestone. QA4 was interrupted: no aggregate/final exit result was recovered, and no app/driver processes were present in the October 7 inspection. PASS is not established. The merge remains prepared but uncommitted.
2. Review this proposal and close ADR-024 plus the requirements/contracts/migration matrix with independent review. Rebase the design on the final integration before dispatching product work.
3. Deliver baseline configuration and the Library/Reader/PRE-P1 tutorial slice. One author, with profiles, metadata/presentation, and tested contracts.
4. Replan T05/T06/T07 for capture, markers/definitions, and historical queries. Do not create a 0003 migration that ignores this approved change once it is closed.
5. Extend the tutorial to Knowledge/P2 when those capabilities exist. Complete practice mode after switch/recovery. T08 incorporates provenance/definition closure into export and backup. T09 tests the complete workflow.
6. The final installed gate includes offline tutorial, settings/restart, profile changes, and reading old records. Do not substitute web tests.

Configuration acceptance SC-01…08: save/reopen profiles, distinguish venue/type/topic, ensure `!` does not change type/confidence and FACT does not certify, ensure an invalid relationship rolls back without a partial receipt, preserve v1 reads after publishing v2, preserve links when terms are retired, and preserve IDs/definitions through export/restore. Symbol selection, keyboard/contrast, and shortcut-conflict tests must exercise behavior, not decorative snapshots.

## 8. Findings and limits of this research

The later review conceptually resolved the three main decisions through D1–D3 in §9. The recommended design is defined. Acceptance as an implementable contract still requires updating the affected standards and consumers.

The recommendation combines annotation/provenance standards with distinctions in evidence assessment. The consulted sources do not justify declaring our symbols universal or calculating truth from a label. Tutorial and configuration are newly recorded requirements. This document does not claim they are implemented, user-tested, or scientifically validated. The original scope of an installable application and phased execution remains.

## 9. Proposed resolutions after independent review

The review identified three decisions needed before turning the research into an implementable brief. They are specified here. **This is the recommended design, pending coordinated transfer into the standards**, not permission for a worker to combine old and new contracts.

**D1 / H1, scope.** I recommend including declarative personal relationships in the requested expanded delivery, rather than relegating them to colors. The thirteen core relationships become the required initial catalog, not the maximum user catalog. Keep the twelve item types. The extension permits new relationships between them, not new types/executable attributes or OWL reasoning. Do not activate T05a until its migration/contracts are reviewed against this decision. Continue to verify the core catalog exactly. A separate test validates extensions. This decision responds to the new human request and explicitly requires replacing the former ADR-008/DOMAIN restriction.

**D2 / H2, identity, reading, and exchange.** Concrete proposal for the ADR/ABI:

- A published definition has a composite primary key `(namespace, code, version)`, where namespace is `core` or a personal-vocabulary UUID retained in backup. Code is stable and version is an explicit semantic version. Fields: label, definition, examples/counterexamples, endpoint matrix, symmetry, context requirement, and backend hash of canonical content. The published version and its hash are immutable. Store availability for new selections separately from semantic content.
- Each Relation references those three columns through a composite FK. The new DTO uses `typeRef:{namespace,code,version}` instead of a free `typeCode`. Do not retain two fields that can contradict each other. Publishing v2 adds a row in the same namespace/code family. It does not move existing Relation FKs. Changing families uses a different code. Retiring a version does not delete it.
- Reads always resolve the exact reference, even when retired. A missing reference or inconsistent hash in the library is a visible IntegrityFailure. Do not substitute the latest version or a relationship with the same name. A valid personal definition is rendered using the same declarative form and matrix validation. No code is required for each term.
- Prepare export format **2.0**: Relation `recordVersion2` with `typeRef` and a new `recordType relationDefinition` for the exact required versions. Exporting relationships requires their definitions and all endpoints. The manifest includes files/hashes and the exact exported references, not just one version per namespace. Renaming a profile does not change identity. This is a proposed format evolution, not an implemented export.
- Backup restore requires all definitions/FKs/hashes and schema compatibility. Any absence/corruption rejects before the switch. A personal definition with a supported structure is accepted even if its namespace is new. An unsupported structure/schema version is rejected without dropping fields or reinterpreting data.
- Do not include general JSONL import in scope based on this research. If implemented later, v1 may map only the thirteen known codes to `core1.0.0` through an explicit adapter. Unknown codes are rejected before writing. Backup keeps its separate path and retains all historical definitions, whether referenced or not.

**D3 / H3, what is restored.** Profiles, catalogs, definitions, applied markers, and profile visual preferences belong to the library and travel in its backup snapshot. Restore to a new root recovers exactly that state without merging it with another library. Scientific export includes applied markers and the definitions required to interpret them, in addition to classifications/relationships. It omits favorites, shortcuts, colors, and learning progress. Add their closed records to contract 2.0 rather than embedding them freely in JSON.

Tutorial progress is also per-library and included in backup for resumption. It is not included in scientific export. It is identified by `tutorialVersion` and `lessonId`. When restored with another app version, preserve the history, but automatically resume only a compatible lesson version. Show incompatible lessons as “completed in an earlier version” and offer to start the new one. An old step never automatically credits a new lesson. The practice library has independent progress. It is not mixed with research progress. Returning from practice uses the recoverable library-switch mechanism and its local paths, which are not part of shareable export.

Exact executable signatures, limits/validation, marker/classification records, and SQL migration remain to be written as contracts. This conceptual closure resolves the three design decisions. It does not claim that this document is already a complete implementation brief.
