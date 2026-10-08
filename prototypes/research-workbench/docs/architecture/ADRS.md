# ADRs — Research Workbench

**Documento de decisiones arquitectónicas para v0.1**  
**Fecha:** 1 de octubre de 2026  
**Estado de todos los ADR:** Accepted (baseline 2026-10-01). Adopción para ejecución registrada en INTENT.md por la instrucción del usuario de crear el entorno e implementar completamente por fases. La aceptación del diseño no acredita ejecución del producto.  
**Base:** `work/spec.md` y `outputs/Research_Workbench_Plan_Multiagente.md`.

Este registro define la arquitectura objetivo de v0.1; el plan divide su entrega en incrementos y su primer piloto solo cubre biblioteca, importación y lectura. Las decisiones de fases y conocimiento se implementan en incrementos posteriores antes de declarar v0.1 completa. Frameworks y versiones se deben fijar al crear el repositorio: las referencias técnicas de este documento justifican capacidades, no acreditan instalaciones ni compatibilidad ya probada en este entorno.

## ADR-001 — Aplicación local para un investigador

**Estado:** Accepted (baseline 2026-10-01)  
**Contexto:** El producto sirve a un investigador que necesita leer, capturar y recuperar conocimiento sin depender de cuentas, red o servicios. Colaboración y sincronización cloud no forman parte del alcance inicial.

**Alternativas:** aplicación web con backend; aplicación local con almacenamiento remoto opcional desde el inicio; aplicación de escritorio local de un solo usuario.

**Decisión:** construir una aplicación de escritorio local, sin login ni servidor, cuyo flujo principal pueda completarse sin conexión. La biblioteca se guarda en una carpeta de datos del usuario y se separa del directorio del programa. Se admite una única instancia escritora por biblioteca.

**Consecuencias:** reduce operación y dependencias externas; facilita conservar documentos privados en el equipo. La sincronización, colaboración y acceso desde varios dispositivos requerirían contratos y resolución de conflictos nuevos.

**Riesgos:** pérdida del equipo o disco; una biblioteca copiada mientras se está usando puede quedar incompleta; una carpeta sincronizada en red podría no respetar los requisitos de SQLite. Backups verificados y restauración pertenecen al alcance v0.1.

**Validación:** completar importación, lectura, cierre y reapertura sin red; probar bloqueo de una segunda instancia escritora y restauración desde un backup verificado.

**Revisar si:** se aprueba colaboración, sincronización, uso multiusuario o almacenamiento de biblioteca en red.

## ADR-002 — Tauri 2, React/TypeScript, Rust y SQLite

**Estado:** Accepted (baseline 2026-10-01)  
**Contexto:** Se necesita una experiencia desktop convencional, un núcleo local con control de archivos y reglas, y almacenamiento relacional portable. El plan pide Tauri 2, React/TypeScript, Rust y SQLite; el runtime de desarrollo no está completo y no se deben dar por instaladas dependencias.

**Alternativas:** aplicación web local con servidor; Electron con backend TypeScript; aplicación desktop nativa integral en Rust; Tauri 2 con frontend React/TypeScript y backend Rust.

