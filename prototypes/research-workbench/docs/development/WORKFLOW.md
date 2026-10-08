# Development with agents

## Reading and authority
`AGENTS.md` is the automatic entry point for Codex. `INTENT.md` is a project document: agents read it because AGENTS explicitly instructs them to, not through an assumed automatic capability. Normative content lives in `docs/architecture/`; the original `docs/session/` archive is historical evidence and never operational instructions. That local session archive is excluded from this public snapshot.

The historical development agreement authorized complete local execution and verified local merges. The coordinating chat maintained the objective, assigned tasks, reviewed results and decided integration. A new session uses the configuration actually available to it; this conversation used the loaded tools and models explicitly selected for each delegation. The original local configuration is not distributed with this prototype.

## Installed skills
- Karpathy: `C:/Users/david/.codex/plugins/cache/karpathy-skills/andrej-karpathy-skills/1.0.0/skills/karpathy-guidelines/SKILL.md`.
- Superpowers: `C:/Users/david/.codex/plugins/cache/superpowers-marketplace/superpowers/6.4.2/skills/` with `using-git-worktrees`, `writing-plans`, `subagent-driven-development`, `test-driven-development`, `requesting-code-review`, `verification-before-completion` and `finishing-a-development-branch`.

These are references to the original workstation installation, not distributed software dependencies. If the installation changes, resolve paths from the actual catalog; do not silently download substitute instructions.

## Task cycle
1. Record BASE and read the brief covering contracts, files and tests. The ledger belongs to the plan under `.superpowers/sdd/`; its first line identifies that plan.
2. Create an `agent/<task>` branch from the latest integrated commit in a `.worktrees/<task>` checkout. Use one active product implementer and new agents for each task; independent researchers may work in parallel.
3. The implementer writes relevant tests, demonstrates failure before a change where appropriate, implements, validates, performs self-review and commits. Changes to other modules or contracts require coordination.
4. The orchestrator provides the brief, report and BASE..HEAD diff for independent review. Check specification conformity and quality; require Rust/TypeScript specialist review where applicable.
5. Important findings return to the author, and reviewers examine the correction diff. Apply the installed Superpowers limit of five rounds; never silently discard or defer a finding.
6. In a clean integration worktree, prepare the merge without committing, run relevant checks, then commit only if they pass. On failure, preserve evidence and correct the problem before integration. When a phase closes, validate the whole milestone and promote main.
7. Record commits and evidence in the ledger and maintain a current summary in `docs/STATUS.md`. Preserve final reports in a durable review archive so scratch cleanup does not lose evidence. The original [execution plans](../plans/README.md) and [reviews](../reviews/README.md) are retained through pinned historical links.

The local development agreement did not authorize publishing branches or changing external infrastructure by default. Routine confirmations were unnecessary for already authorized tasks. Tests and builds provide conventional evidence, not conclusions inferred from another model.

## Worktrees in the original conversation
During preparation, the app's native tool was attached to the original chat directory before a repository existed: its first call failed with `Not a git repository` and could not accept the new repository path. The chat subsequently worked from Research-Workbench. Git-created worktrees under `.worktrees/` were retained, with the chat as orchestrator. The native tool had no parameter for that location. These checkouts were managed by Git, not presented as registered app attachments.

Ignore `.worktrees/` before creating checkouts. Keep binaries, synthetic test libraries and backups in ignored, isolated paths. Before removing a worktree, verify that its commits are integrated and it contains no pending work; preserve useful evidence. Never use global cleanup merely to save space.

## Evidence
A backend test establishes a domain/persistence rule; a frontend test establishes UI behavior; an IPC test establishes the actual contract; a generated installer establishes packaging. Launch, upgrade, uninstall and recovery on a clean machine require separate records. Do not extrapolate from one evidence category to another.
