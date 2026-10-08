# Workspace and Research Graph

Date: October 7, 2026. Status: extension design requested by the user. This document does not establish implementation. It complements the tutorial and scientific-configuration design. Both designs should be transferred to the contracts together before T05 is activated.

## Objective

Give Research Workbench an Obsidian-inspired desktop interface and make it possible to explore real connections through a global graph and a paper-centered graph. Users can record connections between papers with a typed meaning or as simple links. From the graph, they can open a paper, inspect a connection, and continue exploring.

The request also sets the editorial style. All maintained prose and all original interface text should avoid the em dash and semicolon. Keep those characters where required in code syntax, URLs, user data, verbatim quotations, and historical files. The application never rewrites researchers' notes or quotations to impose this style.

## Visual decisions

Use a dark, compact workspace with a selectable light theme. Obsidian's spatial organization and visual calm are the inspiration. Do not copy its brand, custom icons, or code.

| Element | Intended behavior |
|---|---|
| Narrow tool rail | Library, global graph, search, tutorial, and settings, with accessible names and visible help |
| Left panel | Navigation and paper list, collapsible and resizable |
| Central area | Work tabs for the library, a paper, and the graph. Opening a node reuses that paper's tab |
| Paper | Reading, Processing, Knowledge, and Local Graph sections |
| Right panel | Properties and connections for the selection, collapsible. The tutorial uses this space without covering fields |
| Bottom bar | Active library, saved state, and recovery state where applicable |

Replace the current oversized cards and green background with flat surfaces, subtle dividers, and typographic hierarchy. Preserve the PDF's original presentation. Do not invert its colors by changing its subject.

Initial dark palette: background `#17171b`, panel `#202025`, active surface `#292930`, edge `#383840`, primary text `#eeeef2`, secondary text `#b4b4c0`, and accent `#b39aff`. Light palette: background `#faf9fc`, panel `#f0eef5`, text `#24222b`, secondary text `#5e586c`, and accent `#6844ad`. These are initial values subject to contrast measurement, not a validated accessibility result.

Use Segoe UI, 14 px base text, and restrained headings. Buttons should be compact while retaining adequate hit areas. Provide visible focus, non-disruptive shortcuts, and panels that collapse before content is clipped. All actions must remain available at 200% zoom. Theme, layout, and reduced motion are presentation preferences. They do not modify scientific data.

Keep a button labeled Tutorial accessible in every workspace. Tab navigation retains session drafts and the reader's position. Closing a tab with pending changes offers Save, explicitly Discard, or Cancel. Returning to the graph restores its center, filters, and selection while the library remains active.

## What connections represent

Maintain three categories with an explicit legend.

1. **Link between papers.** An explicit record created by the researcher. It may be simple or have a versioned semantic definition.
2. **Relationship between knowledge elements.** The existing Relation connects KnowledgeItems according to the ontology. It continues to preserve its endpoints, context, and provenance.
3. **Derived association.** For example, two papers share a Concept UUID. This is an explainable view of existing links. It is not stored as a new assertion and is not labeled as a citation.

By default, the Papers view shows explicit connections. An optional layer shows associations based on shared concepts. The Knowledge view shows items, concepts, and typed relationships. This separation prevents every coincidence from creating a tangle of links or appearing to be a scientific conclusion.

A simple link means only that I have associated these papers. It does not imply citation, support, contradiction, causation, or equivalence. A paper does not become a KnowledgeItem, and a Reference does not replace the paper's identity.

## PaperLink as a separate addition

Propose `PaperLink` with a UUID, `sourcePaperId`, `targetPaperId`, optional `typeRef`, context, justification, declared origin, lifecycle, revision, and timestamps. The endpoints are foreign keys to Papers in the same library. Retain the existing receipt, audit, and CAS patterns within a DB-actor transaction.

- Without `typeRef`, the link is symmetric. The backend normalizes the UUID pair. No scientific type is invented by default.
- With `typeRef`, resolve the exact definition by namespace, code, and version. The definition permits Paper endpoints and declares direction or symmetry. Do not reuse a KnowledgeItem matrix as though it permitted Papers.
- Extend definitions with a closed scope, `paper_link` or `knowledge_relation`. Scope is part of immutable content. Personal catalogs may include both scopes, but each definition family belongs to only one.
- No self-links, missing endpoints, or links between libraries. Creating or modifying a link requires both papers to be active. Archiving a link remains allowed even if an endpoint is archived. Restoring it requires both endpoints to be active.
- Archiving a paper retains its links. Hide them by default when either endpoint is archived. The archived-items filter allows users to inspect them with their visible state. Restoring the paper shows links that were still active, without restoring links that were explicitly archived.
- Enforce active uniqueness, where applicable, over normalized endpoints, the exact type or absence of a type, and normalized context. Different contexts and different types allow parallel links. A collision during create, update, or restore produces an atomic Conflict.
- Context uses the normalization already defined for relationships. The exact algorithm must be specified in the shared contract before implementation. Retain the original text shown to the researcher.
- Changing type or direction is an explicit revision. Publishing another version of a definition does not change existing links. If a typed link loses its definition, report an integrity error. Never silently convert it to a simple link.
- Context and justification describe the researcher's judgment. Optional provenance reuses existing locators through its own association. A literal quotation retains its source and locator. A missing locator is shown as pending, without preventing a simple personal link.
- PaperLink does not automatically satisfy a P2 gate that requires a Relation between KnowledgeItems. It also does not change the confidence of connected papers or items.

