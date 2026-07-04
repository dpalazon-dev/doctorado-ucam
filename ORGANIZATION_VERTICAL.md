# Organization Vertical

> Materializes the Organization vertical named in [DOMAIN_VERTICALS.md](DOMAIN_VERTICALS.md).
>
> The shared infrastructure research runs on.

---

# Purpose

Compute and GPUs, datasets, software licenses, cloud and API credits, and the budgets and grants that pay for them. Where Research's Use Cases *consume* Resources, this vertical's Use Cases *manage* them — capacity, cost and availability, and the invariant that consumption never exceeds capacity.

This is the operational counterpart to scientific work: the layer that provisions experiments, tracks spend and arbitrates finite capacity across competing projects.

---

# Scope

Organization owns the Resource entity's lifecycle end to end: registration, provisioning, consumption tracking, budget reconciliation and contention arbitration. It does not own the projects or activities that consume Resources — those belong to whichever vertical is doing the consuming (typically [RESEARCH_VERTICAL.md](RESEARCH_VERTICAL.md), via experiments).

It is distinct from [ADMINISTRATION_VERTICAL.md](ADMINISTRATION_VERTICAL.md): Organization manages infrastructure; Administration discharges institutional obligations. See that document for the boundary in more detail.

---

# Derived Types

## Resources
- Compute
- GPU
- Dataset
- License
- Budget

Each Resource carries capacity, cost and availability (`Available → In Use → Depleted / Expired`).

## Value Objects reused here

- **FundingAllocation** — not a Resource itself; a Project value object (Logical Domain Model → Block 0) earmarking an amount against a Resource (Budget) for a period. UC-OR04 is where the two meet: the pool is a Resource this vertical owns, the claim on it is a FundingAllocation the Project owns.

A candidate concept is under observation here, not yet promoted — see Use Cases → Evolution. **Allocation**: UC-OR02 and UC-OR05 both reserve Resource capacity ahead of actual consumption, a forward commitment neither ResourceUse nor FundingAllocation captures today.

---

# Use Cases

Full specifications live in [USE_CASES.md](USE_CASES.md) → Organization.

| Use Case | Intention |
|----------|-----------|
| UC-OR01 · Register a Research Resource | Bring a resource under management as a single authoritative record |
| UC-OR02 · Provision Infrastructure for an Experiment | Reserve and ready what an experiment requires, within capacity |
| UC-OR03 · Track Resource Consumption and Cost | Keep a live account of usage and spend against capacity and budget |
| UC-OR04 · Manage Budget and Grant Spending | Allocate a funding pool across projects and reconcile spend against it |
| UC-OR05 · Arbitrate Resource Contention across Projects | Allocate scarce capacity across competing demands by priority |

---

# Relationships

Organization provisions what Research's experiments consume (UC-OR02 ↔ UC-R01). It has no Use Case that produces scientific or teaching content — its output is always availability, cost visibility or an allocation decision, never Knowledge.
