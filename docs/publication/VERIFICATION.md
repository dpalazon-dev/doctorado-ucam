# Publication verification — 2026-10-08

Scope: repository consolidation, independent prototype import and MIT publication of original material. No product behavior or canonical domain contract was changed. No installer was published or installation claim added.

## Recorded source cuts

- ResearchOS main before consolidation: 75db725.
- Workbench integrated product: 702d08b02f2d2655584a932455ac394fb026f8d2.
- Workbench current documentation: 6e20428fe8267dddac3f4a8adca41a158314fd05 (added/modified documents only; integrated delivery records retained).
- Design proposals: 1f598cbe0bb36d5b0fd8f65c530337a7b36378e6.
- Pending T04c UI: 67446412f9c2f82cab63ccb5bf02d3dff695bb84, preserved as an unapplied patch.

## Checks executed in the consolidated checkout

From `prototypes/research-workbench/`:

```powershell
npm.cmd ci
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1

```

All returned exit 0. The full gate ran TypeScript checking, 69 frontend tests (10 files), the Vite build, Rust formatting, Clippy with warnings denied, Rust tests and generated-contract drift checking. There were 181 passed Rust test entries across the gate, including helper entries, with zero failed. Vite retained its advisory about a JavaScript chunk over 500 kB; no functionality was changed to silence it.

From the repository root:

```powershell
python tools/okf_wiki.py --check
git diff --cached --check -- .gitignore CLAUDE.md DECISIONS.md DOCUMENTATION_ARCHITECTURE.md README.md LICENSE docs/publication/README.md docs/publication/VERIFICATION.md docs/publication/pending-work/README.md
git apply --check --directory=prototypes/research-workbench docs/publication/pending-work/T04c-workflow-ui.patch
```

All returned exit 0. The actual Python executable used was the bundled local runtime; the system WindowsApps alias did not provide a usable interpreter in this session. The documentation checker covered 36 root documents, 33 owned concerns and all five objective checks. Its lexical advisories remain informational. Prototype historical documentation is outside that root corpus.

204 imported product/configuration/script/test files were checked against their source Git blobs and SHA-256 values before testing, then checked again by SHA-256 after the gate. All matched. `import-manifest.json` contains the file inventory. Product source and lockfiles remain byte-identical to the integrated source cut.

## Independent review

- Rust relocation review: approved; relative build/configuration paths and 157 relevant inventory entries checked. Independent cargo checks passed.
- TypeScript relocation review: approved; Vite/PDF workers and assets, IPC types, fixtures and project scripts inspected. No import regression found.
- Publication audit: 82 ResearchOS commits / 185 unique blobs plus 500 snapshot/publication source files inspected. No confirmed secrets or third-party personal data detected; five retained images use synthetic examples. The command and root-cache-ignore findings were corrected before publication.

An unscoped whitespace check over every newly imported file flags pre-existing Markdown hard breaks, historical end-of-file blank lines and literal patch-context whitespace. Those source bytes are deliberately retained; the scoped check above passes for the new publication/editing files.

Original Workbench branches, local uncommitted changes and the prepared T04c merge were preserved. A verified local Git bundle retains its complete local history outside the public repository; the public import is a curated snapshot. ResearchOS history remains intact.

## Limits

These checks verify this import and its build/test behavior. They do not establish clean-machine installation, a new native GUI session, final v0.1 acceptance, or exhaustive absence of secrets. Workbench remains partial and paused; the T04c UI patch and later phases are not active capabilities in the imported product. Third-party dependencies retain their licenses and notices.
