#!/usr/bin/env python3
"""
okf_wiki.py — OKF projection + documentation conformance check for ResearchOS.

This tool treats the repository's Markdown documents as an immutable knowledge
corpus and emits an OKF (Open Knowledge Format) projection over them: a
directory of Markdown pages with YAML frontmatter, an index, a concept-owner
map, a dependency graph and a conformance report.

Authority note
--------------
The generated bundle is a REBUILDABLE PROJECTION. The root docs remain the
single source of truth, per `DOCUMENTATION_ARCHITECTURE.md` (Core Rule) and
`AI_ARCHITECTURE.md` ("a generated wiki ... is a projection of the domain,
reconstructible from it, never authoritative over it"). This tool never edits
the source docs, and the projection must not be treated as authoritative.

The bundle is derived output and is intentionally NOT committed (see
`.gitignore`). Only this generator is versioned; run it to rebuild the bundle.

Modes
-----
    okf_wiki.py --check            Run objective conformance checks; non-zero
                                   exit if any fail. (default; used by CI)
    okf_wiki.py --build --out DIR  Emit the full OKF bundle into DIR.

The objective checks mechanize the "Conformance Checklist" already owned by
`DOCUMENTATION_ARCHITECTURE.md`. Heuristic signals are reported as advisory
only and never affect the exit code — a lexical mismatch is not a defect.

Dependencies: Python 3.9+ standard library only.
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path
from collections import defaultdict

DOC_ARCH = "DOCUMENTATION_ARCHITECTURE.md"

CONTRACT_FIELDS = {
    "level": "level",
    "normative status": "normative_status",
    "authoritative for": "authoritative_for",
    "not authoritative for": "not_authoritative_for",
    "required reading": "required_reading",
    "downstream documents": "downstream",
}

DOC_REF = re.compile(r"`([A-Za-z0-9_./*-]+\.md)`")
MD_LINK = re.compile(r"\]\(([A-Za-z0-9_./*-]+\.md)\)")
HEADER = re.compile(r"^(#{1,3})\s+(.*)$", re.M)


# --------------------------------------------------------------------------- #
# Parsing
# --------------------------------------------------------------------------- #
def repo_root(explicit: str | None) -> Path:
    if explicit:
        return Path(explicit).resolve()
    try:
        out = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            capture_output=True, text=True, cwd=Path(__file__).resolve().parent,
        )
        if out.returncode == 0 and out.stdout.strip():
            return Path(out.stdout.strip())
    except Exception:
        pass
    return Path(__file__).resolve().parent.parent


def git_date(repo: Path, name: str) -> str:
    try:
        out = subprocess.run(
            ["git", "-C", str(repo), "log", "-1", "--format=%cI", "--", name],
            capture_output=True, text=True, timeout=20,
        )
        d = out.stdout.strip()
        return d[:10] if d else "unknown"
    except Exception:
        return "unknown"


def extract_refs(s: str) -> list[str]:
    refs, seen = [], set()
    for rx in (DOC_REF, MD_LINK):
        for m in rx.finditer(s):
            r = m.group(1)
            if r not in seen:
                seen.add(r)
                refs.append(r)
    return refs


def parse_contract(text: str) -> dict:
    fields: dict[str, str] = {}
    for m in re.finditer(r"^\|\s*\*\*(.+?)\*\*\s*\|\s*(.*?)\s*\|\s*$", text, re.M):
        key = m.group(1).strip().lower()
        if key in CONTRACT_FIELDS:
            fields.setdefault(CONTRACT_FIELDS[key], m.group(2).strip())
    return fields


def parse_level_tables(text: str) -> dict[str, int]:
    level_of: dict[str, int] = {}
    for lvl in (1, 2, 3, 4):
        m = re.search(rf"^##\s+Level\s+{lvl}\b.*$", text, re.M)
        if not m:
            continue
        start = m.end()
        nxt = re.search(r"^##\s+Level\s+\d\b|^#\s+", text[start:], re.M)
        chunk = text[start: start + nxt.start()] if nxt else text[start:]
        for row in re.finditer(r"^\|\s*`([A-Za-z0-9_./*-]+\.md)`\s*\|", chunk, re.M):
            level_of[row.group(1)] = lvl
    return level_of


def parse_authority_matrix(text: str) -> list[tuple[str, list[str]]]:
    m = re.search(r"^#\s+Authority Ownership Matrix\s*$", text, re.M)
    if not m:
        return []
    start = m.end()
    nxt = re.search(r"^#\s+", text[start:], re.M)
    chunk = text[start: start + nxt.start()] if nxt else text[start:]
    rows = []
    for line in chunk.splitlines():
        line = line.strip()
        if not line.startswith("|") or line.startswith("| Concern") or set(line) <= set("|-: "):
            continue
        cells = [c.strip() for c in line.strip("|").split("|")]
        if len(cells) >= 2 and cells[0] and extract_refs(cells[1]):
            rows.append((cells[0], extract_refs(cells[1])))
    return rows


# --------------------------------------------------------------------------- #
# Model
# --------------------------------------------------------------------------- #
class Corpus:
    def __init__(self, repo: Path):
        self.repo = repo
        self.docs = sorted(p.name for p in repo.glob("*.md"))
        self.text = {n: (repo / n).read_text(encoding="utf-8") for n in self.docs}
        self.contracts = {n: parse_contract(self.text[n]) for n in self.docs}
        arch = self.text.get(DOC_ARCH, "")
        self.level = parse_level_tables(arch)
        self.authority = parse_authority_matrix(arch)
        self.existing = set(self.docs)
        self.required = {n: extract_refs(self.contracts[n].get("required_reading", "")) for n in self.docs}
        self.downstream = {n: extract_refs(self.contracts[n].get("downstream", "")) for n in self.docs}

    def real(self, refs: list[str]) -> list[str]:
        return [r for r in refs if "*" not in r and r in self.existing]

    def objective_checks(self) -> dict[str, list[str]]:
        c = self.contracts
        checks: dict[str, list[str]] = {}

        checks["C1 · every doc has a complete Document Contract"] = [
            f"{n}: missing {', '.join(f for f in CONTRACT_FIELDS.values() if not c[n].get(f))}"
            for n in self.docs if any(not c[n].get(f) for f in CONTRACT_FIELDS.values())
        ]

        broken = []
        for n in self.docs:
            for kind, refs in (("required_reading", self.required[n]),
                               ("downstream", self.downstream[n]),
                               ("body", extract_refs(self.text[n]))):
                for r in refs:
                    if "*" not in r and r not in self.existing:
                        broken.append(f"{n}: {kind} -> `{r}` does not exist")
        checks["C12 · local links resolve"] = sorted(set(broken))

        lvlmis = []
        for n in self.docs:
            m = re.search(r"Level\s+([1-4])", c[n].get("level", ""))
            decl, canon = (int(m.group(1)) if m else None), self.level.get(n)
            if decl and canon and decl != canon:
                lvlmis.append(f"{n}: declares Level {decl}, tables place it at Level {canon}")
        checks["Level self-declaration matches the level tables"] = lvlmis

        checks["Every doc is classified in a level table (no orphan)"] = [
            f"{n}: absent from all '## Level N' tables" for n in self.docs
            if n not in self.level and not n.startswith("specs")
        ]

        checks["Every Authority-Matrix concern has an existing owner"] = [
            f"'{concern}' -> `{o}` (missing)" for concern, owners in self.authority
            for o in owners if "*" not in o and o not in self.existing
        ]
        return checks

    def advisory(self) -> tuple[list[str], tuple[int, int, int, int]]:
        stop = {"and", "the", "for", "system", "rules", "semantics", "meaning",
                "ownership", "status", "current", "project", "set"}
        hints = []
        for concern, owners in self.authority:
            for o in owners:
                if "*" in o or o not in self.existing:
                    continue
                auth = self.contracts[o].get("authoritative_for", "").lower()
                keys = [w for w in re.findall(r"[A-Za-z][A-Za-z-]{3,}", concern.lower()) if w not in stop]
                if keys and not any(k in auth for k in keys):
                    hints.append(f"'{concern}' -> `{o}`: 'Authoritative for' lacks {keys[:3]} "
                                 f"(lexical only — verify)")
        req_edges = {(a, b) for a in self.docs for b in self.real(self.required[a])}
        down_edges = {(a, b) for a in self.docs for b in self.real(self.downstream[a])}
        req_recip = sum(1 for (a, b) in req_edges if (b, a) in down_edges)
        down_recip = sum(1 for (a, b) in down_edges if (b, a) in req_edges)
        return hints, (req_recip, len(req_edges), down_recip, len(down_edges))


# --------------------------------------------------------------------------- #
# Emit OKF bundle
# --------------------------------------------------------------------------- #
def _y(s: str) -> str:
    return f'"{s.replace(chr(34), chr(39))}"' if s else '""'


def _yl(xs: list[str]) -> str:
    return "[" + ", ".join(xs) + "]" if xs else "[]"


def build(cor: Corpus, out: Path) -> None:
    out.mkdir(parents=True, exist_ok=True)
    (out / "pages").mkdir(exist_ok=True)
    level_name = {1: "Canonical Foundations", 2: "Specialized Models",
                  3: "Functional / Vertical / Development",
                  4: "Navigation / Delivery / Process", 9: "Unclassified"}

    for n in cor.docs:
        c = cor.contracts[n]
        lvl = cor.level.get(n, "?")
        headers = [h for _, h in HEADER.findall(cor.text[n])
                   if h.strip() and h.strip() != "Document Contract"][:40]
        owns = [concern for concern, owners in cor.authority if n in owners]
        fm = [
            "---", f'type: "Level {lvl} document"', f"title: {_y(n)}",
            f"normative_status: {_y(c.get('normative_status',''))}",
            f"required_reading: {_yl(cor.real(cor.required[n]))}",
            f"downstream: {_yl(cor.real(cor.downstream[n]))}",
            f'source: "../../{n}"', f"timestamp: {git_date(cor.repo, n)}", "---",
        ]
        body = [
            f"# {n}", "",
            f"> **Projection** of `{n}` — non-authoritative. Canonical source: [`{n}`](../../{n}).", "",
            f"**Role:** Level {lvl} · {c.get('normative_status','—')}", "",
            f"**Authoritative for:** {c.get('authoritative_for','—')}", "",
            f"**Not authoritative for:** {c.get('not_authoritative_for','—')}", "",
            "**Concerns owned (Authority Matrix):** " + (", ".join(f"_{o}_" for o in owns) if owns else "_none_"), "",
            "**Requires:** " + (", ".join(f"[`{r}`](./{r})" for r in cor.real(cor.required[n])) or "—"), "",
            "**Feeds (downstream):** " + (", ".join(f"[`{d}`](./{d})" for d in cor.real(cor.downstream[n])) or "—"), "",
            "**Defines (sections):** " + (", ".join(headers) if headers else "—"), "",
        ]
        (out / "pages" / n).write_text("\n".join(fm + [""] + body), encoding="utf-8")

    by_level: dict[int, list[str]] = defaultdict(list)
    for n in cor.docs:
        by_level[cor.level.get(n, 9)].append(n)
    req_edges = {(a, b) for a in cor.docs for b in cor.real(cor.required[a])}
    checks = cor.objective_checks()
    obj_fail = sum(1 for v in checks.values() if v)

    idx = ["---", 'type: "OKF index"', 'title: "ResearchOS documentation wiki"', "---", "",
           "# ResearchOS — Documentation Wiki (OKF projection)", "",
           "> Rebuildable projection over the root Markdown docs. The docs are the single "
           "source of truth; this bundle is derived and non-authoritative.", "",
           f"**{len(cor.docs)} documents** · **{len(req_edges)} dependency edges** · "
           f"**{len(cor.authority)} owned concerns** · objective conformance: "
           f"{'✅ clean' if obj_fail == 0 else f'❌ {obj_fail} failing'}", ""]
    for lvl in sorted(by_level):
        idx.append(f"## Level {lvl} — {level_name[lvl]}")
        for n in sorted(by_level[lvl]):
            idx.append(f"- [`{n}`](./pages/{n}) — {cor.contracts[n].get('authoritative_for','')[:100]}")
        idx.append("")
    (out / "index.md").write_text("\n".join(idx), encoding="utf-8")

    con = ["---", 'type: "OKF concept index"', 'title: "Concept ownership map"', "---", "",
           "# Concept → Owner (Authority Ownership Matrix)", "", "| Concern | Owner | OK |", "|---|---|---|"]
    for concern, owners in cor.authority:
        ok = "✅" if all("*" in o or o in cor.existing for o in owners) else "❌"
        con.append(f"| {concern} | {', '.join(f'`{o}`' for o in owners)} | {ok} |")
    (out / "concepts.md").write_text("\n".join(con), encoding="utf-8")

    g = ["---", 'type: "OKF dependency graph"', 'title: "Required-reading graph"', "---", "",
         "# Documentation dependency graph", "", "`A --> B` = **A requires B** as reading.", "",
         "```mermaid", "graph LR"]
    for a, b in sorted(req_edges):
        g.append(f'  {a[:-3].replace("-","_")} --> {b[:-3].replace("-","_")}')
    g.append("```")
    (out / "graph.md").write_text("\n".join(g), encoding="utf-8")

    hints, (rr, rn, dr, dn) = cor.advisory()
    log = ["---", 'type: "OKF lint report"', 'title: "Documentation conformance lint"', "---", "",
           "# Documentation conformance lint", "",
           f"Corpus: **{len(cor.docs)} docs**. Objective conformance: "
           f"**{'CLEAN ✅' if obj_fail == 0 else f'{obj_fail} check(s) FAILING ❌'}**.", "",
           "## Objective conformance (repo Conformance Checklist)", ""]
    for check, viols in checks.items():
        log.append(f"- {'✅' if not viols else '❌'} {check}" + (f" — {len(viols)} issue(s)" if viols else ""))
        for v in viols:
            log.append(f"    - {v}")
    log += ["", "## Advisory (heuristic — human confirmation required)", "",
            f"- Ownership lexical hints: {len(hints)} (lexical mismatch only; not defects until confirmed)"]
    for v in hints:
        log.append(f"    - {v}")
    log += ["",
            f"- required-reading ⇄ downstream coupling (informational, not a repo rule): "
            f"{rr}/{rn} required-edges reciprocated by a downstream entry; "
            f"{dr}/{dn} downstream-edges reciprocated by a required entry.",
            "  The two fields are defined as *different* relations, so partial overlap is expected."]
    (out / "log.md").write_text("\n".join(log), encoding="utf-8")


# --------------------------------------------------------------------------- #
# CLI
# --------------------------------------------------------------------------- #
def run_check(cor: Corpus) -> int:
    checks = cor.objective_checks()
    obj_fail = sum(1 for v in checks.values() if v)
    print(f"ResearchOS documentation conformance — {len(cor.docs)} docs, "
          f"{len(cor.authority)} owned concerns\n")
    print("=== OBJECTIVE CONFORMANCE (DOCUMENTATION_ARCHITECTURE.md checklist) ===")
    for check, viols in checks.items():
        print(f"  {'✅ PASS' if not viols else f'❌ FAIL ({len(viols)})'}  {check}")
        for v in viols:
            print(f"       - {v}")
    hints, (rr, rn, _dr, _dn) = cor.advisory()
    print("\n=== ADVISORY (heuristic — does not affect exit code) ===")
    for v in hints:
        print(f"  ~ {v}")
    print(f"  ~ coupling: {rr}/{rn} required-edges also declared downstream (informational)")
    if obj_fail:
        print(f"\nRESULT: ❌ {obj_fail} objective check(s) failing.")
        return 1
    print("\nRESULT: ✅ all objective conformance checks pass.")
    return 0


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="OKF projection + doc conformance check for ResearchOS.")
    mode = ap.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true",
                      help="Run objective conformance checks; non-zero exit on failure (default).")
    mode.add_argument("--build", action="store_true", help="Emit the OKF bundle.")
    ap.add_argument("--repo", default=None, help="Repository root (default: git toplevel).")
    ap.add_argument("--out", default="wiki", help="Output dir for --build (default: ./wiki).")
    args = ap.parse_args(argv)

    cor = Corpus(repo_root(args.repo))

    if args.build:
        out = Path(args.out).resolve()
        build(cor, out)
        print(f"OKF bundle written to: {out}")
        return 0

    # default: --check
    return run_check(cor)


if __name__ == "__main__":
    sys.exit(main())
