# T02 — native PDF selection: technical verification

Verified on 1 October 2026. Decision adopted by the coordinator for T02; this dependency has not yet been added or tested.

The [official Tauri Dialog guide](https://v2.tauri.app/plugin/dialog/) allows using the picker from Rust without installing the JavaScript package. The `pick_file` operation receives a callback and avoids the blocking variant. Selected paths remain in the backend; the project interface will return only the token and contract metadata. The guide also describes generic plugin permissions, which must not be granted to this application's frontend.

The [crate registry](https://crates.io/crates/tauri-plugin-dialog/2.8.1) and its API returned:
- tauri-plugin-dialog 2.8.1; rust_version 1.90; Apache-2.0 OR MIT.
- Dependency tauri ^2.12, compatible by version constraint with scaffold tauri=2.12.1 and Rust1.99.0. The T02 build must demonstrate actual compatibility.
- Relevant dependencies: rfd ^0.16 and tauri-plugin-fs ^2.6.0; a transitive dependency does not imply frontend permission.
- Crate checksum: daf9a5c92e39bdd84f22f6be9230e39d0c4e140345f71ffe6058dca4f71127cf.
- Source metadata: https://crates.io/api/v1/crates/tauri-plugin-dialog and https://crates.io/api/v1/crates/tauri-plugin-dialog/2.8.1/dependencies.

The [Rust reference](https://docs.rs/tauri-plugin-dialog/2.8.1/tauri_plugin_dialog/) confirms DialogExt and FileDialogBuilder. The detailed FileDialogBuilder link was inaccessible through the web during this inquiry; verify the exact signature in downloaded source when implementing, without inventing it.

Implementation decision: pin Cargo =2.8.1 and register the plugin in the backend, without an npm package or `dialog:default`/`dialog:allow-open`. Bridge the callback to a oneshot and the selection port; use a PDF filter without treating the extension as file validation. Keep the contract's 63 custom commands and test that invoking plugin:dialog|open directly from main remains denied. Update the license inventory and lockfile. Do not use the automatic tauri add installer, which could expand configuration without review. These details are project decisions; the sources above establish availability/API, not evidence that integration has occurred.
