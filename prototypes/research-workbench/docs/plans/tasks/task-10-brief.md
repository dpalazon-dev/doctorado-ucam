# Tarea 10 — NSIS y verificación instalada (piloto y final)

**Owner:** Luna release/QA con Sol build/config/version/integración. **Ejecutar dos cortes:** 10-piloto tras T01–T03; 10-final tras T01–T09. **Riesgo:** alto instalación/protección datos. **Report:** `docs/reports/task-10-report.md` con ambas secciones, artifact SHA y entorno.

Leer IMPLEMENTATION, SPEC-001/SPECS alcance piloto/v0.1, QUALITY installed gate, ARCHITECTURE distribución. No estás solo, no editar tauri.conf/capabilities/manifiestos/lockfiles sin transferencia Sol. No modificar máquinas reales/biblioteca personal ni retirar WebView2 de equipo trabajo; usar VM/entorno prueba autorizado. No publicación/distribución a terceros.

**Posee:** `scripts/build-release.ps1`; `src-tauri/installer/research-workbench.nsh` si personalización realmente necesaria; `src-tauri/tests/installed_lifecycle.rs`; `docs/{INSTALLATION.md,release-checklist.md}`; `docs/releases/{0.0.1.md,0.1.0.md}`; `docs/verification/{installed-pilot.md,installed-v01.md}`. Sol firma final notas/versiones y genera artefactos bajo `dist-release/<version>/`; binarios no se commitean sin instrucción.

**Consume:** config NSIS x64 currentUser/WebView2 offline/identity T01, app y tests piloto T02/T03 o final T09, lockfiles congelados y baseline SHA. **Produce:** `Research-Workbench_<version>_x64-setup.exe`, SHA-256/manifest/version/sourceSHA, guía instalar/actualizar/desinstalar/conservar datos y evidencia QA sobre ese artefacto exacto.

- [ ] T10-piloto: checks verdes/revisión → build `npm run tauri:build -- --bundles nsis`; verificar offlineInstaller WebView2 bytes incluidos, resources PDFjs/worker/fonts, icon/Inicio/escritorio opcional/uninstaller/currentUser/release sin consola. T10-final repite con version0.1.0 y todos journeys, no hereda evidencia sin repetir sobre artifact final.
- [ ] Guardar manifest SHA256/version/sourceSHA/toolchains/build commands/lockfiles, scripts falla por exit code y no etiqueta build fallida como release. Sol realiza build sobre árbol inmóvil; script no descarga o instala SDK global.
- [ ] VM Windows11x64 limpia sin Node/npm/Rust/Cargo/Git/Codex/WebView2 y offline; registrar OS/hardware/runtime/artefacto/red/source ausente/cwd. Instalar abre desde Inicio/escritorio opcional sin terminal/devserver/source, entrega WebView2 offline y entrada Apps instaladas. Nuevo perfil en dev machine no demuestra clean-tools gate.
- [ ] Piloto recorrido import sintético→mover original→PDF/page2→close/process ended→reopen→archive→restore, segunda apertura enfoca/informa Busy un escritor; local assets sin red. Final además PRE/P1/P2/shared concept/provenance/relations/FTS/export/backup/restore/switch y reopen UUID/hash/state/cola P3 idénticos.
- [ ] Upgrade sobre fixture versión anterior con migración real; backup pre-migration íntegro y checks ID/hash/position/fase. Fault-injection mid-migration queda rollback/readOnly safe; downgrade future schema rechazo sin modificar. Actualizar versión compatible/reinstall abre investigación anterior.
- [ ] Uninstall retira binarios/accesos/registro y conserva biblioteca/backups. Reinstall compatible confirma datos. Examinar NSIS para ninguna ruta ajena o RecursiveRemove datos. Tests siempre synthetic temporal, no borrar biblioteca personal.
- [ ] Revisión config/script NSIS por Sol, resolver findings y repetir afectados. Clasificar cada test comprobado/fallido/pendiente con evidencia y condiciones; si VM limpia/offline falta, marcar instalado pendiente y entregar candidato build con límite honesto, no declarar piloto/v0.1 aceptado.
- [ ] Documentar alcance 0.0.1 vs0.1.0 y fuera scope P3/P4/IA/OCR/merge/delete, ausencia firma digital si aplica; checklist final y checksum entregable local. Sol actualiza STATUS y decide entrega tras gates, sin remoto/pub.

**Aceptación:** setup.exe real identificado SHA, instalación limpia/offline/runtime incluido y uso sin herramientas/source/cwd, upgrade/fail-safe/uninstall/reinstall conservan IDs/PDFs/backups. Build, exe nativo y mocks no reemplazan QA instalado.

## Pendientes trasladados del piloto inspeccionado

- [ ] T10-final excluye `generate_contracts.exe` del instalador. Es un helper de desarrollo encontrado en ambos pilotos; la inspección lo clasificó como observación menor, no como funcionalidad para el usuario. Sol debe aprobar la configuración concreta de empaquetado antes de cambiarla.
- [ ] Distinguir en el manifest o anexo el ejecutable del target del ejecutable extraído del NSIS. En el piloto Tauri cambió exactamente tres bytes del marcador `UNK` a `NSS`; el hash del payload es la referencia para una futura instalación. Repetir la comparación sobre el artefacto final, sin asumir los mismos offsets.
- [ ] Repetir las mediciones del lector nativo: proporción CSS/raster, escala en ambos ejes, desplazamiento hasta los extremos y página/zoom tras reiniciar el proceso. El piloto verificó reabrir el lector dentro de la misma app; eso no satisface el reinicio completo.

Evidencia de partida: `docs/reviews/task-03/FIX5-NATIVE.md` y `docs/reviews/task-10/pilot-corrected-artifact-review.md`. Los JSON originales y la inspección del setup corregido se conservan como artefactos históricos; no se modifican para aparentar una nueva verificación.
