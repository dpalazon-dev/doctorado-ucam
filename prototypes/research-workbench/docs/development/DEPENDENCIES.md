# Dependency and packaging research

Checked 2026-10-01 against npm registry metadata, crates.io metadata, and official framework documentation. Versions below are registry latest stable releases at check time, except Tauri's Rust crates explicitly held to the stable 2.x line because 3.x is alpha. This is a compatibility snapshot, not a lockfile; commit lockfiles to make builds reproducible.

## Recommended compatible versions and licenses

| Package | Version | License | Relevant constraint / note |
| --- | --- | --- | --- |
| `@tauri-apps/cli` | 2.12.1 | Apache-2.0 OR MIT | Node >=10; align major/minor with Tauri Rust/API. |
| `@tauri-apps/api` | 2.12.1 | Apache-2.0 OR MIT | Keep on Tauri v2 line. |
| Rust `tauri` | 2.12.1 | Apache-2.0 OR MIT | MSRV 1.90; project Rust 1.99 satisfies it. |
| Rust `tauri-build` | 2.7.1 | Apache-2.0 OR MIT | MSRV 1.90; this is the current build crate dependency range used by `tauri` 2.12.1. |
| `react` / `react-dom` | 19.3.0 / 19.3.0 | MIT / MIT | React DOM peers on React `^19.3.0`. |
| `@types/react` / `@types/react-dom` | 19.3.0 / 19.3.0 | MIT / MIT | Keep type packages on React 19; `@types/react-dom` peers on `@types/react ^19.3.0`. |
| `vite` | 8.3.2 | MIT | Node `^20.19.0 || >=22.12.0`; installed Node 24.14.1 satisfies. |
| `@vitejs/plugin-react` | 6.1.1 | MIT | Vite peer `^8.0.0`; Node `^20.19.0 || >=22.12.0`. |
| `typescript` | 7.0.2 | Apache-2.0 | Node >=16.20.0. |
| `tailwindcss` / `@tailwindcss/vite` | 4.3.3 / 4.3.3 | MIT / MIT | Tailwind v4 Vite plugin supports Vite `^5.2 || ^6 || ^7 || ^8`; use the plugin with Vite. |
| `@radix-ui/react-dialog` | 1.1.23 | MIT | React and React DOM peer ranges include 19. |
| `react-hook-form` | 7.89.0 | MIT | React peer range includes 19; Node >=18. |
| `zod` | 4.6.5 | MIT | Zod v4. |
| `@hookform/resolvers` (if using RHF + Zod) | 5.9.1 | MIT | Peers support RHF `^7.55.0` and Zod `^3.25.0 || ^4.0.0`; current RHF/Zod fit. |
| `pdfjs-dist` | 6.3.289 | Apache-2.0 | Node >=22.13.0 or >=24; installed Node 24.14.1 satisfies. Recheck target WebView/browser support when integrating. |
| `vitest` | 5.0.3 | MIT | Vite peer includes 8; Node `^22.12 || ^24 || >=26`. |
| `@testing-library/react` | 16.3.3 | MIT | React/React DOM peers include 19; requires `@testing-library/dom >=10`. |
| `@testing-library/dom` | 10.4.2 | MIT | Node >=18. |
| `@testing-library/jest-dom` | 7.0.1 | MIT | Vitest peer >=0.32; Node >=22. |
| `@testing-library/user-event` | 14.6.7 | MIT | DOM peer >=7.21.4; Node >=12. |
| Rust `rusqlite` | 0.40.2 | MIT | Use `features = ["bundled", "backup"]` to build bundled SQLite and expose backup API; both feature names are present. crates.io omits an explicit `rust_version` for this release, so its MSRV is unverified from registry metadata. |
| Rust `ts-rs` | 12.0.1 | MIT | MSRV 1.78; optional feature integrations (for example chrono/uuid) must be enabled when those types are exported. |

