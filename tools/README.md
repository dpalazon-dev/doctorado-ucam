# tools/ — Documentation conformance tooling

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 4 — Navigation, Delivery and Process (operational tooling) |
| **Normative status** | Non-normative. Operational tooling documentation. |
| **Authoritative for** | How to run the documentation conformance check and how to rebuild the OKF projection. |
| **Not authoritative for** | Documentation governance rules (owned by `DOCUMENTATION_ARCHITECTURE.md`), domain semantics, architecture, or Spec status. |
| **Required reading** | `DOCUMENTATION_ARCHITECTURE.md`. |
| **Downstream documents** | None. |

> The rules this tool verifies are owned by `DOCUMENTATION_ARCHITECTURE.md`. This tool only mechanizes them; it does not define them.

---

## What this is

`okf_wiki.py` treats the repository's root Markdown documents as an immutable
knowledge corpus and does two things:

1. **Conformance check** (`--check`) — mechanizes the *Conformance Checklist*
   owned by `DOCUMENTATION_ARCHITECTURE.md`: every document has a complete
   Document Contract, every local link resolves, each document's declared
   level matches the level tables, no document is orphaned, and every concern
   in the Authority Ownership Matrix has an existing owner. Exits non-zero on
   any failure. This is what CI runs.

2. **OKF projection** (`--build`) — emits an
   [Open Knowledge Format](https://cloud.google.com/blog/products/data-analytics/how-the-open-knowledge-format-can-improve-data-sharing/)
   bundle over the corpus: one Markdown page per document with YAML
   frontmatter (its Document Contract, made machine-readable), plus an index,
   a concept→owner map, a required-reading dependency graph, and a lint report.

## Authority

The generated bundle is a **rebuildable projection**. The root docs remain the
single source of truth — see the Core Rule in `DOCUMENTATION_ARCHITECTURE.md`
and `AI_ARCHITECTURE.md` ("a generated wiki ... is a projection of the domain,
reconstructible from it, never authoritative over it"). The tool never edits
the source docs, and the projection must not be treated as authoritative.

Because the bundle is derived output, it is **not committed** (see the root
`.gitignore`). Only this generator is versioned; run it to rebuild.

## Usage

```sh
# Run the objective conformance check (default; non-zero exit on failure)
python3 tools/okf_wiki.py --check

# Rebuild the OKF projection into ./wiki/ (gitignored)
python3 tools/okf_wiki.py --build --out wiki
```

No third-party dependencies — Python 3.9+ standard library only.

## Design notes

- **Objective vs advisory.** Only objective, rule-based checks affect the exit
  code. Heuristic signals (e.g. a lexical mismatch between a concern name and
  its owner's "Authoritative for" text) are reported as *advisory* and require
  human confirmation — a lexical mismatch is not a defect.
- **Scope.** The tool scans root-level `*.md`. When `specs/` is introduced by
  `SPEC-001`, extend the glob accordingly.
- **Governance.** This is documentation-conformance tooling, not product code.
  Repository build/CI is formally owned by `SPEC-001`; when that Spec is
  authored, this check can be folded into it.

## Possible extensions

- Transitive closure of required-reading ("everything I must read to change X").
- Cross-check that `README.md`, `ROADMAP.md`, `SPEC_CATALOG.md` and `CLAUDE.md`
  report the same project state (Conformance Checklist item 10).
- A query mode (`--who-owns <concern>`, `--impact <doc>`).
