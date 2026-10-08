# Espacio de trabajo y grafo de investigación

Fecha: 7 de octubre de 2026. Estado: diseño de ampliación solicitado por el usuario. No acredita implementación. Complementa el diseño de tutorial y configuración científica. Ambos deben trasladarse conjuntamente a los contratos antes de activar T05.

## Objetivo

Dar a Research Workbench una interfaz de escritorio inspirada en Obsidian y permitir explorar conexiones reales mediante un grafo global y un grafo centrado en un paper. El usuario puede registrar conexiones entre papers con significado tipado o como enlaces simples. Desde el grafo puede abrir un paper, consultar una conexión y continuar explorando.

La petición también fija el estilo editorial. Toda prosa mantenida y todo texto propio de la interfaz deben evitar la raya larga y el punto y coma. Se conservan los caracteres necesarios en sintaxis de código, URLs, datos del usuario, citas literales y archivos históricos. La aplicación nunca reescribe notas o citas del investigador para imponer este estilo.

## Decisiones visuales

Se adopta un escritorio oscuro y compacto con tema claro seleccionable. La inspiración es la organización espacial y la calma visual de Obsidian. No se copian su marca, iconos propios ni código.

| Elemento | Comportamiento previsto |
|---|---|
| Barra estrecha de herramientas | Biblioteca, grafo global, búsqueda, tutorial y configuración, con nombre accesible y ayuda visible |
| Panel izquierdo | Navegación y lista de papers, plegable y redimensionable |
| Zona central | Pestañas de trabajo para biblioteca, paper y grafo. Abrir un nodo reutiliza la pestaña del mismo paper |
| Paper | Secciones Lectura, Procesamiento, Conocimiento y Grafo local |
| Panel derecho | Propiedades y conexiones de la selección, plegable. El tutorial usa este espacio sin tapar los campos |
| Barra inferior | Biblioteca activa, estado de guardado y estado de recuperación cuando corresponda |

La interfaz sustituye las tarjetas de gran tamaño y el fondo verdoso actuales por superficies planas, separadores sutiles y jerarquía tipográfica. Se conserva el PDF con su presentación original, sin invertir sus colores por cambiar de tema.

Paleta inicial oscura: fondo `#17171b`, panel `#202025`, superficie activa `#292930`, borde `#383840`, texto principal `#eeeef2`, secundario `#b4b4c0` y acento `#b39aff`. Paleta clara: fondo `#faf9fc`, panel `#f0eef5`, texto `#24222b`, secundario `#5e586c` y acento `#6844ad`. Son valores iniciales sujetos a medición de contraste, no un resultado de accesibilidad ya validado.

Usar Segoe UI, texto base de 14 px y títulos discretos. Botones compactos con áreas de interacción suficientes, foco visible, atajos descubribles y paneles que se repliegan antes de recortar el contenido. A 200 % de zoom se conserva acceso a todas las acciones. El tema, la disposición y el movimiento reducido son preferencias de presentación. No modifican datos científicos.

El tutorial permanece accesible desde un botón identificado como Tutorial en todos los espacios. La navegación por pestañas conserva los borradores de la sesión y el estado del lector. Cerrar una pestaña con cambios pendientes permite guardar, descartar explícitamente o cancelar. Volver al grafo restaura centro, filtros y selección mientras la biblioteca siga activa.

## Qué representan las conexiones

Se mantienen tres categorías con leyenda explícita.

1. **Enlace entre papers**. Registro explícito realizado por el investigador. Puede ser simple o tener una definición semántica versionada.
2. **Relación entre elementos de conocimiento**. La Relation existente conecta KnowledgeItems según la ontología. Sigue conservando sus extremos, contexto y procedencia.
3. **Asociación derivada**. Por ejemplo, dos papers comparten un Concept UUID. Es una consulta explicable sobre enlaces existentes. No se guarda como una afirmación nueva ni se etiqueta como cita.

La vista Papers muestra conexiones explícitas por defecto. Una capa opcional permite ver asociaciones por conceptos compartidos. La vista Conocimiento muestra items, conceptos y relaciones tipadas. Esta separación evita que toda coincidencia produzca una maraña de enlaces o parezca una conclusión científica.

Una conexión simple significa únicamente «he relacionado estos papers». No implica citación, apoyo, contradicción, causalidad o equivalencia. Un paper no se convierte en KnowledgeItem y un Reference no sustituye la identidad del paper.

## PaperLink como agregado separado

