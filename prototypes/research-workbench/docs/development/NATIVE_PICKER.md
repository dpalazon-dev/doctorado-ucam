# T02 — selección nativa de PDF: verificación técnica

Verificado el 1 de octubre de 2026. Decisión adoptada por el coordinador para T02; aún no se ha añadido ni probado esta dependencia.

La [guía oficial Dialog de Tauri](https://v2.tauri.app/plugin/dialog/) permite utilizar el selector desde Rust sin instalar el paquete JavaScript. La operación `pick_file` recibe un callback y evita la variante bloqueante. Las rutas seleccionadas quedan en backend; la interfaz del proyecto devolverá únicamente el token y metadatos del contrato. La guía también describe permisos genéricos del plugin, que no deben concederse al frontend de esta aplicación.

El [registro de la crate](https://crates.io/crates/tauri-plugin-dialog/2.8.1) y su API devolvieron:
- tauri-plugin-dialog 2.8.1; rust_version 1.90; Apache-2.0 OR MIT.
- Dependencia tauri ^2.12, compatible por restricción de versión con tauri=2.12.1 del scaffold y Rust1.99.0. La compilación de T02 deberá demostrar compatibilidad real.
- Dependencias relevantes: rfd ^0.16 y tauri-plugin-fs ^2.6.0; dependencia transitoria no implica permiso frontend.
- Checksum crate: daf9a5c92e39bdd84f22f6be9230e39d0c4e140345f71ffe6058dca4f71127cf.
- Metadatos fuente: https://crates.io/api/v1/crates/tauri-plugin-dialog y https://crates.io/api/v1/crates/tauri-plugin-dialog/2.8.1/dependencies.

La [referencia Rust](https://docs.rs/tauri-plugin-dialog/2.8.1/tauri_plugin_dialog/) confirma DialogExt y FileDialogBuilder. El enlace detallado a FileDialogBuilder no fue accesible mediante web en esta consulta; verificar firma exacta en fuente descargada al implementar, sin inventarla.

Decisión de implementación: pin Cargo =2.8.1 y registro backend del plugin, sin paquete npm ni `dialog:default`/`dialog:allow-open`. Callback bridged a oneshot y puerto de selección, filtro PDF sin tratar extensión como validación del archivo. Mantener los63 comandos propios del contrato y probar que invocar directamente plugin:dialog|open desde main sigue denegado. Actualizar inventario de licencias/lockfile. No usar el instalador automático tauri add, que podría ampliar configuración sin revisar. Estos detalles son decisiones propias del proyecto; las fuentes anteriores sustentan disponibilidad/API, no acreditan que ya esté integrado.
