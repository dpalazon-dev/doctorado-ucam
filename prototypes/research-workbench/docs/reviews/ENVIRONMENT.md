# Environment review

Reviewed on 2026-10-01. Scope: the full environment/configuration package in `work/reviews/environment-diff.md`, commits `d758f6c` and `1c7c005`, and relevant current files. Product implementation, pending planner outputs, and archived product-design details are outside this task.

## Spec compliance verdict

PASS for the reviewed environment setup. `AGENTS.md` and `INTENT.md` state purpose, accepted baseline, explicit document authority, local execution authorization, worktree isolation, Sol coordination, Luna implementation, sequential product workers, independent reviews, and evidence gates. Historical session content is explicitly non-operative. Project-local agent files have narrow roles and do not redefine global security settings. The archived material has 42 manifest entries, all present with matching size and SHA-256; Git tracks those 42 files plus the manifest, totaling 43 entries.

This is not acceptance of the whole phase-zero deliverable: the master implementation plan and current-guidance document are being prepared by the planner and need their own preflight. Their temporary absence is recorded as unfinished parallel work, not as a final omission in this reviewed package.

## Quality verdict

APPROVE. The initial MEDIUM documentation finding was corrected and its scoped re-review passed. No CRITICAL, HIGH or other open issues remain in this environment package.

### [MEDIUM, ADDRESSED] Developer shell instructions mix cmd.exe and PowerShell syntax

File: `docs/development/ENVIRONMENT.md:29`; affected commands at lines 35–38.

Issue: the first recipe opens and stays inside `cmd.exe /k` after running `vcvars64.bat`. The next instruction says to execute the Rust commands “there”, but uses PowerShell's `& 'path'` syntax. Those lines are invalid in the cmd shell that the guide just opened. Executing them back in the original PowerShell instead loses the MSVC/SDK environment created in the child cmd process, defeating the reproducible developer-shell recipe.

Reproduction: follow the documented shell choice and paste line 35 into cmd. The invocation operator/single-quoted path is not a valid cmd program invocation. This is a shell-language error; repeating the already recorded Rust smoke compilation is unnecessary to establish it.

Fix: either document cmd commands using a double-quoted executable path, or open a child PowerShell after `vcvars64.bat` so it inherits the configured environment. Keep the code-block language and “there” instruction consistent with the chosen shell. Validate the corrected recipe with a read-only version check rather than repeating the compilation.

Scoped re-review: `work/reviews/environment-fix-diff.md` changes the recipe to `cmd.exe /c 'call "...vcvars64.bat" && powershell.exe -NoExit'` and explicitly names the child PowerShell for subsequent commands. This is correct: cmd first sets its environment, then starts PowerShell with that inherited environment. A read-only probe used the same `call ... && powershell.exe` chain, substituting a terminating `-NoProfile -NonInteractive -Command "Get-Command cl.exe | Select-Object -ExpandProperty Source"` for the interactive `-NoExit`. It exited 0 and resolved MSVC `14.39.33519` at `HostX64\x64\cl.exe` after reporting x64 environment initialization. No compilation was repeated. Verdict: ADDRESSED; no new breakage in the fix diff.

## Evidence and limits

- Initial `git diff --staged` and `git diff` were empty; recent commits and the supplied committed diff defined the review range. The planner's untracked task brief was observed but not treated as part of this environment package.
- The four `agents` scalar keys, the primary model keys, and standalone `.codex/agents/*.toml` fields match the current official [configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference) and [custom-agent schema](https://learn.chatgpt.com/docs/agent-configuration/subagents). The concurrency limit counts spawned threads and excludes the primary, matching the project's three-subagent policy. No obsolete-key finding applies.
- `WORKFLOW.md` distinguishes next-session project configuration from the tools/models already loaded in this conversation. A configuration file being valid does not itself demonstrate that a new session has loaded it.
- The configured implementer/researcher use Luna. Sol review has an explicit correctness/integrity role; no unexplained high-cost model escalation was found.
- `.gitignore` isolates worktrees, scratch output, build artifacts, databases and secrets, while explicitly allowing archived `docs/session/work/` files. Git's 43 tracked archive paths confirm that the historical work files survive the scratch ignore rule.
- Git identity and line-ending settings are repository-local. No global Codex config or security setting change appears in the reviewed changes. The review does not claim to reconstruct every earlier workstation mutation from a Git diff.
- Rust installer, checksum, throwaway smoke source and artifacts exist in ignored `work/toolchain/`. The recorded compilation was not rerun. The guide correctly distinguishes toolchain smoke evidence from a built, installed, or tested application.
- No application functionality is claimed complete: STATUS lists product phases as pending; README and INTENT describe intended behavior and acceptance targets. Installed-app, upgrade, clean-machine and recovery evidence remain future gates.
- No source, configuration, toolchain, global state or archived originals were modified by this review. Only this report was written.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 open; 1 addressed | pass |
| LOW | 0 | pass |

Verdict: APPROVE — the shell-documentation finding is addressed; no open environment-package findings remain. Pending planner deliverables retain their own preflight gate.