Se propone `PaperLink` con UUID, `sourcePaperId`, `targetPaperId`, `typeRef` opcional, contexto, justificación, origen declarado, lifecycle, revision y timestamps. Los extremos son FKs a Paper de la misma biblioteca. Se conservan los patrones existentes de receipt, auditoría y CAS dentro de una transacción del actor DB.

- Sin `typeRef`, el enlace es simétrico. El backend normaliza el par de UUID. No se inventa un tipo científico por defecto.
- Con `typeRef`, se resuelve la definición exacta por namespace, code y version. La definición admite extremos Paper y declara dirección o simetría. No se reutiliza una matriz de KnowledgeItem como si admitiera Papers.
- Las definiciones se amplían con un ámbito cerrado `paper_link` o `knowledge_relation`. Ese ámbito forma parte del contenido inmutable. Los catálogos personales pueden incluir ambos ámbitos, pero cada familia pertenece a uno solo.
- No hay autoenlaces, extremos inexistentes ni conexiones entre bibliotecas. Crear o modificar exige ambos papers activos. Archivar un enlace sigue permitido aunque un extremo esté archivado. Restaurarlo exige ambos extremos activos.
- Archivar un paper conserva sus enlaces. Se ocultan por defecto cuando algún extremo está archivado. El filtro de archivados permite consultarlos con su estado visible. Restaurar el paper vuelve a mostrar los enlaces que seguían activos, sin restaurar los archivados explícitamente.
- Unicidad activa por extremos normalizados cuando corresponda, tipo exacto o ausencia de tipo y contexto normalizado. Contextos diferentes y tipos diferentes permiten conexiones paralelas. Una colisión en create, update o restore produce Conflict atómico.
- El contexto usa la normalización ya fijada para relaciones. La definición del algoritmo exacto debe quedar en el contrato compartido antes del código. Se conserva el texto original presentado al investigador.
- Cambiar tipo o dirección es una edición explícita con revision. Publicar otra versión de una definición no cambia enlaces anteriores. Un enlace tipado que pierde su definición provoca un error de integridad, nunca pasa silenciosamente a simple.
- El contexto y la justificación describen el juicio del investigador. La procedencia opcional reutiliza localizadores existentes mediante una asociación propia. Una cita literal conserva fuente y localizador. Ausencia de localizador se presenta como pendiente, sin impedir un enlace personal simple.
- PaperLink no satisface automáticamente un gate P2 que requiere Relation entre KnowledgeItems. Tampoco modifica la confianza de los papers o items conectados.

El catálogo inicial de conexiones entre papers puede incluir `cites`, `extends` y `compares_with`, cada una con definición, dirección, ejemplos y límites revisados. Los nombres no bastan para afirmar que esas conexiones existen. No se inferirán desde el PDF ni desde títulos parecidos.

Para `cites`, el significado es «el investigador registra que el paper origen cita al destino». Es una declaración humana, no una comprobación automática. Sin localizador, formulario, inspector y lista muestran «Citación registrada. Procedencia pendiente». Con localizador muestran «Citación registrada. Fuente localizable» y permiten abrirla. Ninguno de estos estados se etiqueta como cita verificada. Si se necesitara una certificación bibliográfica, requeriría otro contrato de validación humana. La existencia de una cita tampoco valida el contenido citado. Esta distinción debe trasladarse a CONTRACTS y SPECS antes de habilitar el catálogo.

## Grafo global y local

El grafo global se abre desde la navegación principal. Muestra los papers del alcance seleccionado, sus enlaces y, si se activa, las asociaciones derivadas. El grafo local centra el paper abierto y muestra sus vecinos a profundidad 1 por defecto, con profundidad seleccionable de 1 a 3. El recorrido incluye enlaces entrantes y salientes y conserva las flechas de dirección.

Pulsar un nodo lo selecciona y abre sus propiedades. Doble clic o la acción Abrir paper abre su espacio de trabajo. La acción Centrar aquí cambia el centro del grafo local. Esta separación permite explorar sin perder el grafo. Atrás recupera la selección y el centro anteriores.

Pulsar una arista muestra significado, dirección, tipo y versión cuando existan, contexto, origen y procedencia disponible. Los enlaces simples se identifican como Sin tipo. Las asociaciones derivadas se representan con trazo distinto y enumeran los conceptos compartidos que las explican. No se ofrecen controles de edición sobre una arista derivada.

Acciones: acercar, alejar, ajustar a vista, buscar un nodo visible, filtrar por título/dominio/tipo de relación, mostrar archivados, alternar aislados y elegir capas. La búsqueda del grafo filtra metadatos ya soportados. No introduce una segunda sintaxis FTS.