**Decisión:** adoptar Tauri 2 como shell e IPC, React con TypeScript y Vite para la interfaz, Rust para servicios y dominio, y SQLite mediante `rusqlite` como base local. La UI parte de shadcn/ui sobre Radix UI y Tailwind, con React Hook Form y Zod para formularios. PDF.js y su worker se empaquetan localmente para lectura. `rusqlite` se compila con SQLite incluido y la capacidad de backup; se prueba la creación real de una tabla FTS5 antes de contar con FTS5 en el artefacto. La selección de paquetes queda sometida a auditoría conjunta de versiones compatibles, licencias, APIs y lockfiles antes de implementar; no se fijan aquí parches no verificados. [shadcn/ui instalación manual](https://ui.shadcn.com/docs/installation/manual), [React Hook Form](https://www.react-hook-form.com/), [Zod](https://zod.dev/), [PDF.js Getting Started](https://mozilla.github.io/pdf.js/getting_started/?lang=en), [rusqlite bundled](https://docs.rs/crate/rusqlite/latest/source/README.md), [rusqlite features](https://docs.rs/crate/rusqlite/latest/features).

**Consecuencias:** separa UI y capacidades locales; permite distribuir una aplicación Windows compacta en comparación con un runtime Chromium incluido. PDF.js requiere empaquetar y servir también sus recursos/worker desde el bundle. `rusqlite` expone transacciones y tipos SQLite, pero la estrategia de conexiones y serialización sigue siendo responsabilidad de la aplicación. La documentación de Tauri describe comandos IPC; PDF.js separa API de visualización y worker; rusqlite documenta transacciones explícitas. [Tauri IPC](https://v2.tauri.app/concept/inter-process-communication/), [PDF.js Getting Started](https://mozilla.github.io/pdf.js/getting_started/?lang=en), [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/).

**Riesgos:** interoperabilidad y empaquetado de recursos deben probarse en el WebView2 de Windows; actualizaciones de las dependencias pueden cambiar APIs; Rust MSVC y Visual Studio C++ Build Tools son requisitos de build en Windows. La auditoría actual encontró VS 2022 y WebView2, pero Rust no está instalado en PATH ni en el directorio de Cargo observado.

**Validación:** crear una ventana instalada de Tauri, abrir un PDF de fixture con recursos locales, invocar un comando Rust y abrir una base SQLite temporal; comprobar la build Windows con lockfiles. No afirmar estas validaciones antes de ejecutarlas.

**Revisar si:** el instalador real no cumple tamaño, accesibilidad, rendimiento o soporte de los documentos; o si la compatibilidad de las versiones fijadas no puede mantenerse.

## ADR-003 — SQLite como autoridad estructurada única

**Estado:** Accepted (baseline 2026-10-01)  
**Contexto:** Las vistas por paper y por concepto deben representar las mismas entidades y conservar integridad al reabrir el programa. Markdown y JSONL sirven para intercambio y lectura humana, no como bases canónicas paralelas.

**Alternativas:** Markdown/JSONL como origen; SQLite más un almacén de grafo; SQLite canónica con archivos binarios administrados e índices/exportaciones reconstruibles.

**Decisión:** SQLite es la fuente de verdad para entidades, relaciones, procedencia, workflow, revisiones y metadatos de archivos. Los PDF y otros binarios se guardan en la biblioteca; índices FTS5 y proyecciones son derivados. Exportaciones Markdown/JSONL documentan datos y versiones y nunca se reimportan silenciosamente para sobrescribir la base.

**Consecuencias:** transacciones pueden mantener juntas las entidades y asociaciones relacionales; se puede reconstruir el índice de búsqueda. Foreign keys han de habilitarse y comprobarse por conexión, pues SQLite documenta que su enforcement debe activarse explícitamente. WAL puede mejorar concurrencia de lectores, pero permite un solo writer y no funciona sobre filesystem de red. [Foreign keys](https://www.sqlite.org/foreignkeys.html), [WAL](https://www.sqlite.org/wal.html), [transacciones](https://www.sqlite.org/lang_transaction.html).

**Riesgos:** la autoridad única hace esenciales las migraciones, backups y exportaciones verificables; escribir manualmente SQL arbitrario desde la UI rompería límites de dominio.

**Validación:** comprobar foreign_keys al abrir cada conexión; correr `foreign_key_check` y `integrity_check` en validación/restauración; ejecutar `CREATE VIRTUAL TABLE ... USING fts5(...)` durante la prueba de capacidad del binario y recrear el índice desde filas canónicas; exportar y reimportar en biblioteca temporal preservando UUID, enlaces y procedencia. La elección de la feature de Cargo se cierra tras auditar `rusqlite`/`libsqlite3-sys` fijados, su configuración de compilación FTS5 y el resultado de esta prueba; la palabra `bundled` por sí sola no se toma como comprobación funcional.

**Revisar si:** el volumen o patrones de consulta medidos superan SQLite de manera demostrable, o existe una exigencia real de backend multiusuario.

## ADR-004 — Conceptos globales, enlaces y procedencia localizable

**Estado:** Accepted (baseline 2026-10-01)  
**Contexto:** Capturar se hace en el contexto de un paper, pero un concepto puede reaparecer en muchas fuentes. Cada KnowledgeItem debe conservar su tipo, alcance, atribución y localizador; no se debe convertir una inferencia propia en una afirmación de la fuente.

**Alternativas:** duplicar todos los conceptos por paper; compartir conceptos y mezclar todas sus afirmaciones; mantener conceptos con UUID global y afirmaciones/evidencias asociadas a sus fuentes y contextos.

**Decisión:** Concept tiene UUID estable y puede ser reutilizado entre papers; nombre, alias y definición son editables. KnowledgeItem, Evidence, Question y Relation conservan sus enlaces de fuente y procedencia propios. Reutilizar o sugerir una coincidencia es explícito; no se fusionan sentidos por igualdad textual.

**Consecuencias:** la pantalla de paper conserva la secuencia de lectura y la vista de concepto agrega enlaces de los mismos registros. Las contradicciones permanecen como afirmaciones distintas con sus alcances y fuentes. Un localizador apunta a un documentId/version concreta; si no existe aún, se marca pendiente.

**Riesgos:** el mismo término puede tener sentidos diferentes; una vista agregada puede ocultar condiciones si resume sin mostrar alcance; eliminar una fuente puede invalidar localizadores, pero no debe borrar conocimiento compartido automáticamente.

**Validación:** enlazar dos papers de fixture al mismo Concept UUID; renombrar el concepto y verificar que no cambian enlaces; comprobar que cada afirmación abre su paper y localizador; confirmar que la vista conserva afirmaciones incompatibles sin sintetizarlas.

**Revisar si:** nuevos dominios requieren identidad de conceptos dependiente de contexto o una política de equivalencia semántica más rica.

## ADR-005 — Gates de workflow evaluados en dominio

**Estado:** Accepted (baseline 2026-10-01)  
**Contexto:** PRE, P1 y P2 guían el trabajo en v0.1; P3 y P4 se habilitan después. Completar una fase requiere procesar sus requisitos, pero no demuestra certeza o verdad científica.

**Alternativas:** permitir avance decidido solo por frontend; reglas fijas dentro de componentes de pantalla; definiciones de fase versionadas y validación repetida por servicios de dominio Rust.

**Decisión:** cada PhaseDefinition guarda versión, objetivo, preguntas, acciones permitidas y requerimientos. El servicio Rust evalúa y vuelve a evaluar el gate dentro de la operación de avance. Campos vacíos no cuentan como procesados; `unknown` con explicación puede satisfacer completitud cuando la definición lo permite. El resultado del gate indica completitud operativa, no validación epistémica.

**Consecuencias:** actualizar una plantilla no reescribe retrospectivamente el significado de fases ya completadas; la UI explica fallos con requisitos concretos y el backend impide saltos inconsistentes.

**Riesgos:** criterios demasiado rígidos fuerzan contenido artificial; criterios vagos hacen opaca la transición. Las reglas no deben exigir una conclusión correcta ni aprobar afirmaciones automáticamente.

**Validación:** probar respuestas vacías, desconocido justificado, respuesta capturada y salidas incompletas; invocar el comando de avance con un gate obsoleto y comprobar que el backend lo rechaza; verificar que la fase y su versión quedan registradas.

**Revisar si:** la evidencia de uso muestra requisitos insuficientes, diferencias importantes entre dominios o necesidad de evolucionar definiciones sin alterar trabajos históricos.

## ADR-006 — Documentos administrados con identidad estable

**Estado:** Accepted (baseline 2026-10-01)  
**Contexto:** Los PDFs deben seguir disponibles aunque el usuario mueva el archivo original o se retire el código fuente. Las rutas absolutas de origen pueden revelar información privada y no son una identidad documental.

**Alternativas:** abrir el PDF desde su ruta original; insertar todos los binarios en SQLite; copiarlo a una biblioteca administrada y mantener un registro relacional por UUID, hash y ruta relativa.

**Decisión:** importar mediante staging dentro de la biblioteca: registrar intención durable, copiar, calcular SHA-256 y verificar que se puede leer, promover a una ruta interna derivada de identificadores, y confirmar Paper/Document en SQLite. Un reconciliador al iniciar repara operaciones interrumpidas sin borrar archivos de propietario desconocido. El localizador se asocia a un `documentId` y versión, no solo a nombre o ruta original.

**Consecuencias:** los binarios quedan separados de los datos relacionales pero bajo una raíz portable; la DB conserva nombre original, fecha/hash y, cuando proceda, ruta de origen solo para diagnóstico. SQLite y filesystem no comparten commit atómico: la importación y eliminación requieren un protocolo explícito de intención y recuperación.

**Riesgos:** caída o falta de espacio entre pasos puede dejar staging o archivos sin registro; mover una biblioteca fuera de la aplicación puede romper rutas relativas; SHA-256 identifica cambios de bytes, no validez científica del contenido.

**Validación:** simular interrupción en cada límite de importación, reabrir y verificar reconciliación; mover el archivo original tras importar; comparar SHA-256 y ruta interna; no eliminar archivos ajenos durante recuperación.

**Revisar si:** se incorporan fuentes remotas, OCR, almacenamiento externo, versiones múltiples visibles o cambios que alteren el formato de localizadores.

## ADR-007 — Captura manual antes de asistencia automática

**Estado:** Accepted (baseline 2026-10-01)  
**Contexto:** La prueba inicial consiste en procesar manualmente un survey real de PRE a P2 y comprobar utilidad, recuperación y trazabilidad antes de añadir modelos. La inferencia de un sistema no equivale a evidencia.

**Alternativas:** incorporar extracción/LLM durante el primer flujo; aplazar todos los modelos hasta disponer de un proceso manual trazable; permitir sugerencias automatizadas sin control de aceptación.

**Decisión:** v0.1 funciona sin IA, agentes, RAG, embeddings ni conexión. La captura tipada y los localizadores deben ser suficientes. Una eventual asistencia futura produce propuestas separadas con modelo, fecha y procedencia; el investigador acepta o descarta explícitamente cada propuesta.

**Consecuencias:** se valida primero si el modelo de datos y el flujo son útiles sin costos, exposición de texto o dependencia de red. Una integración posterior podrá ser un adaptador opcional y no la autoridad para afirmar conocimiento.

**Riesgos:** la entrada manual puede resultar lenta; crear interfaces de integración antes de observar tareas reales añade abstracción sin evidencia.

**Validación:** completar un survey de prueba sin conexión, reabrirlo y exportar knowledge items con sus fuentes; confirmar ausencia de peticiones remotas en el recorrido núcleo.

**Revisar si:** observaciones de uso identifican una tarea repetida y delimitada que pueda asistirse sin eliminar control, procedencia y funcionamiento offline.

## ADR-008 — Ontología pequeña, tipada y ampliable con revisión

**Estado:** Accepted (baseline 2026-10-01)  
**Contexto:** Una clasificación permite capturar afirmaciones, evidencias, preguntas, interpretaciones y relaciones sin convertir la aplicación en un editor de ontologías. Los tipos y relaciones deben conservar restricciones de dominio.

**Alternativas:** notas libres sin tipos; vocabulario extenso editable por usuario; núcleo pequeño con enumeraciones estables y cambios de esquema versionados.

**Decisión:** incluir un conjunto corto de tipos de KnowledgeItem y Relation, conceptos globales, alias/dominio y enlaces de procedencia. La creación de relaciones valida extremos, dirección y compatibilidad tipada. Las ampliaciones se incorporan con migraciones/versiones; no se permite que texto libre sustituya un tipo del núcleo.

**Consecuencias:** formularios pueden guiar sin que el usuario conozca teoría ontológica; las exportaciones conservan un vocabulario documentado y las consultas agregadas son reproducibles.

**Riesgos:** una taxonomía inicial puede no cubrir un campo concreto; cambios de enumeraciones afectan import/export y migración; normalizar etiquetas no acredita equivalencia.

**Validación:** comprobar relaciones válidas e inválidas en servicio de dominio y constraints; round-trip de tipos con UUID y valores desconocidos compatibles según versión; revisar que crear una relación no permita autorrelación/ciclo donde el tipo lo prohíba.

**Revisar si:** el uso real produce categorías repetidas no expresables o vocabulario que requiera extensión por dominio con contrato versionado.

## ADR-009 — Monolito modular con límites hexagonales pragmáticos

**Estado:** Accepted (baseline 2026-10-01)  
**Contexto:** La aplicación es un ejecutable local con una base y un investigador. Separar reglas de UI y persistencia ayuda a proteger consistencia; construir microservicios, mediadores o un marco arquitectónico completo no aporta un límite operativo útil aquí.

**Alternativas:** módulos de pantalla con lógica y SQL mezclados; arquitectura hexagonal estricta con framework, contenedor DI y adaptadores genéricos para todo; monolito modular con casos de uso, dominio pequeño y repositorios donde aíslan persistencia o pruebas.

**Decisión:** usar un monolito Rust organizado por módulos de dominio/casos de uso. Comandos Tauri son adaptadores de entrada; servicios validan y orquestan; entidades/valores de dominio contienen invariantes; repositorios encapsulan consultas cuando mejoran claridad o permiten prueba/cambio. Los límites deben ser concretos y de bajo costo. En React, usar `useReducer` local para estado transitorio que lo requiera y derivar valores computables. La navegación inicial es un `ShellView` discriminado, sin React Router; los formularios estructurados usan React Hook Form/Zod; el body de texto plano usa `<textarea>` v1, sin TipTap. Zustand, TanStack Query y cache global se difieren hasta que una necesidad observada los justifique. No introducir framework global de state/routing, DI, event bus, ORM, CQRS ni DAO por defecto. React recomienda mantener el estado mínimo y derivar valores que puedan calcularse. [Thinking in React](https://react.dev/learn/thinking-in-react), [React Testing Library](https://testing-library.com/docs/react-testing-library/intro/), [Vitest](https://vitest.dev/guide/).

**Consecuencias:** pocos pasos para seguir una operación desde IPC a transacción; los módulos se prueban con SQLite temporal o adaptadores mínimos. La arquitectura no exige separar lecturas de escrituras en infraestructura: comandos/casos de uso pueden organizarse como consultas y operaciones de cambio dentro del mismo proceso y DB. React recomienda mantener una representación mínima del estado UI y calcular lo derivado. [Thinking in React](https://react.dev/learn/thinking-in-react).

**Riesgos:** módulos demasiado grandes pueden volver a acoplar reglas; interfaces añadidas sin sustituto real encarecen cambios; evitar patrones por principio también sería un error si aparece una necesidad concreta.

**Validación:** revisar que gates, cambios de fase, archivo, importación y fusiones no dependan de componentes React; que cada comando tenga un caso de uso identificable; comprobar que la UI no ejecuta SQL ni accede al disco general.

**Revisar si:** aparecen varios adaptadores reales, backends distintos, complejidad de transacción repetida o tamaño de equipo que requiera límites formales.

## ADR-010 — IPC Tauri tipado y versionado, sin API REST

**Estado:** Accepted (baseline 2026-10-01)  
**Contexto:** UI y backend se distribuyen juntos como una app local. No hay cliente remoto ni necesidad de servidor HTTP; los comandos deben limitar capacidades y devolver errores con estructura estable.

**Alternativas:** servidor local REST; acceso de frontend a archivos/SQL; comandos Tauri con DTOs explícitos y contrato de versión.

**Decisión:** utilizar comandos Tauri para solicitudes de UI a Rust con DTOs serde canónicos, UUID estables, fechas RFC3339 UTC, revisión esperada y resultados/errores tipados. Generar los tipos TypeScript desde los DTO Rust para evitar dos definiciones independientes; `ts-rs` es la herramienta seleccionada; se comprueba la representación de tipos/serde al fijar el toolchain, antes de escribir funcionalidades. Si esa comprobación revela incompatibilidad, se modifica este ADR antes de implementar. La API de frontend definitiva (`LibraryApi`, etc.) declara desde bootstrap las operaciones acordadas y encapsula `invoke`; no se dejan contratos ficticios como placeholders. Módulos de dominio no conocen Tauri. Los eventos se reservan para notificaciones unidireccionales de ciclo de vida/progreso, no para mutaciones que necesiten confirmación. Versionar el contrato exportable y cualquier compatibilidad de esquema. No desplegar REST ni aceptar rutas de archivos/SQL arbitrarias de la UI. [Tauri IPC](https://v2.tauri.app/concept/inter-process-communication/), [Runtime Authority](https://v2.tauri.app/security/runtime-authority/), [ts-rs](https://docs.rs/ts-rs/latest/ts_rs/).

**Consecuencias:** comandos acotados son fáciles de autorizar mediante capabilities/scopes y probar con adaptador. Tauri documenta que `invoke` serializa argumentos y resultado con protocolo tipo JSON-RPC y que Events son mensajes unidireccionales; la versión del producto aún debe definir sus propios DTOs y errores. [IPC Tauri](https://v2.tauri.app/concept/inter-process-communication/), [Runtime Authority](https://v2.tauri.app/security/runtime-authority/).

**Riesgos:** cambios simultáneos de DTO Rust y TypeScript pueden desincronizarlos; errores como cadenas sin código hacen frágil la UI; un comando demasiado general se convierte en API de capacidades excesiva.

**Validación:** prueba de serialización cruzada Rust/TypeScript; prueba de errores conocidos y desconocidos; probar rechazo de `expectedRevision` obsoleta; auditar capabilities con comandos permitidos mínimos.

**Revisar si:** se requiere cliente externo, actualización independiente del frontend/backend o comunicación de larga duración bidireccional.

## ADR-011 — Identidad desktop estable e instalación Windows offline

**Estado:** Accepted (baseline 2026-10-01)  
**Contexto:** La aceptación exige que un usuario abra la app desde Inicio sin Node, Rust, Git, Codex, terminal ni checkout y trabaje sin red. La instalación no debe poseer ni borrar el directorio de datos.

**Alternativas:** distribución como web app/servidor local; MSI o instalación portable sin identidad estable; instalador NSIS por usuario con identidad de bundle fija y runtime offline.

**Decisión:** fijar antes del primer artefacto distribuible un bundle identifier, nombre, versión, arquitectura Windows x64 e iconos estables. Producir NSIS per-user; separar binarios/accesos de la biblioteca persistente; empaquetar localmente PDF.js y configurar `offlineInstaller` para WebView2 cuando se mantenga el requisito de instalación desconectada. El desinstalador elimina el programa y accesos, preserva biblioteca y backups. El directorio de trabajo actual no define rutas de datos. La guía Tauri contempla NSIS y `offlineInstaller`, documentando el aumento aproximado de 127 MB y uso sin internet para instalar WebView2. [Instalador Windows Tauri](https://v2.tauri.app/distribute/windows-installer/), [configuración Tauri](https://v2.tauri.app/reference/config/).

**Consecuencias:** las actualizaciones pueden identificar la misma instalación y abrir la misma biblioteca; el tamaño de descarga sube de forma material por el runtime offline. Se requiere probar instalación, actualización, desinstalación y reinstalación en entorno Windows limpio. La presencia de WebView2 en el equipo de desarrollo no prueba independencia del runtime.

**Riesgos:** identidad cambiante puede crear instalaciones duplicadas; un script NSIS mal delimitado podría borrar datos; recursos externos o CDN romperían modo offline.

**Validación:** instalar en Windows 11 x64 limpio sin herramientas dev y sin WebView2/red; abrir desde Inicio, importar/leer/reanudar; actualizar preservando UUID y PDF; desinstalar y verificar que la biblioteca sigue disponible para reinstalación. Comprobar recursos empaquetados y ausencia de descargas en el flujo offline.

**Revisar si:** se cambia Windows como plataforma primaria, se distribuye vía Store, se decide usar runtime online, o cambia el propietario/ubicación de datos.

## ADR-012 — Un escritor coordinado, migraciones protegidas e intenciones de filesystem

**Estado:** Accepted (baseline 2026-10-01)  
**Contexto:** SQLite puede leer en paralelo pero serializa escritores; WAL no habilita varios escritores y no sirve en filesystem de red. Importar/mover un archivo y confirmar SQL no son una transacción única. Un fallo no debe mostrar «Guardado» ni dejar una biblioteca parcialmente actualizada.

**Alternativas:** escrituras concurrentes desde cada comando/ventana; una conexión compartida sin política; actor/worker local que serializa escrituras SQLite y coordina operaciones con archivos mediante intenciones durables, revisiones y recuperación.

**Decisión:** por biblioteca, permitir una instancia escritora y usar un actor/worker en hilo propio como único propietario de la conexión síncrona `rusqlite`. El IPC asíncrono espera ese worker; ninguna llamada SQLite bloqueante se ejecuta en el hilo de UI/runtime. Toda mutación se serializa; las lecturas usan la misma conexión y el mismo dueño. Se seleccionan WAL, synchronous=FULL, foreign_keys=ON y busy_timeout=5000 ms, con cola acotada a 64 trabajos; estas decisiones se verifican mediante pruebas de durabilidad, backup y cierre antes de entregar; no habilita varios escritores. No usar la biblioteca sobre unidad de red. Operaciones largas de filesystem copian/hashean en worker de fondo y coordinan su resultado con una intención durable y una unidad de trabajo SQL breve; no se retiene transacción durante copia de PDF, backup o espera de usuario. Cada actualización de entidad usa `expectedRevision` para rechazar ediciones obsoletas sin borrar el texto pendiente de la UI. Operaciones filesystem multi-paso (importar, restaurar, eliminación futura) registran intención/idempotency key y estado recuperable, ejecutan filesystem y SQL en orden definido, y reconcilian al iniciar sin borrar elementos de dueño desconocido. Migraciones versionadas/checksum se ejecutan en exclusión de escritura, con snapshot consistente de SQLite y documentos relevantes, verificación y rechazo de esquema futuro; fallo conserva la biblioteca anterior recuperable. El backup usa SQLite Online Backup API o equivalente consistente; no copia a ciegas la DB con WAL activo. [SQLite transactions](https://www.sqlite.org/lang_transaction.html), [WAL](https://www.sqlite.org/wal.html), [Online Backup API](https://www.sqlite.org/backup.html), [rusqlite Transaction](https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html), [rusqlite backup](https://docs.rs/rusqlite/latest/rusqlite/backup/).

**Consecuencias:** existe orden claro para guardar y señalizar confirmación; una operación se recupera tras cierre inesperado sin fingir atomicidad entre DB y directorios. Añade tablas de operaciones, revisiones, manifiestos y pruebas de fallo; los accesos UI esperan respuesta de commit para presentar Guardado.

**Riesgos:** el worker puede convertirse en cuello de botella si se hacen trabajos largos dentro de transacción; cierres forzados, discos llenos y permisos requieren escenarios de prueba. WAL, si se activa, crea archivos `-wal`/`-shm` que forman parte del estado mientras la base está abierta. El actor necesita cierre ordenado, cancelación y errores que alcancen a quien inició la solicitud; no debe transformarse en un bus genérico.

**Validación:** concurrencia de cambios con la misma revisión; cierre forzado tras cada paso de importación; disco lleno/fallo de backup/migración; restauración en carpeta nueva con hashes e integridad; comprobación de rechazo del downgrade incompatible; un solo proceso puede escribir. Comprobar que los ajustes WAL/FULL elegidos se aplican y que backup/recuperación pasan antes de aceptar la implementación.

**Alcance y secuencia de mantenimiento:** el documento de producto contempla fusionar conceptos y eliminación permanente con revisión de dependencias. El plan del primer piloto los aplaza, igual que conocimiento tipado, gates, export y backup de usuario. Por tanto, en el primer piloto se implementan archivo/restauración reversible, revisión esperada e intenciones de importación/migración; no se ofrece una acción de fusión ni borrado permanente. En el incremento de mantenimiento posterior, cualquier fusión o eliminación requiere pantalla de dependencias, destino/alcance explícito, backup recuperable, confirmación y registro de auditoría; no se ejecuta como efecto lateral de borrar un paper.

**Revisar si:** se autorizan varias instancias escritoras, carpetas sincronizadas, almacenamiento de red, operaciones masivas, borrado permanente o migraciones que cambien la forma de localizar documentos.

---

## ADR-013 — Puertos de Library e intención de importación durable

**Estado:** Accepted (ejecución 2026-10-01).
**Contexto:** el servicio de biblioteca debe coordinar selección, archivos y SQL conservando una sola transacción de confirmación. La revisión de T01 mostró que ocultar SQL detrás de una función no basta si la aplicación sigue importando el adaptador concreto.

**Decisión:** tres puertos específicos: selección PDF nativa, DocumentStore y LibraryPersistence. SQLite implementa persistencia sobre DbActor; helpers de aplicación reciben el repositorio transaccional y la Transaction de UoW explícitamente permitida. El servicio no importa el adaptador concreto. Composición en lib/desktop. No añadir un executor genérico ni contenedor DI.

La intención confirmada se conserva en metadata_json mediante un formato interno versionado, separando entrada original y metadata normalizada; cancelación usa FAILED con intención OperationCancelled y estado de cleanup. El detalle vinculante, pruebas y ownership están en [TASK02_PORTS](../plans/TASK02_PORTS.md). Recovery termina solo intenciones confirmadas inequívocas; conserva los casos pendientes/ambiguos y comunica recoveryRequired. No se añade una API implícita para reanudar drafts de sesiones anteriores.

El picker se implementará con tauri-plugin-dialog=2.8.1 desde Rust, sin paquete JavaScript ni permisos genéricos de diálogo/filesystem. Compatibilidad de versión y fuentes se verificaron en [NATIVE_PICKER](../development/NATIVE_PICKER.md); compilación y pruebas siguen pendientes. Los 63 comandos propios y CONTRACTS v1 permanecen iguales.

**Consecuencias:** wrappers de persistencia y JSON interno validado, a cambio de frontera comprobable y recuperación sin modificar 0001. Las copias sin confirmación que no puedan cancelarse inequívocamente permanecen como incidencias explícitas. T04 extiende la misma confirmación transaccional para inicializar Workflow, conservando un único COMMIT.

## ADR-014 — Probe estructural de PDF y documentos protegidos

**Estado:** Accepted (ejecución 2026-10-02).

**Decisión:** validar en backend mediante pdf=0.10.0 sin features por defecto, sobre staging administrado, input acotado a 500 MiB y una validación activa por biblioteca. Comprobar estructura y primera página sin decodificar contenido gráfico ni modificar bytes. Rechazar cifrado en v0.1; no añadir contraseñas ni descifrado persistido. Detalle, límites y fuentes en [PDF_VALIDATION](../development/PDF_VALIDATION.md). Ningún contrato IPC cambia.

**Consecuencias:** hasta 500 MiB de buffer más estructuras del parser; no equivale a una cota total de RAM ni a validar todas las páginas. Algunos PDF protegidos legibles con otros lectores se rechazan en el piloto. Se evita parser artesanal, mmap unsafe y cambio de NSIS por una API Windows que requiere identidad de paquete. La compatibilidad, fixtures y costes reales se verifican en T02, no se dan por probados por este ADR.

## ADR-015 — Apertura y entrega local de PDF al lector

**Estado:** Accepted (ejecución 2026-10-02).

**Decisión:** ReaderPersistence específico y acceso lector sobre DocumentStore existente; helpers transaccionales explícitos. openPaper registra actividad/contexto/receipt sin modificar revisión o updatedAt bibliográficos. Protocolo research asíncrono autorizado por UUID, ventana y origen; perfil200completo sin anunciar Range/streaming. PDF.js y recursos locales completos, useWasm=true con CSP wasm-unsafe-eval específica y sin unsafe-eval JavaScript. Detalles y fuentes en [TASK03_BOUNDARIES](../plans/TASK03_BOUNDARIES.md). [TASK03_PORTS](../plans/TASK03_PORTS.md) fija las firmas internas, el handle propietario retenido durante commit/respuesta y RequestRegistry único inyectado en Library/Reader. La fase contextual se obtiene del Paper; no se añade columna a app_session ni otra capa repository en modules.

**Consecuencias:** apertura no causa conflictos artificiales en borradores de metadatos. La lectura completa implica memoria proporcional al PDF y copias; se mide antes del piloto, sin reducir500MiB por conveniencia. WASM añade recursos y una directiva CSP limitada; exige verificar bundle/render offline. Cancelación del consumidor no implica aborto del job Rust. Firmas internas/shared ownership se congelan después de T02, antes de T03; no se adelanta implementación.

## ADR-016 — Archivos vinculados a handles y propiedad de sagas admitidas

**Estado:** Accepted (corrección T02, 2026-10-02).

**Decisión:** las operaciones administradas Windows conservan handles verificados y guardas de padres; eliminación sobre el objeto verificado y promoción atómica sin reemplazo. Autorizar windows-sys0.61.2 ya presente en el lockfile, encapsulado en un adaptador privado. Cada saga admitida posee sus permisos/exclusiones hasta terminar aunque desaparezca el caller; requestId se coordina antes de efectos filesystem. Recovery conserva incidencias/errores para Settings y nunca declara cleanup DONE sin prueba. Detalle, límites y referencias primarias en [MANAGED_FILES](../development/MANAGED_FILES.md). Sin nueva API pública ni cambio de migración.

**Consecuencias:** código Windows FFI pequeño que exige revisión de seguridad/Rust, más handles retenidos y posibles Busy ante acceso concurrente externo. Los trabajos admitidos pueden retrasar cierre y seguir hasta punto seguro aunque la UI descarte la respuesta. Un fallo global de recovery impide nuevas mutaciones hasta reconciliación satisfactoria; los pendientes aislados conservan operativos los papers ajenos. No se certifica tolerancia a cortes eléctricos por compilar o por probar reinicios de proceso.

**Precisión F10, 2026-10-02:** sharing no impide FILE_WRITE_ATTRIBUTES/reparse. Se adopta [WINDOWS_DIRECTORY_GUARDS](../development/WINDOWS_DIRECTORY_GUARDS.md): volumen local NTFS, adquisición NtCreateFile relativa y directorio no vacío mediante hijo retenido READ_DATA/no-delete, con revalidación posterior del mismo directorio. DB existente actúa como pin de lectura sin escrituras antes del diagnóstico de versión. DB nueva se crea atómicamente FILE_CREATE solo después de LibraryLock; colisión aborta sin abrir SQLite. `.rw-directory-pin` protege directorios administrados vacíos, se omite de export/backup y se regenera al restaurar. Features Windows mínimas adicionales documentadas; promoción Win32 por handle permanece. Costes: handles y pins persistentes adicionales, almacenamiento de biblioteca limitado a NTFS local y coordinación de liberación al cerrar/switch. Probe sintético aprobado y ejecutado avala la elección; la implementación Rust y sus regresiones aún requieren revisión.

## ADR-017 — Gates canónicos, aceptación explícita y clock de fase por Paper

**Estado:** Accepted (Sol, 2026-10-02).

**Contexto:** PRE no distinguía tipo unknown importado de revisión explícita; payloads/resoluciones P2 y composición de reglas no estaban fijados. Gate suficiente sin aceptación permitía touch hacia delante omitiendo NEW→ACTIVE/snapshot previo. Hashes históricos no demostraban acceso actual al PDF. Revisiones locales iguales podían aceptar expectativas de otro contexto activo.

**Decisión:** [WORKFLOW_GATES.md](WORKFLOW_GATES.md) cierra semántica sin cambiar DTOs/firmas contractVersion=1. [Definiciones v1](phase-definitions/) canónicas inmutables embebibles al compilar. Sexta salida PRE review_type confirma metadato con estructura existente; P1 caracteriza la fuente. UNKNOWN/NA explicado sin duplicar texto, P1 decisión ANSWERED estructurada y PENDING/null pendiente. P2 enlaza artefactos por salida (síntesis Insight), sin cuotas; candidata usa OR cerrado por key y una única justificación PhaseAnswer; proyección cambiada renueva answer.revision y phase clock.

Forward/completar exige cadena COMPLETED+snapshot aceptado+gate vigente y continue aceptado P1; consulta de datos iniciados no acepta implícitamente. PRE/P1/P2 proyectan documento vivo. Proof/handle fuera TX, referencia DB revalidada dentro; ABI tras T03, no lectura/hash 500MiB bajo DbActor. Clock fresco max(revision)+1 por Paper/phase afectada/contexto bajo UoW, replay antes CAS/no-op después; snapshot excluye clock/contexto/timestamps. NEW→ACTIVE sólo advance PRE; P1 light/archive conserva active P1/P2 previa. COMPLETED reservado conserva upgrade/read/Library archive/restore y bloquea transiciones incompatibles sin migración especial. Aceptación P2 real espera T07; capability faltante nunca simula artefactos completos.

**Consecuencias positivas:** confirmación humana observable, reglas/payloads estables, aprobación stale rechazada, historia preservada y una sola UoW/autoridad de candidato. **Costes/límites:** seis respuestas PRE; propagación explícita de clock a mutaciones compartidas; proof/handle requiere composición T03; edición de definiciones exige versión nueva. El acceso comprobado no demuestra detección de toda modificación externa ni tolerancia hardware. Esta decisión no acredita implementación/instalación.

**Alternativas descartadas:** tratar default unknown como confirmado; exigir texto redundante o cantidades científicas mínimas; AND global de candidatos y ausencia; hash UI adicional en wire; tabla/servicio de reloj; forward basado sólo en preview; comprobar sólo PDF histórico; leer/hashear archivos grandes en SQLite; normalizar lifecycle reservado en upgrade.

## ADR-018 — Composición Workflow sobre el piloto integrado

**Estado:** Accepted por Sol, 2026-10-03. **Ámbito:** T04, después de T10-piloto; no altera CONTRACTS v1.

**Contexto:** T03 aporta acceso documental con handle retenido, pero su error público no distingue archivo inaccesible de fallos internos. Workflow necesita representar disponibilidad en el gate sin ocultar errores de integridad; también debe recuperar receipts anteriores aunque el PDF haya desaparecido. El piloto aún no inicializa procesamiento ni ofrece entrada UI independiente del lector.

**Decisión:** [TASK04_PORTS](../plans/TASK04_PORTS.md) fija los puertos y la composición. Migración 0002 autocontenida con seed y backfill en la transacción existente; 0001 permanece inmutable. Helpers en application con repositorio prestado inicializan el import antes de su resultado durable e invalidan inputs efectivos en su misma UoW. Dominio puro y SQL exclusivamente en adaptadores.

La prueba documental usa una ABI interna tipada: handle disponible o causa de apertura OS conocida (ausente, acceso denegado, sharing violation). Guardas, fallos genéricos y panic siguen siendo errores. Reader conserva su contrato mediante un núcleo de apertura compartido. Replay temprano y repetido dentro de la TX; referencia revalidada también cuando el acceso falló; permisos y handle sobreviven al commit/rollback. No se sostiene una TX durante filesystem ni se lee el PDF completo para el gate.

La Biblioteca ofrece acceso a Procesamiento por paperId sin pasar por Reader. Consultar archivados o documentos inaccesibles no requiere restaurar ni aceptar fases. Se habilitan ocho comandos PRE/P1, con P2 y candidatos limitados por sus capacidades reales.

**Costes y alternativas:** seed histórico duplicado con pruebas de paridad; clasificación de apertura interna adicional y una lectura breve para replay. Se descartan callbacks de migración innecesarios, traducción indiscriminada de StorageUnavailable a PDF ausente, un segundo actor y entrada a formularios condicionada por openPaper. Los fallos de apertura no prueban guardas sobre un handle que no se adquirió. Revisión Rust/seguridad y regresiones de las guardas Windows serán obligatorias antes de integrar.

**Aclaración de conformidad, 2026-10-03:** el preflight T04b detectó que TASK04_PORTS resumía la invalidación conservando IN_PROGRESS también en fases posteriores, en contradicción con DOMAIN/SPEC-003. Se corrige el resumen: inputs directos COMPLETED pasan a NEEDS_REVIEW; toda fase posterior ya iniciada pasa a NEEDS_REVIEW aunque fuera IN_PROGRESS. Un input directo IN_PROGRESS se conserva sólo si no es también posterior de otra entrada modificada. NOT_STARTED, contenido y última aceptación histórica se preservan; clock único por fase/UoW. No cambia el contrato normativo de dominio ni wire; pruebas cubrirán ambos casos y la unión de entradas.

## ADR-019 — Fechas UTC canónicas y lectura de resultados Reader previos

**Estado:** Accepted por Sol, 2026-10-03. **Disparador:** QA nativa de T03, diagnóstico en [native-open-diagnosis](../reviews/task-03/native-open-diagnosis.md).

**Contexto:** DATA exige UTC RFC3339. Reader produjo fechas UTC válidas con `+00:00`; el validador de la interfaz sólo admitía `Z`. Una apertura confirmada en SQLite fue rechazada en el cliente. Corregir sólo el escritor deja inaccesibles Paper/Position y receipts previos con la representación anterior.

**Decisión:** los dos escritores Reader usan el formato canónico existente de milisegundos y sufijo `Z`. El decoder UTC de wire admite las dos representaciones UTC verificadas, `Z` y `+00:00`, manteniendo validación completa de fecha/hora. Rechaza fechas locales, offsets distintos de cero y `-00:00` (offset desconocido). Conserva el string recibido: no reescribe DB, receipts, precisión ni resultados históricos. Esto concreta la representación de los strings UTC existentes, sin nuevos campos, enums o versión IPC.

**Verificación:** resultados Reader reales frente a los schemas cliente; apertura, guardado/consulta y replay, incluido un fixture del receipt anterior. Deben seguir fallando fechas inválidas, offsets no UTC y valores no string. Se mantiene la validación estricta de las demás propiedades del envelope y DTO.

**Alternativas descartadas:** aceptar cualquier offset, ignorar errores de contrato, mutar receipts confirmados, borrar la biblioteca sintética o introducir una migración de datos por una representación equivalente de UTC. La compatibilidad de lectura no demuestra que el visor nativo funcione; ese recorrido debe repetirse sobre el código corregido.

## Referencias técnicas consultadas

Enlaces primarios citados junto a las decisiones que respaldan:

- Tauri: [IPC](https://v2.tauri.app/concept/inter-process-communication/), [Runtime Authority y capabilities](https://v2.tauri.app/security/runtime-authority/), [instalador Windows](https://v2.tauri.app/distribute/windows-installer/), [configuración](https://v2.tauri.app/reference/config/).
- SQLite: [transacciones](https://www.sqlite.org/lang_transaction.html), [WAL](https://www.sqlite.org/wal.html), [Online Backup API](https://www.sqlite.org/backup.html), [foreign keys](https://www.sqlite.org/foreignkeys.html).
- Rust/SQLite: [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/), [features](https://docs.rs/crate/rusqlite/latest/features), [Online Backup API en rusqlite](https://docs.rs/rusqlite/latest/rusqlite/backup/) y [Transaction](https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html).
- PDF.js: [Getting Started](https://mozilla.github.io/pdf.js/getting_started/?lang=en) y [examples](https://mozilla.github.io/pdf.js/examples/index.html).
- React: [Thinking in React](https://react.dev/learn/thinking-in-react).
- UI tests: [React Testing Library](https://testing-library.com/docs/react-testing-library/intro/); runner previsto en el plan: [Vitest](https://vitest.dev/guide/).
- Contratos: [ts-rs](https://docs.rs/ts-rs/latest/ts_rs/) es la herramienta seleccionada para generar bindings TS desde tipos Rust; se verifica la representación de enums, UUID, fecha, errores y atributos serde requeridos.

La disponibilidad actual de paquetes, versiones exactas, compatibilidad entre crates y configuración concreta de Tauri aún se debe revisar al inicializar el repositorio y probar en build instalada. Este ADR no afirma que se hayan instalado o ejecutado.
## ADR-020 — Guardar respuestas no inicia fases

**Estado:** Accepted por Sol, 2026-10-03, antes de implementar save en T04b. Contraste independiente con DOMAIN/WORKFLOW_GATES/CONTRACTS, sin regla explícita anterior sobre save en NOT_STARTED.

**Contexto:** touch/advance exigen cadena aceptada vigente para habilitar fases, pero no se había fijado si guardar podía escribir en una fase NOT_STARTED. Permitirlo sin iniciar produciría respuestas con una fase de revisión0 que la invalidación deja intacta; iniciarla desde save crearía otra vía de habilitación. Exigir que toda edición sea de la fase activa tampoco está en el contrato y dificultaría corregir fases previas ya iniciadas.

**Decisión:** después de lookup/replay durable y las guardas aplicables, una petición nueva savePhaseAnswer contra NOT_STARTED falla con GateBlocked, sin respuesta, clock, contexto, auditoría de éxito ni receipt nuevo. Save nunca inicia. Una fase iniciada puede editarse aunque no sea la activa, con CAS de respuesta y sin cambiar contexto. Cambio efectivo invalida fase completada y posteriores iniciadas; no-op conserva tokens. No se exige reconfirmar cadena antes de editar: la habilitación hacia delante y la aceptación siguen sujetas a R1. Archivados, COMPLETED reservado y capacidades ausentes conservan sus restricciones. Un replay confirmado sigue precediendo a esta guarda.

**Verificación:** save P1 NOT_STARTED no deja efectos durables; save de PRE/P1 iniciada fuera del contexto activo conserva activePhaseCode e invalida/clocks según D1; replay y stale no-op conservan precedencias. UI permite consultar fases sin iniciarlas y explica la necesidad de activación explícita antes de guardar en NOT_STARTED. Tests nativos/instalación no se infieren de esta decisión.

**Coste y alternativas:** no permite prellenar respuestas persistentes de una fase aún no habilitada. Se descartan inicio implícito, datos persistidos bajo fase NOT_STARTED/revision0 y restricción adicional a sólo la fase activa. No modifica wire, DTOs, schema ni definiciones canónicas. WORKFLOW_GATES y CONTRACTS incorporan la precisión; DOMAIN/SPEC mantienen sus reglas y la referencian.

## ADR-021 — Casos Workflow transaccionales en aplicación

**Estado:** Accepted, 2026-10-03. **Ámbito:** corrección de T04b antes de integrar; no nueva capacidad de producto.

**Contexto:** la revisión encontró pin ignorado, historial genérico sin identidad del cambio y coordinación de casos de uso dentro del adaptador SQLite. Separar sólo algunas funciones puras no cumple la arquitectura aceptada de aplicación con transacción prestada.

**Decisión:** adoptar [TASK04_APPLICATION_CASES](../plans/TASK04_APPLICATION_CASES.md). Cinco funciones application reciben TX, WorkflowRepository y LibraryRepository. Se añaden cuatro primitivas de persistencia y un struct prestado; se reutilizan paper/update_lifecycle/audit_change y archive_paper_in_tx, sin ampliar LibraryRepository. Un plan concreto puro decide destino/estado conservado y activación NEW→ACTIVE; application aplica clocks/pins/CAS y eventos específicos. Pin no soportado falla explícitamente, nunca se reinterpreta como v1. Auditoría genérica de petición legacy permanece separada de cambios efectivos; no-op/replay no inventa modificación.

**Fronteras:** no modificar WorkflowPersistence, wire/DTOs, SQLite schema, actor, UnitOfWork, Reader, Windows ni receipts. Adaptador conserva SQL/mapping, trabajo del actor, replay, revalidación documental y ownership de proof/permisos alrededor de todo commit/rollback. No mover SQL a application ni crear un motor genérico de efectos.

**Verificación:** plan puro primera aceptación/reconfirmación; pin persistido y no soportado; auditoría before/after con identidad; no-op/CAS/replay; rollback tardío en evento/receipt y regresiones de ownership. La aceptación del anexo no acredita implementación ni gate.

**Coste:** cuatro métodos de repositorio, un plan concreto y extracción de cinco casos existentes. Se descarta replicar servicios Library o añadir otro harness/actor/UoW, porque sus interfaces actuales cubren la composición necesaria.

## ADR-022 — Captura y conceptos con contratos transaccionales cerrados

**Estado:** Accepted por Sol, 2026-10-03, antes de implementar T05. Revisión documental histórica R1–R5 registrada; revisión final PASS, sin hallazgos importantes abiertos. No capacidad implementada por esta decisión.

**Decisión:** adoptar TASK05_PORTS con catorce commands wire existentes y casos application explícitos captura/create/update/lifecycle/link. TX prestada, replay previo, CAS/no-op y alcance/inversa P2 coherentes; eventos efectivos con requestId y before/after.0003 tiene una autoridad por atributo, catálogo core1.0.0 y esquema consumidor T07/T08 sin sus servicios. T05a entrega schema/dominio/captura; T05b edits/queries/IPC, secuenciales con autor nuevo y revisión.

**Semántica:** NONE conserva ausencia, padre/procedencias inicial revision0, anclaje cerrado page_region y LOCATED como registro. Fuente/hash inmutables y STALE conservado al editar; no revalidación implícita. ConceptApi controla canon; Knowledge sólo confidence de Concept; Insight affected es subset explícito de enlaces. IDs sets canónicos, homónimos permitidos, alias intraconcepto únicos. Nuevas escrituras sobre padre archivado exigen restore; enlace existente desde padre activo sigue no-op con destino archivado. Relations activas únicas por extremos/tipo/contexto; create/update/restore en colisión Conflict atómico.

**Verificación:** captura rollback, doce attrs roundtrip/reopen, migración/backup, CAS/replay/no-op, auditoría específica, cierre transitivo/ciclos y no expansión por relación, link archivado, independencia de candidate answer, locator pendiente/stale, catorce handlers y propiedad de permisos. DDL/Rust/SQL necesitan revisión ejecutable posterior.

**Costes y alternativas:** mirror de nombre acotado y verificado; no dedup global ni fusión semántica. Alias normalizados no prueban equivalencia. v0.1 no elimina STALE ni reasigna fuente silenciosamente. Se descarta nueva API/CAS para set-add link, duplicar application en adapters o guardar procedencia ficticia. No wire/DTO/dependencias nuevos ni cambio de migraciones integradas.

## ADR-023 — Candidatos persistentes y habilitación completa de P2

**Estado:** Accepted por Sol, 2026-10-03; revisión documental independiente PASS. Semántica exacta en [TASK06_DECISIONS](../plans/TASK06_DECISIONS.md). No implementado por esta decisión; ABI de T06 pendiente de contrastar con T05 integrada.

**Decisión:** set candidato exige P2 iniciada, conserva contexto y usa CAS P2/replay existente. Seleccionar exige claim/question activo con asociación directa; deseleccionar archivado está permitido, priority/rationale deben ser null. Decisión final cero exige justificación canónica1..5000, con candidatos null; genera ANSWERED. Archive/restore conserva selección/count; summary incluye archivados, gate los invalida como artefactos vigentes. Save no modifica selección y permite borrador PENDING/null, con objeto presente siempre completo/comprobado. Única justificación en PhaseAnswer, sin segundo almacén.

**Entrega:** T06a candidatos/backend/policy, T06b captura/cola/UI; todos save/evaluate/advance P2 de producción esperan T07. Sin habilitación por key ni flag wire nuevo. En T06, ausencia con cero items todavía no se guarda mediante save ni mediante item ficticio. T07 habilita el conjunto al conectar resolutores completos; fixtures no acreditan capacidades. Preguntas PRE sirven de contexto manual, sin NLP ni asociación nueva por fragmento.

**Verificación:** CAS/no-op/replay, último deseleccionado/justificación, PENDING/null, archivado seleccionado visible e inválido, rollback de asociación/respuesta/clock/audit/receipt, ausencia de activación implícita y de aceptación sin resolutores. Contrastar ABI/decoders/cierre con T05 antes del autor T06.

**Costes:** la cola conserva candidatos inválidos señalados; no permite precandidatos antes de P2; los formularios formales esperan un hito más. Se evita divergencia al filtrar archivados y habilitación parcial difícil de explicar. No schema nuevo ni reescritura de migraciones.
