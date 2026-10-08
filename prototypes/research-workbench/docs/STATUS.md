# Publication status

Updated: 8 October 2026. Research Workbench is a partial Windows 11 x64 prototype, paused because its author has no time for active maintenance. Source is available under MIT for independent development. Publication offers no support or delivery commitment.

## Integrated source and pending work

| Scope | State |
| --- | --- |
| Library and reader | Integrated source: native PDF import, metadata, duplicate handling, managed copies, reader and reading-position persistence, archive/restore. |
| PRE/P1 backend | Integrated; definitions, answers, gates and transaction rules are implemented in the imported source. |
| PRE/P1 workflow UI (T04c) | Unapplied; preserved through the [historical patch](../../../docs/publication/pending-work/README.md). |
| P2 and knowledge capture | Normative design available; complete production resolvers and UI remain pending. |
| Search, export, backup/restore and library switching | Contracts/design available; remaining implementation and acceptance are pending. |
| Graph/workspace and tutorial/configuration | Design proposals only; not implemented. |
| Installation on a clean/offline machine | Pending; historical debug QA and static installer inspection do not close this gate. |

The original product snapshot came from `702d08b02f2d2655584a932455ac394fb026f8d2`. Documents came from `6e20428fe8267dddac3f4a8adca41a158314fd05`; proposals came from `1f598cbe0bb36d5b0fd8f65c530337a7b36378e6`. The repository now uses English maintained documentation and interface copy. Immutable v1 workflow payloads retain their original hashes and data values. See [publication provenance](../../../docs/publication/README.md).

## Historical evidence

The full local execution chronology—including original commits, test counts, review findings, candidate hashes and incomplete QA attempts—remains [unchanged at the original imported revision](https://github.com/dpalazon-dev/doctorado-ucam/blob/c985b079d39ee5915c017c38c1f50b7a94526843/prototypes/research-workbench/docs/STATUS.md). Detailed [reviews](reviews/README.md), [reports](reports/README.md) and [plans](plans/README.md) have English indexes linking each authentic historical file. Read every result with its date, source commit and limits; old task assignments are not current operational instructions.

Recorded native debug QA verified portions of the reader flow; a corrected installer candidate passed static inspection, while clean installation remained pending. The pending T04c integration/QA was never accepted into the imported active product. No new English native GUI or installed-machine result is claimed by the language change. Current language-change validation is recorded in [LANGUAGE_CHANGE.md](../../../docs/publication/LANGUAGE_CHANGE.md).

## Continuing development

Start with [INTENT](../INTENT.md), the [architecture index](architecture/README.md), [SPECS](architecture/SPECS.md) and [CONTRACTS](architecture/CONTRACTS.md). Preserve the Workbench/ResearchOS boundary and existing data compatibility. Implement and review pending capabilities before exposing controls; use synthetic libraries for checks and keep native/installer acceptance separate from automated tests.