Crear relación desde un paper o nodo abre un formulario con selector de paper destino. El tipo es opcional y el formulario explica su significado antes de guardar. La conexión aparece solo después de confirmación del backend. Una edición en conflicto conserva el borrador y permite recargar el registro.

Los papers sin enlaces siguen siendo accesibles. El grafo local vacío muestra su nodo y «Este paper todavía no tiene conexiones», con acción Añadir conexión. Biblioteca vacía, consulta sin coincidencias, capacidad todavía no disponible y error de consulta tienen mensajes diferentes. Nunca se incluyen nodos de demostración en una biblioteca real.

La posición, tamaño del nodo y proximidad visual no representan calidad, verdad o relevancia científica. Tamaño uniforme por defecto. Las relaciones paralelas se agrupan visualmente con un contador y se pueden inspeccionar por separado.

## Arquitectura y límites

SQLite sigue siendo la única fuente canónica. No se incorpora un servidor, una base de datos de grafos ni un almacén duplicado en frontend. Graph es una proyección de lectura. PaperLinks tiene dominio, casos de uso, repositorio y comandos propios.

Fronteras propuestas: `domain/paper_links`, `application/paper_links`, adaptador SQLite, comandos IPC cerrados de enlaces y consulta de grafo, DTOs Rust generados para TypeScript, `features/graph` para navegación/representación y `features/paper-links` para formularios. Los nombres ABI, permisos y tablas definitivos deben cerrarse en CONTRACTS y DATA antes de despachar código.

Una respuesta de grafo incluye identidades tipadas para evitar colisiones entre Paper y KnowledgeItem, extremos completos, tipo de arista, referencias a la definición exacta, indicador de asociación derivada y metadatos de recorte. Ninguna arista puede apuntar a un nodo ausente. La consulta se resuelve sobre una instantánea coherente de lectura.

Límites iniciales de producto: hasta 300 nodos y 1000 aristas por vista, profundidad máxima 3. Las consultas pasan por el actor DB existente. El backend limita la expansión durante el recorrido, no después de materializar toda la biblioteca. La elección es determinista, prioriza el nodo central y ordena los candidatos por UUID. La respuesta indica si el resultado está recortado y permite reducir filtros. Nunca se presenta el subconjunto como toda la biblioteca.

El layout se calcula fuera del hilo de interacción cuando sea costoso, se detiene al estabilizarse y puede pausarse. Cambiar de biblioteca cancela consultas y descarta respuestas antiguas. Las coordenadas son estado de presentación reconstruible. Un fallo del render ofrece la lista de conexiones con las mismas acciones.

Para la implementación se evaluará un componente de grafo mantenido que funcione offline y permita flechas, selección y límites de recursos. La dependencia, versión y licencia se verificarán en un spike acotado al despachar esa tarea. No se selecciona una librería por una maqueta ni se instala una ahora. La alternativa de desarrollar un motor propio añade complejidad innecesaria.

## Accesibilidad y rendimiento

Todas las acciones del grafo tienen equivalentes de teclado y una lista accesible de nodos y conexiones. Los nombres accesibles incluyen título y tipo. Dirección, selección y categorías no se comunican solo mediante color. Escape cierra el inspector emergente o cancela una interacción sin borrar datos.

Movimiento reducido desactiva la animación continua. Paneles y menús respetan foco y orden de tabulación. Se medirá contraste AA en ambos temas y se verificará el recorrido a 200 % de zoom. La lectura PDF, PRE/P1 y sus borradores deben pasar regresión tras cambiar el shell.

Presupuestos propuestos para medir en el equipo de referencia: consulta y primera vista utilizables en menos de 2 s para 300 nodos y 1000 aristas, feedback de selección en menos de 100 ms una vez estabilizado. Registrar hardware, datos y percentil 95 de 20 ejecuciones. Son criterios pendientes de prueba, no prestaciones demostradas.

## Persistencia, exportación y migración

No editar migraciones publicadas 0001 o 0002. Antes de crear 0003 se coordina el esquema de configuración y conocimiento. PaperLinks puede usar una migración posterior independiente para evitar acoplar toda la captura al grafo.