The npm versions and license / peer / engine metadata come from each package's live registry record; e.g. [Tauri CLI](https://www.npmjs.com/package/@tauri-apps/cli), [Vite](https://www.npmjs.com/package/vite), [Vitest](https://www.npmjs.com/package/vitest), [PDF.js](https://www.npmjs.com/package/pdfjs-dist). Crate metadata: [tauri](https://crates.io/crates/tauri), [tauri-build](https://crates.io/crates/tauri-build), [rusqlite](https://crates.io/crates/rusqlite), [ts-rs](https://crates.io/crates/ts-rs). Rusqlite's `bundled` feature uses the SQLite amalgamation; SQLite itself is [public domain](https://sqlite.org/copyright.html), while rusqlite remains MIT-licensed.

Current Node 24.14.1 satisfies Vite 8, Vitest 5 and PDF.js 6.3.289. However the latest `jsdom` registry release (30.1.1) declares Node `^22.22.2 || ^24.15.0 || >=26`; therefore it is currently incompatible with this workstation's Node 24.14.1. For Vitest DOM tests, either choose a jsdom version whose engine range includes Node 24.14.1 or use another supported DOM environment such as `happy-dom`, after checking its current registry engines and Vitest compatibility. Do not suppress the engine check as a substitute for choosing a compatible version.

## Tauri Windows configuration schema

The official v2 schema / reference locations are `bundle.windows.allowDowngrades`, `bundle.windows.webviewInstallMode`, and `bundle.windows.nsis.installMode`. These are separate objects/keys. Example JSON shape:

```json
{
  "bundle": {
    "windows": {
      "allowDowngrades": false,
      "webviewInstallMode": {
        "type": "offlineInstaller",
        "silent": true
      },
      "nsis": {
        "installMode": "currentUser"
      }
    }
  }
}
```

`allowDowngrades` is a sibling of `nsis` on `bundle.windows` and currently defaults to `true`; set it explicitly to express release policy. `installMode: "currentUser"` belongs inside `bundle.windows.nsis` and is also the documented default. It writes app installer metadata under HKCU and avoids requiring administrator rights by default. `webviewInstallMode` is a sibling of `nsis`. `offlineInstaller` is a valid mode that embeds and runs Microsoft's WebView2 offline installer silently; the official docs estimate about 127 MB added and say it does not require internet. It is distinct from `fixedRuntime`, which bundles an extracted runtime directory (about 180 MB) and uses it at runtime. Check official documentation before changing those schema keys: [Tauri config reference](https://v2.tauri.app/reference/config/) and [Windows installer](https://v2.tauri.app/distribute/windows-installer/).

Tauri CSP belongs under `app.security.csp`; permissions are declared in capability files and selected by `app.security.capabilities`. A capability's `permissions` array lists API/plugin permission identifiers; these are not a top-level `permissions` field. Tauri's runtime authority only allows IPC for windows/webviews matched by a capability, and permissions/scopes constrain what those APIs may do. Keep the capability set as small as the app's real UI needs. See [CSP](https://v2.tauri.app/security/csp/), [capabilities](https://v2.tauri.app/security/capabilities/), and [runtime authority](https://v2.tauri.app/security/runtime-authority/).

## PDF.js worker guidance for a packaged desktop app

Bundle the PDF.js worker with the frontend so it works offline; do not use a CDN worker. PDF.js requires its worker to be separately addressable/configured (`GlobalWorkerOptions.workerSrc` or `workerPort`). For Vite, make the worker a build asset using a module-relative `new URL("pdfjs-dist/build/pdf.worker.mjs", import.meta.url)` (or an equivalent Vite URL import) and confirm the production bundle emits and loads it under Tauri's app origin. Keep `app.security.csp` restrictive and allow the local worker origin in `worker-src` only as needed; avoid `blob:` unless the chosen worker bundling strategy actually uses blob URLs. PDF.js recent builds also ship decoder/WASM assets; if those are used by the selected PDF.js build, bundle them locally and set `wasmUrl` to that local directory rather than a remote host. Validate PDF rendering in both development and the packaged WebView2 app because dev-server URLs differ from packaged assets. Sources: [PDF.js setup guidance](https://github.com/mozilla/pdf.js/wiki/Setup-pdf.js-in-a-website), [official PDF.js webpack entry](https://github.com/mozilla/pdf.js/blob/master/external/dist/webpack.mjs), [PDF.js getting started](https://mozilla.github.io/pdf.js/getting_started/), and [Tauri CSP](https://v2.tauri.app/security/csp/).

## Scope and evidence limits

Versions are a point-in-time metadata check, not a security review, license compatibility opinion, SBOM, or tested dependency installation. Registry `license` fields are package-level declarations; verify the full transitive notice set before redistribution. No package was installed and no product files were changed as part of this research.