The initial catalog of paper connections may include `cites`, `extends`, and `compares_with`, each with a definition, direction, examples, and reviewed limits. Names alone do not establish that these connections exist. Do not infer them from PDFs or similar titles.

For `cites`, the meaning is “the researcher records that the source paper cites the destination.” This is a human assertion, not an automatic check. Without a locator, the form, inspector, and list show “Citation recorded. Provenance pending.” With a locator, they show “Citation recorded. Source is locatable” and allow the user to open it. Neither state is labeled as a verified quotation. Bibliographic certification, if required, would need a separate human-validation contract. The existence of a citation also does not validate the cited content. Transfer this distinction to CONTRACTS and SPECS before enabling the catalog.

## Global and local graph

Open the global graph from the main navigation. It shows papers in the selected scope, their links, and, if enabled, derived associations. The local graph centers on the open paper and shows its neighbors at depth 1 by default, with selectable depth from 1 to 3. Traversal includes incoming and outgoing links and preserves direction arrows.

Clicking a node selects it and opens its properties. Double-clicking it or choosing Open Paper opens its workspace. The Center Here action changes the center of the local graph. This separation lets users explore without losing the graph. Back restores the previous selection and center.

Clicking an edge shows its meaning, direction, type and version when present, context, origin, and available provenance. Simple links are labeled No type. Derived associations use a different line style and list the shared concepts that explain them. Do not offer editing controls on a derived edge.

Actions: zoom in, zoom out, fit to view, search visible nodes, filter by title/domain/relationship type, show archived items, toggle isolated nodes, and choose layers. Graph search filters metadata already supported by the product. Do not introduce a second FTS syntax.

Create Relationship from a paper or node opens a form with a destination-paper selector. Type is optional, and the form explains its meaning before saving. The connection appears only after backend confirmation. If an edit conflicts, retain the draft and allow the record to be reloaded.

Papers without links remain accessible. An empty local graph shows its node and “This paper has no connections yet,” with an Add Connection action. An empty library, a query with no matches, unavailable capability, and query error have distinct messages. Never include demonstration nodes in a real library.

Node position, size, and visual proximity do not represent quality, truth, or scientific relevance. Use uniform size by default. Visually group parallel relationships with a count and allow them to be inspected separately.

## Architecture and limits

SQLite remains the only canonical source. Do not add a server, graph database, or duplicate frontend store. Graph is a read projection. PaperLinks has its own domain, use cases, repository, and commands.

Proposed boundaries: `domain/paper_links`, `application/paper_links`, a SQLite adapter, closed link IPC commands and graph query, Rust-generated DTOs for TypeScript, `features/graph` for navigation/representation, and `features/paper-links` for forms. Final ABI names, permissions, and tables must be closed in CONTRACTS and DATA before product work is dispatched.

A graph response includes typed identities to avoid collisions between Paper and KnowledgeItem, complete endpoints, edge type, references to the exact definition, a derived-association indicator, and truncation metadata. No edge may point to a missing node. Resolve the query against a coherent read snapshot.

Initial product limits: up to 300 nodes and 1000 edges per view, with a maximum depth of 3. Queries go through the existing DB actor. The backend limits expansion during traversal, not after materializing the entire library. Selection is deterministic, prioritizes the central node, and orders candidates by UUID. The response indicates whether it was truncated and lets users narrow filters. Never present a subset as the entire library.

Calculate layout outside the interaction thread when expensive, stop when stabilized, and allow it to be paused. Changing libraries cancels queries and discards stale responses. Coordinates are reconstructible presentation state. A rendering failure offers the connection list with the same actions.

For implementation, evaluate a maintained graph component that works offline and supports arrows, selection, and resource limits. Verify the dependency, version, and license in a bounded spike when that task is dispatched. Do not select or install a library based on a mockup. Building a custom engine would add unnecessary complexity.

## Accessibility and performance

Every graph action has a keyboard equivalent and an accessible list of nodes and connections. Accessible names include title and type. Do not communicate direction, selection, or categories through color alone. Escape closes the popover inspector or cancels an interaction without deleting data.