Backup y restore incluyen enlaces, definiciones históricas y asociaciones de procedencia. Export añade records explícitos de PaperLink y sus definiciones en la misma revisión de formato 2.0 propuesta por configuración científica. Un export de paper incluye sus enlaces incidentes y los metadatos de sus extremos. No recorre transitivamente todos los enlaces de los papers agregados. Completa además los Documents, Papers y localizadores referenciados por la procedencia. No incluye todos los PDFs de los vecinos salvo que formen parte del cierre documental requerido y se hayan solicitado PDFs.

El cierre es finito por conjuntos de IDs visitados. Las definiciones exactas y ambos extremos siempre se exportan. La política de archivados del export debe quedar explícita en CONTRACTS antes de implementarlo. Restore preserva IDs y revisiones sin reinterpretar un tipo retirado. No se añade un importador genérico ni se afirman capacidades aún inexistentes.

## ADR y entrega por fases

ADR-025 propuesto: espacio de trabajo con paneles, PaperLink separado y grafo derivado. Se coordina con ADR-024 de tutorial/configuración. La petición humana modifica el alcance que excluía el grafo, pero este documento no sustituye por sí solo la ABI vigente.

1. Cerrar los ADRs y revisar DOMAIN, CONTRACTS, DATA, SPECS, QUALITY y briefs afectados. Retirar la exclusión del grafo acotado y mantener fuera inferencias automáticas, plugins ejecutables y edición genérica de ontología. No activar T05 con su catálogo fijo antiguo.
2. Integrar T04c cuando complete su QA nativa pendiente. El candidato congelado no recibe cambios estéticos durante esa validación.
3. UI-01 aplica el sistema visual y la política editorial a las pantallas reales de Biblioteca, Reader y PRE/P1. UI-02 incorpora paneles y navegación preservando borradores. Ningún botón activo simula capacidades futuras.
4. PL-01 implementa PaperLink y su catálogo por ámbito, con pruebas de dominio/persistencia. PL-02 añade formularios, listado y navegación entre papers. Es utilizable sin motor de grafo.
5. GR-01 implementa consultas global/local y una lista accesible. GR-02 añade render interactivo y filtros. La capa de conocimiento se incorpora cuando T05/T07 estén integradas.
6. Extender el tutorial con creación de enlace, inspección de significado, navegación local/global y diferencia entre enlace y asociación derivada. Coordinar export/backup con T08 y validar el recorrido instalado en T09/T10.

Cada corte tiene un autor de producto nuevo, revisión independiente y merge verificado del orquestador. La planificación detallada debe fijar firmas y comandos sobre el código integrado antes de delegar. No hay autorización para que un worker invente contratos a partir de esta propuesta.

## Aceptación

| ID | Resultado verificable |
|---|---|
| ED-01 | Prosa mantenida y texto propio visible sin raya larga ni punto y coma. Código necesario, citas y datos del usuario conservados |
| UI-01 | Tema oscuro/claro coherente, foco y contraste medidos, paneles utilizables a 200 % |
| UI-02 | Navegar paper, grafo, paper conserva borradores y posición del lector |
| PL-01 | Crear enlace simple A/B y consultar desde A o B devuelve el mismo registro |
| PL-02 | Enlace dirigido conserva orientación y definición exacta después de reiniciar |
| PL-03 | Self-link, extremo ausente y colisión fallan atómicamente. CAS y replay preservan integridad |
| PL-04 | Archive/restore respeta extremos y lifecycle sin borrar conexiones |
| GR-01 | Global y local muestran únicamente registros reales de la biblioteca activa |
| GR-02 | Local respeta profundidad y límites. Toda arista tiene ambos extremos y recorte visible |
| GR-03 | Seleccionar, abrir, centrar y volver funcionan con ratón y teclado |
| GR-04 | Arista derivada explica su origen y nunca se edita como afirmación |
| GR-05 | Cambio de biblioteca descarta resultados antiguos. Error de render conserva lista accesible |
| PT-01 | Export/restore preserva enlaces, extremos, definiciones y procedencia sin cierre transitivo ilimitado |
| TU-09 | Tutorial enseña grafo y significado de las conexiones con datos de práctica aislados |

## Referencia y artefactos

La [documentación oficial del grafo de Obsidian](https://help.obsidian.md/plugins/graph), consultada el 7 de octubre de 2026, describe vistas global y local, profundidad local, filtros y navegación entre notas. Se toma como referencia de interacción. La semántica científica y los enlaces entre papers de este documento son decisiones propias de Research Workbench.

La maqueta `OBSIDIAN_WORKSPACE_MOCKUP.svg` muestra la dirección visual con títulos sintéticos. Es una imagen estática de diseño, no una pantalla de producto ni una prueba de funcionamiento.
