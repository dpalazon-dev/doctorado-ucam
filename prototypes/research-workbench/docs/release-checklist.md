# Checklist de release

## Piloto 0.0.1

- [ ] Build desde un commit limpio, con `x86_64-pc-windows-msvc` explícito.
- [ ] `manifest.json` identifica commit, locks, herramientas, comando, ruta de origen del setup y SHA-256 del artefacto congelado.
- [ ] El ejecutable `research-workbench.exe` del target es x64; registrar su nombre, arquitectura y SHA-256. Leer y registrar por separado la máquina PE real del stub NSIS (se permite `i386` o `x64`) y el SHA-256 del setup congelado.
- [ ] Revisar el `.nsi` generado: instalación current-user, icono, menú Inicio, entrada de Aplicaciones instaladas y desinstalador; descartar eliminación recursiva de datos de usuario o rutas ajenas.
- [ ] Confirmar que el instalador contiene el payload de WebView2 offline y los recursos PDF.js servidos desde `dist` (worker, cmaps, fuentes estándar, ICC y WASM requeridos).
- [ ] En Windows 11 x64 limpio, sin Node/npm/Rust/Cargo/Git/Codex/WebView2 y sin red, instalar y abrir desde Inicio. Registrar si existe checkout de fuente y el directorio de trabajo usado.
- [ ] Importar un PDF sintético; mover el original; leer la página 2; cerrar y confirmar fin del proceso; reabrir y comprobar posición; archivar y restaurar.
- [ ] Abrir una segunda instancia y comprobar el aviso/estado `Busy` mientras la primera mantiene el escritor.
- [ ] Actualización desde una versión previa: solo marcar si existe migración real y fixture de origen. Reinstalar 0.0.1 no prueba migración.
- [ ] Desinstalar y verificar que se retiran aplicación/accesos y se conservan biblioteca y backups; reinstalar y reabrir la misma biblioteca.
- [ ] Registrar cada paso como comprobado, fallido o pendiente, con evidencia, fixture y entorno. Si no hay VM limpia/offline, el resultado es candidato con instalación pendiente.

## Límites de este piloto

- No hay firma Authenticode ni actualizador integrado.
- Migración, rollback inducido, downgrade de esquema futuro y recuperación tras fallo se mantienen pendientes salvo prueba independiente sobre el artefacto exacto.
- El piloto no acredita funciones previstas para v0.1/P3/P4, IA, OCR, merge ni borrado permanente.