Reduced motion disables continuous animation. Panels and menus respect focus and tab order. Measure AA contrast in both themes and verify the workflow at 200% zoom. PDF reading, PRE/P1, and their drafts must pass regression after the shell changes.

Proposed budgets to measure on the reference machine: query and first usable view in under 2 s for 300 nodes and 1000 edges, and selection feedback in under 100 ms once stabilized. Record hardware, data, and the 95th percentile across 20 runs. These are pending test criteria, not demonstrated performance.

## Persistence, export, and migration

Do not edit published migrations 0001 or 0002. Coordinate the configuration and knowledge schema before creating 0003. PaperLinks may use a separate later migration to avoid coupling all capture workflows to the graph.

Backup and restore include links, historical definitions, and provenance associations. Export adds explicit PaperLink records and their definitions in the same proposed 2.0 format review as scientific configuration. A paper export includes its incident links and endpoint metadata. Do not traverse all links transitively from added papers. Also include Documents, Papers, and locators referenced by provenance. Do not include every neighboring PDF unless it belongs to the required documentary closure and PDFs were requested.

Closure is finite, based on sets of visited IDs. Always export exact definitions and both endpoints. Specify the export policy for archived records in CONTRACTS before implementation. Restore preserves IDs and revisions without reinterpreting a retired type. Do not add a generic importer or claim capabilities that do not exist.

## ADR and phased delivery

Proposed ADR-025: workspace with panels, separate PaperLink, and derived graph. Coordinate it with ADR-024 for tutorial/configuration. The human request changes the scope that excluded the graph, but this document alone does not replace the current ABI.

1. Close the ADRs and review affected DOMAIN, CONTRACTS, DATA, SPECS, QUALITY, and briefs. Remove the exclusion of the bounded graph while keeping automatic inferences, executable plugins, and generic ontology editing out of scope. Do not activate T05 with its old fixed catalog.
2. Integrate T04c when its pending native QA is complete. Do not make aesthetic changes to the frozen candidate during this validation.
3. UI-01 applies the visual system and editorial policy to the actual Library, Reader, and PRE/P1 screens. UI-02 adds panels and navigation while preserving drafts. No active button simulates future capabilities.
4. PL-01 implements PaperLink and its scoped catalog, with domain/persistence tests. PL-02 adds forms, listing, and navigation between papers. It works without a graph engine.
5. GR-01 implements global/local queries and an accessible list. GR-02 adds interactive rendering and filters. Add the knowledge layer when T05/T07 are integrated.
6. Extend the tutorial with link creation, meaning inspection, local/global navigation, and the distinction between a link and a derived association. Coordinate export/backup with T08 and validate the installed workflow in T09/T10.

Each cut has a new product author, independent review, and verified merge by the orchestrator. Detailed planning must set signatures and commands against integrated code before delegation. Workers are not authorized to invent contracts from this proposal.

## Acceptance

| ID | Verifiable result |
|---|---|
| ED-01 | Maintained prose and original visible text contain no em dash or semicolon. Necessary code, citations, and user data are preserved |
| UI-01 | Coherent dark/light themes, measured focus and contrast, panels usable at 200% |
| UI-02 | Navigating between paper and graph preserves drafts and reader position |
| PL-01 | Creating a simple A/B link and querying it from A or B returns the same record |
| PL-02 | A directed link retains its exact direction and definition after restart |
| PL-03 | Self-link, missing endpoint, and collision fail atomically. CAS and replay preserve integrity |
| PL-04 | Archive/restore respects endpoints and lifecycle without deleting links |
| GR-01 | Global and local views show only real records from the active library |
| GR-02 | Local view respects depth and limits. Every edge has both endpoints and truncation is visible |
| GR-03 | Select, open, focus, and return work with mouse and keyboard |
| GR-04 | A derived edge explains its origin and is never edited as an assertion |
| GR-05 | Changing libraries discards stale results. Rendering errors preserve the accessible list |
| PT-01 | Export/restore preserves links, endpoints, definitions, and provenance without unbounded transitive closure |
| TU-09 | Tutorial teaches the graph and meaning of connections using isolated practice data |

## References and artifacts

The [official Obsidian graph documentation](https://help.obsidian.md/plugins/graph), consulted on October 7, 2026, describes global and local views, local depth, filters, and navigation between notes. It is used as an interaction reference. The scientific semantics and links between papers in this document are Research Workbench design decisions.

The [OBSIDIAN_WORKSPACE_MOCKUP.svg](./OBSIDIAN_WORKSPACE_MOCKUP.svg) mockup shows the visual direction using synthetic titles. It is a static design image, not a product screen or an operational test.
