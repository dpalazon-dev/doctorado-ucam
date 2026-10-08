# English publication verification

Date: 8 October 2026.

## Baseline and scope

The language update starts from repository revision `c985b079d39ee5915c017c38c1f50b7a94526843`, after the Workbench import. ResearchOS remains the root architecture; the independent prototype remains under `prototypes/research-workbench/`. [ADR-0014](../../DECISIONS.md#adr-0014--english-repository-and-prototype-presentation) records the presentation decision.

Current documentation, interface copy, help, source comments and synthetic example labels use English. The prototype's nine normative architecture documents and two current design proposals were translated in full. Maintained development, installation and verification guides retain their original requirements and evidence limits. The current product and status overviews link to the complete superseded originals.

## Historical evidence

Historical execution plans, reviews, reports and design review records remain verbatim at the baseline revision. English indexes link to each original file:

- [Execution plans](../../prototypes/research-workbench/docs/plans/README.md)
- [Reviews](../../prototypes/research-workbench/docs/reviews/README.md)
- [Reports](../../prototypes/research-workbench/docs/reports/README.md)
- [Design proposals and historical records](../../prototypes/research-workbench/docs/design/README.md)
- [Unapplied T04c patch](pending-work/README.md)

The update removes 208 historical source paths from the current tree, including former Spanish raster previews and the unapplied patch. Every removed path has a pinned link to its exact baseline source. Git history is preserved. Historical screenshots are authentic evidence of the former interface; no edited screenshot is presented as a new English native test.

[import-manifest.json](import-manifest.json) and [VERIFICATION.md](VERIFICATION.md) describe the original import. They are historical records, not checksum verification for this later language update.

## Immutable data

The three workflow definition files in `docs/architecture/phase-definitions/` and `src-tauri/src/adapters/sqlite/migrations/0002_workflow.sql`, relative to the prototype, remain byte-identical to the baseline. Their serialized Spanish values participate in definition hashes and stored answer snapshots. Translating them in place would change identity and compatibility. Future workflow UI work must provide English display text without altering stored definitions. Bibliographic names, technical identifiers, paths and third-party notices retain their original values.

## Validation

Run from the prototype directory on Windows:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1
```

The complete gate exited with code 0: TypeScript type checking, 70 frontend tests, frontend build, Rust formatting, Clippy with warnings denied, 181 Rust test result entries and contract generation/drift checks passed. A subsequent focused library run passed all 22 tests after two final fixture-label corrections. The command registry still checks all 63 implemented commands.

Run from the repository root:

```powershell
python tools/okf_wiki.py --check
git diff --check
```

Root documentation checks passed for 36 documents and 33 owned concerns, including local links, contracts and authority rules. Diff whitespace validation passed. Independent general, Rust and TypeScript reviews are required before integration; their local reports and the task ledger are kept outside the published source tree.

These checks establish source and build validation. This update does not establish an English native GUI session, a newly validated installer or clean-machine operation. The prototype remains partial, paused and without a maintenance commitment.
