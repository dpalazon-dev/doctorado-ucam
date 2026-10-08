# Native WebDriver profile and close follow-up — 2026-10-02

**Outcome: PARTIAL.** This is a separate follow-up from the earlier frozen-binary smoke. One approved run used the same immutable T02 executable; no build, install, product edit, library-personal access, or rerun occurred.

## Artifacts and command

Run directory: `work/qa/t02-native-smoke/profile-close-attempt-20261002-4ecddb6dca5f483b8402a09728e19059/`.

- Frozen executable source and copied run executable both matched SHA-256 `C3FB989E17B20640FB4C530CC08959CDC839A72E5DF54E5442827F20837924D8`.
- Reviewed QA script `native-smoke.ps1`: SHA-256 `C168CE9FCEB317F04702FDD2F6C5B5B1B93D434B2B572400EEA32DE5AE840684`.
- Observer wrapper `run-with-process-observer.ps1`: SHA-256 `5C7A7B1EB162671E4077956E4AF1E8A2892BEB538B46C11B7E847CA3EEF1790E`.
- Command: `pwsh -NoProfile -File .\work\qa\t02-native-smoke\profile-close-attempt-20261002-4ecddb6dca5f483b8402a09728e19059\run-with-process-observer.ps1`. Observer wrapper exit code 0; the QA script exit code was 1 with status `partial; CloseMainWindow did not produce observed exit code 0`. Exact request/response log and the script result are retained in `smoke-result.json`; observer summary is `process-evidence.json`.
- Tauri driver 2.1.0 SHA-256 `2684A7B9E1E667D6689BEA8EAFF2AEFE093C5B1F5CA329B3156B384A9CAB9286`; EdgeDriver 154.0.4258.48 SHA-256 `4032E9D74DA42B30805EC5125EEAF9CBD4F678C2A41104EE90471BE16CAAD997`. Edge and WebView2 Runtime were both 154.0.4258.48.

## Observed UI, profile, and close

A real WebDriver session was created and deleted successfully. The native WebView displayed the first-run confirmation, accepted `Entendido`, and opened Settings with `Estado: Disponible`. Screenshot [settings-smoke.png](C:/Users/david/Projects/Research-Workbench/work/qa/t02-native-smoke/profile-close-attempt-20261002-4ecddb6dca5f483b8402a09728e19059/settings-smoke.png), 116,412 bytes, SHA-256 `468D2360C0D9A7C821311CDEFC399A02187F9918C0FF603CFF0BF9F936F6B296`.

The exact session request included `tauri:options.webviewOptions.userDataFolder` set to the unique run-local directory. EdgeDriver still returned an empty `msedge.userDataDir`. The requested directory contained 126 files, and the reviewed script found six `msedgewebview2.exe` processes whose `--user-data-dir` argument matched that run path and whose parent chain reached the one app PID (45608). The result stores their PIDs and parent-chain IDs, not raw command lines. The separate live observer did not capture a sample; that does not override the scoped process evidence saved by the QA script. This supports use of the run-local WebView2 profile for those processes. It does not establish isolation of every Tauri app write: Tauri may also create its default app-local-data directory. No personal LocalAppData contents were read or cleaned.

`Process.MainWindowHandle` was nonzero (`11141912`), `CloseMainWindow()` returned `true`, and the synthetic lifecycle log recorded `started` at `13:44:03.785Z` followed by `stopped` at `13:44:05.925Z`. The script called `WaitForExit(15000)` before `DELETE /session`, but it did not store the Boolean result or the elapsed time. The JSON `appExitCode: null` and `normalClose: false` do not prove that the wait returned false: the time between `started` at 13:44:03.785Z and JSON creation at approximately 13:44:06.434Z is only about 2.65 seconds, so a full 15-second timeout is not supported by the recorded chronology. A PowerShell property getter can also yield null when the `ExitCode` getter throws, without preserving the exception in these fields. Therefore neither timeout nor exit code 0 is established. After `DELETE /session`, the exact app process was absent. The log and accepted close request do not prove process exit code 0, and the disappearance after session cleanup is not classified as a normal close. The cause remains unresolved; see [.superpowers/sdd/IMPLEMENTATION/native-close-diagnosis.md](../.superpowers/sdd/IMPLEMENTATION/native-close-diagnosis.md) and retain the evidence for a follow-up measurement.

## Cleanup and limits

The script's scoped cleanup reported no exact copied-exe process, the test's tauri-driver PID exited, and its EdgeDriver child was absent. Subsequent CIM and listener queries used `-ErrorAction Stop`; they found no process with the exact app/driver executable paths and no listener on the run's dynamically selected ports 11863 or 11864. The synthetic log, screenshot, WebDriver trace, and generated library database remain under this run folder for review.

This run did not test the OS advisory writer lock, picker, reader/PDF rendering, NSIS installation, or a clean Windows machine. It does not validate installation or total write isolation. The absence of an app exit code is the unresolved close result; no further run was performed.
