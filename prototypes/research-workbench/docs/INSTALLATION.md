# Installing pilot 0.0.1

This guide describes the local per-user Windows x64 NSIS installer. The pilot has no integrated updater or Authenticode signature. Offline installation and WebView2 contents are confirmed only by inspecting and testing the frozen installer. This public repository distributes source, not a verified installer.

## Install

1. Download or copy `Research-Workbench_0.0.1_x64-setup.exe` with its `manifest.json` when a candidate is available.
2. Verify that its SHA-256 matches `artifact.sha256` in the manifest.
3. Run the installer and follow the Windows wizard. Product configuration installs for the current user and creates the `Research Workbench` Start entry.
4. Launch from Start. Installed use requires no Node.js, Rust, Cargo, Git or source checkout.

The WebView2 payload is configured to install from the package. Until clean offline testing is complete, retain the installer and treat that capability as pending.

## Local data

The release library lives in `%LOCALAPPDATA%\ResearchWorkbench\library`. Backups and logs live in sibling `backups` and `logs` folders. Preserve them when updating or uninstalling. `RESEARCH_WORKBENCH_TEST_ROOT` affects debug builds only and does not redirect release data.

## Update and uninstall

Before installing a later version, close the app and keep a copy of the library and `backups`. Pilot 0.0.1 establishes no migration from an earlier version. Install a compatible later version only when its notes identify the source version and migration test.

To uninstall, use **Windows Settings → Apps → Installed apps** and select Research Workbench. Expected policy removes binaries and shortcuts while retaining the library, backups and logs. Confirm retention through installed testing before relying on it.

## Verification

Recorded installer/test status is in [installed-pilot.md](verification/installed-pilot.md). Generating a setup does not prove clean installation, offline operation, upgrade or data preservation.
