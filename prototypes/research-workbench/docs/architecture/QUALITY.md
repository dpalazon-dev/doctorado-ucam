# Research Workbench — calidad y verificación

Estado: plan de calidad previo a implementación. Los umbrales de rendimiento de abajo son propuestas por medir, no resultados ni criterios aceptados hasta acordar hardware y obtener evidencia. Las pruebas citadas son planificadas, no ejecutadas.

## Gates de evidencia

- **Piloto 0.0.1 instalado:** Windows 11 x64, NSIS real, máquina limpia/offline, WebView2 desde el paquete, recorrido de biblioteca/lector, continuidad y actualización/uninstall/reinstall. Un build sin bundle, tests de navegador, componentes mock o abrir desde el checkout no demuestran este gate.
- **v0.1:** flujo PRE/P1/P2, conocimiento global y procedencia, search, export, backup/restore y switch library verificados; ningún estado P2 implica P3/P4.
- Registrar cada evidencia como `automated`, `manual`, `simulated` o `pending`, con versión/commit, fixture, entorno, pasos y resultado. Ausencia de evidencia significa pendiente, no aceptado.

## NFR propuestos

Equipo de referencia propuesto: Windows 11 x64, 4 núcleos, 16 GB RAM y SSD local; registrar modelo y configuración reales. Tras calentamiento, medir al menos 30 repeticiones para latencias, mediana y p95; reportar frío/caliente por separado y usar build instalada/release. Objetivos iniciales de diseño, no medidos:

| Métrica | Presupuesto propuesto | Fixture/método | Estado |
|---|---:|---|---|
| Inicio caliente a Home | p95 ≤ 3 s | DB fixture 1k papers, app release | No medido |
| Inicio frío | p95 ≤ 8 s, excluye primera instalación | VM limpia sin red; runtime incluido | No medido |
| Abrir PDF a primera página | p95 ≤ 2 s | PDF local de 20 páginas/10 MB | No medido |
| Guardado pequeño confirmado | p95 ≤ 500 ms | metadata y edición con 10k items | No medido |
| Búsqueda básica | p95 ≤ 500 ms a 20 resultados | 1k papers + 10k items y filtros | No medido |
| Importación | ≥20 MB/s de copia secuencial | archivo fixture 100 MB en SSD, hash verificado | No medido |
| Restauración | ≤2x creación del backup mismo corpus | 1k papers, 10k items, 500 MB PDF | No medido |
| Memoria estable | p95 <700 MB | 30 minutos, PDF de 100 páginas; abrir/cerrar 20 | No medido |
| Instalador | medir y registrar tamaño | NSIS con runtime WebView2 offline | No medido |

Medir también durabilidad y crecimiento de memoria; ningún límite justifica debilitar transacciones, trazabilidad o recuperación. Si umbral falla, registrar hardware/corpus y decidir con evidencia antes de optimizar. No afirmar soporte máximo basándose en fixture.

## Seguridad, privacidad e integridad

1. **Rutas y protocolo PDF:** la UI selecciona con diálogo/token o UUID de documento registrado, nunca lee un path arbitrario. Backend resuelve ruta canónica bajo biblioteca y rechaza traversal, IDs desconocidos, symlink/junction de escape, recursos no registrados y archivos inesperados. Verificar CSP/origen local. Pruebas `protocol_rejects_traversal`, `protocol_rejects_symlink_escape`, `unknown_document_not_found`, `unregistered_path_never_served`.
2. **IPC y base:** validar DTO en Rust aunque TS valide; enums, tamaños, UUID, revisions, nullability y estados. SQL parametrizado. No exponer consultas, paths arbitrarios ni capacidades de filesystem general. Pruebas de contrato Rust/TS para camelCase, enums, errores, fechas e ids; `query_is_parameterized`, `stale_revision_rejected`.
3. **Contenido no confiable:** títulos no se convierten en paths; texto se escapa al presentar; PDFs y ontologías son datos. v0.1 `body` plain_text; no añadir editor enriquecido sin diseño/sanitización. Sin CDN/worker remoto. Enlaces externos requieren acción explícita.
4. **Privacidad local:** sin telemetría de papers/contenido, login, API remota o envío automático. Logs rotados omiten cuerpos, fragmentos, paths originales innecesarios y secretos. Export compartible omite rutas privadas por defecto. No afirmar cifrado de SQLite/backups.
5. **Concurrencia/recuperación:** un escritor por biblioteca, lock liberado al morir proceso; import staging/reconciliación no borra archivos de dueño desconocido; escritura de objetos y asociaciones es atómica. Bajo WAL, backup usa API snapshot consistente y coordina cambios a archivos.
6. **Backup y restauración:** staging/nueva raíz, verificar hashes, SQLite/FKs y recursos antes de switch. Fallo mantiene biblioteca activa. Migración con copia previa, checksums, bloqueo y protección contra downgrade/schema futuro.
7. **Mantenimiento:** archive/restore no cascada sobre conocimiento global; pruebas `archive_paper_no_cascade`, `archive_restore_preserves_links`. Fusión y borrado permanente no son funciones v0.1.
8. **Instalador:** NSIS por usuario, WebView2 offline, scripts auditados, datos fuera de carpeta instalable, checksum. Sin promesa de firma Authenticode en piloto.

Pruebas de regresión adicionales: `import_retry_is_idempotent`, `import_token_changed_payload_is_rejected`, `duplicate_doi_normalized`, `malicious_title_cannot_escape_export_root`, `plain_text_only_v01`, `logs_exclude_body_and_source_path`, `archive_restore_preserves_links`, `archive_paper_no_cascade`, `trash_merge_hard_delete_not_exposed_v01`. Una prueba de red offline cubre recorridos principales y captura solicitudes; su alcance limitado se documenta, no se presenta como prueba universal de ausencia de comunicaciones.

