# Administration Vertical

> Not present in the original six-vertical sketch in [DOMAIN_VERTICALS.md](DOMAIN_VERTICALS.md). It emerged directly from the Use Case catalogue and is formalized here.
>
> The institutional obligations that surround the doctorate without directly advancing it.

---

# Purpose

Annual progress reports, committee and supervisor approvals, mandatory training credits, bureaucratic procedures and the deadlines that govern them. This work produces no scientific knowledge, yet failing it can halt the thesis.

Its friction is administrative overload — forms, portals, regulations and hard dates scattered across institutional platforms and email — the friction the [Operational Model](RESEARCHER_OPERATIONAL_MODEL.md) names among the researcher's recurring costs. These Use Cases treat that overload as operational state: tracked, reasoned over and discharged, so attention returns to research.

---

# Scope and History

Earlier drafts of the domain named a distinct **Doctorate** vertical — "Research produces science; Doctorate produces a thesis" — covering thesis chapters, milestones and defence preparation together. Once expressed as Use Cases, that single vertical split cleanly in two, and neither half needed anything new:

- Producing a chapter is Writing, part of [RESEARCH_VERTICAL.md](RESEARCH_VERTICAL.md) (UC-W01) — a Document composed from Knowledge and Evidence like any other.
- Everything institutional around the thesis — the annual evaluation, procedures, training, committee decisions, deadlines — is this vertical.

Doctorate is retired as a named vertical because both of its concerns already had a home before the retirement was noticed.

Administration is distinct from [ORGANIZATION_VERTICAL.md](ORGANIZATION_VERTICAL.md): Administration discharges institutional *obligations* (reports, approvals, credits); Organization manages shared *infrastructure* (compute, budget, licenses). A budget line is a Resource Organization manages; the annual report that accounts for how a grant was spent is a Document Administration produces.

---

# Derived Types

## Projects
- Doctoral Thesis
- Administrative Process

## Documents
- Progress Report
- Regulation
- institutional forms

## Tasks
- Procedure

## Knowledge
- Decision

Reuses Activity (Meeting, Training) and Person (Supervisor, Coordinator) without specialization.

---

# Use Cases

Full specifications live in [USE_CASES.md](USE_CASES.md) → Administration.

| Use Case | Intention |
|----------|-----------|
| UC-AD01 · Prepare the Annual Doctoral Progress Report | Assemble the year's progress into the formal report the institution requires |
| UC-AD02 · Manage an Institutional Procedure | Carry a multi-step procedure — deposit, ethics, extension — to completion |
| UC-AD03 · Track Mandatory Training Activities | Keep an evaluation-ready record of training completed against required credits |
| UC-AD04 · Record a Committee Decision | Capture an institutional verdict as a traceable decision |
| UC-AD05 · Track Institutional Deadlines | Surface every mandatory deadline in time to act on it |

---

# Relationships

Administration reports on Research and Teaching without altering how either operates: UC-AD01 retrieves "activities, completed tasks, training, publications and research-plan progress" but produces only a Document, changing no upstream state. Committee decisions (UC-AD04) can update the Doctoral Thesis project's state directly — the one point where an institutional verdict has authority over research-facing state.