## Pruebas planificadas

### Dominio, contratos y UI

- Unitarias sin UI: unknown justificado; gate incompleto y gate revalidado; definiciones versionadas; edición posterior y fases marcadas `NEEDS_REVIEW` preservando datos; tipos/origen/procedencia; restricciones de relaciones; archivo/restauración; filtros FTS; versión/manifest export.
- Contract tests adaptador Tauri contra comandos reales cuando existan; mocks se etiquetan como simulados. Errores conservan código estable y mensaje español sin path privado.
- UI por versión: biblioteca vacía/import/duplicado/error; posición/reapertura; PRE/P1 con bloqueo, unknown y elección; P2 captura con/sin selección, concepto compartido y procedencia pendiente; búsqueda, export y restauración. Revisar estados carga/vacío/sin resultados/guardando/guardado/error/conflicto.
- Accesibilidad: teclado completo, foco visible/inicial/retorno, labels, errores asociados, anuncios de estado, contraste y semántica no basada solo en color. DPI 100/150/200%, ventana redimensionada y escalas diferentes.

### SQLite/filesystem y fallos

Cada test usa directorio temporal aislado y datos sintéticos; jamás biblioteca personal.

- Import válido, mover original, DOI/hash duplicado, Unicode, inválido, permisos/espacio, interrupción tras staging y promoción; recuperar sin registrar éxito falso.
- Integridad de FK, rollback de paper-item-relation-provenance; escritura fuera de orden; lock/segunda instancia; optimistic revision.
- Migración pre/durante/post, checksum errado, disco lleno, WAL, esquema futuro; no escribir en migración parcial.
- Backup con todos los recursos, SQLite corrupta, FK/hash roto, recurso faltante, espacio insuficiente, restore staging, fallo antes del switch y recuperación a biblioteca previa.
- FTS interrumpido entre mutación e índice; reconstrucción coincide con una consulta de referencia sobre fuente canónica y excluye archivados/papelera por defecto.
- Export conteos/referencias, JSONL parseable por línea, hashes, binarios opcionales, path privado omitido y destinos maliciosos/no escribibles.

Usar fault injection en filesystem/DB donde sea posible. `Guardado`, `Exportado`, `Restaurado` y ubicación activa solo tras commit/verificación. Tests de interrupción deben comprobar estado en una conexión/proceso nuevo.

### Corpus de rendimiento moderado

Fixture determinista sin contenido privado: **1.000 papers**, 2.000 autores, **10.000 knowledge items**, 15.000 relations, 12.000 provenances, 100.000 asociaciones y **500 MB de PDFs sintéticos**. Registrar seed, tipos/tamaños, filtros y hardware. Medir por separado metadata/FTS y binarios. Corpus evalúa calidad, no limita tamaño soportado.

### QA de producto instalado

VM/máquina Windows 11 x64 sin Node/npm/Rust/Cargo/Git/Codex y sin WebView2; nunca retirar runtimes del ordenador de trabajo. Registrar OS/arquitectura, installer/version/checksum, WebView2, red, cwd y si existe source checkout. Instalar offline, iniciar desde menú, usar con código ausente/cwd distinto, recorrer import→lectura→página→cierre→reapertura→archivo/restauración sin red, comprobar segunda apertura, proceso finalizado al cerrar, actualizar sobre fixture, inyectar fallo migración/rechazar downgrade, desinstalar conservando datos y reinstalar reabriendo misma biblioteca. Un perfil nuevo en máquina de desarrollo no prueba ausencia de herramientas.

## Fitness de arquitectura

- **Monolito modular hexagonal:** módulos de dominio (`library`, `workflow`, `knowledge`, `relations`, `provenance`, `search`, `export`, `backup`) dependen de puertos/interfaces; adaptadores SQLite/filesystem/Tauri quedan fuera del dominio. El dominio no importa React, Tauri, SQLite ni tipos UI.
- **Dirección de dependencias:** `app/composition → adapters → application/domain`; UI invoca casos de uso por adaptador typed/versioned IPC; ningún componente importa repositorios ni llama SQL. Dominio no contiene `invoke` ni paths del SO.
- **Canon y derivados:** SQLite + archivos administrados son autoridad; FTS, UI projections, JSONL/Markdown son reconstruibles/salidas. IDs UUID se mantienen estables entre adaptadores/export/recovery.
- **Transacciones:** mutaciones multiobjeto exponen casos de uso atómicos; no repartir gates, merge, restore o switch library como secuencia de decisiones React.
- **Verificación:** import boundaries/lints y contract tests verifican dependencias; prueba de composición crea adaptador SQLite temporal y evalúa el mismo dominio que IPC; no hay una segunda regla de negocio dentro del frontend.

## Gates de aceptación

- **Pilot installed:** instalador real probado limpio/offline, runtime incluido, recorrido Library/Reader, continuidad, actualización y uninstall/reinstall. No implica PRE/P1/P2.
- **v0.1:** PRE→P1→P2, conceptos/procedencia, gates e invalidación, búsqueda, export, backup/restore y cambio de biblioteca; cerrar y reabrir conserva estado. No implica P3/P4.
- Bloquean aceptación: pérdida de cambio confirmado, UUID/enlaces rotos, acceso a path fuera de raíz, migración parcial escribible, restore sin validación, export que comparte paths por defecto, o indicador de guardado antes del commit.
- NFR solo se marca alcanzado cuando se mida en hardware/corpus anotados. QA manual/nativa no se sustituye por tests web cuando estos no ejercitan el producto instalado.

