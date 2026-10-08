# Tutorial y configuración científica del Research Workbench

Fecha: 7 de octubre de 2026. Estado: **propuesta de diseño e investigación. No implementada y no sustituye todavía los contratos vigentes**. Petición humana: tutorial útil ejecutable desde un botón. Configurar revista, temática/dominio, símbolos de notas y ontología de relaciones y sus significados. La autorización general de desarrollo continúa. Esta ampliación necesita cerrar contratos antes de activar sus implementadores.

## 1. Objetivo y supuestos

La aplicación debe enseñar el trabajo real de investigación y permitir adaptar el vocabulario personal sin alterar el significado de datos antiguos. Conserva Windows local/offline, un usuario, SQLite canónico, trazabilidad y ausencia de LLM. El tutorial no evalúa competencia científica ni certifica veracidad.

Supuesto explícito ante «tipo de revista»: cubrir **revista/medio de publicación y tipo de artículo/revisión como campos distintos**. Se preguntó por esta ambigüedad. No se recibió respuesta en el contexto recuperado. Ambos campos son útiles independientemente y no deben mezclarse. La propuesta es corregible sin cambiar datos existentes.

Estado comprobado: producto5d35f03, esquema2. Onboarding de bienvenida, Library/Reader y PRE/P1. No existen tutorial repetible, preferencias semánticas configurables ni implementación Knowledge/ontología0003. DOMAIN define doce tipos y trece relaciones cerradas, y excluye editor genérico de ontologías. La petición nueva amplía ese alcance. No basta añadir colores en Settings. Preflight archivado junto a este diseño.

## 2. Interpretación científica y fuentes

Estas fuentes justifican distinciones, no una validación científica de nuestra interfaz ni un vocabulario universal de símbolos.

| Fuente primaria consultada | Aportación | Límite de uso aquí |
|---|---|---|
| [W3C Web Annotation](https://www.w3.org/TR/annotation-model/) - principios y motivaciones | Separa cuerpo, objeto anotado e intención: destacar, comentar, clasificar o evaluar. | Inspiración para separar marca y contenido. No se afirma conformidad con JSON-LD por usar estos conceptos. |
| [W3C SKOS](https://www.w3.org/TR/skos-reference/) - etiquetas, notaciones, notas y relaciones | Distingue identidad conceptual, etiqueta, definición y relaciones. `related` es simétrica. | Un icono UI no equivale automáticamente a skos:notation. `part_of` tampoco equivale sin más a `broader`. No importar inferencias ajenas. |
| [CiTO](https://sparontologies.github.io/cito/current/cito.html) - citesAsEvidence, citesAsPotentialSolution, citesAsRecommendedReading | Permite distinguir la función de una cita. | Una relación de citación expresa cómo se usa una fuente. No garantiza que el resultado citado sea verdadero. Nuestros enlaces Evidence→Claim no son automáticamente equivalentes a CiTO. |
| [W3C PROV Overview](https://www.w3.org/TR/prov-overview/) | Procedencia sobre entidades, actividades y agentes facilita evaluar fiabilidad. | Registrar quién afirma algo, de dónde sale y cómo se transforma no demuestra por sí solo su validez. |
| [Cochrane Handbook, capítulo14](https://www.cochrane.org/authors/handbooks-and-manuals/handbook/current/chapter-14) | La certeza se valora sobre un cuerpo de evidencia y un resultado. Considera sesgo, inconsistencia, indirectitud, imprecisión y sesgo de publicación. | GRADE pertenece a un contexto metodológico específico. No trasladar una puntuación automática a papers de IA/ingeniería ni equiparar nuestra confianza personal a GRADE. |
| [PRISMA2020, artículo original](https://www.bmj.com/content/372/bmj.n71), [sitio oficial](https://www.prisma-statement.org/prisma-2020) | Guía de reporte de revisiones sistemáticas. El artículo advierte que no es una herramienta para evaluar calidad metodológica. | Completar un checklist o un workflow no convierte una revisión en correcta ni un claim en probado. El artículo se recuperó por resultado indexado. Apertura completa BMJ falló. |
| [W3C WAI, formularios por pasos](https://www.w3.org/WAI/tutorials/forms/multi-page/) | Orientación, pasos identificables y progreso comprensible. | Se adapta al tutorial por decisión de diseño. No es un estudio que demuestre eficacia de nuestro tutorial. |

**Decisión recomendada:** conservar ejes separados. Tipo del contenido, origen, localización/procedencia, confianza humana e importancia para el lector no se deducen entre sí.

| Marca inicial propuesta, configurable | Significado operativo | Representación recomendada |
|---|---|---|
| `!` Importante | Merece atención para mi objetivo de lectura. | Marca personal independiente, compatible con cualquier tipo. No aumenta confianza. |
| `REF` Referencia | Fuente que quiero citar, localizar o leer. | Reference y su identificador bibliográfico. No Evidence por existir DOI. |
| `C` Afirmación | Proposición expresada por la fuente o por el investigador. | Claim, origen explícito y procedencia cuando exista. |
| `OBS` Resultado observado/reportado | Medición o resultado con método, condiciones y límites. | Evidence, distinguiendo resultado reportado por autores de reproducción propia. |
| `FACT` Hecho documentado | Expresión del usuario que necesita contexto y fuente. | Acceso guiado a Claim o Evidence según contenido. Nunca asigna «verdadero» o confianza suficiente automáticamente. Etiqueta visible recomendada: «Hecho reportado». |
| `H` Hipótesis | Proposición propia pendiente de contrastar. | Claim con origin=researcher_hypothesis. No nuevo tipo incompatible. |
| `?` Pregunta | Incertidumbre o acción de investigación. | Question. |
| `LIM` Límite | Restricción explícita de una afirmación o método. | Limitation. |
| `IDEA` Interpretación/síntesis | Lectura propia que conecta resultados. | Insight con origen investigador. |

Los doce tipos core siguen disponibles. Esta tabla es una paleta inicial abreviada, no su reemplazo. Los símbolos son convenciones del producto elegidas por usabilidad, no estándares científicos universales. No basta color: siempre etiqueta textual y descripción accesible.

Ejemplo **ficticio**: «F1=0,94 en el conjunto X con partición temporal Y» se captura como resultado reportado con página/tabla, condiciones y límites. «El método generaliza a cualquier planta» es otra afirmación, con respaldo pendiente. `!` puede marcar ambas por relevancia. Una arista supports expresa el respaldo interpretado para un contexto. No transforma el segundo texto en un hecho establecido.

## 3. Tutorial ejecutable

**Botón persistente `Tutorial` en el shell**, también accesible desde Ayuda. Abre un índice con duración orientativa, módulos disponibles y progreso local. Acciones Empezar, Reanudar y Reiniciar progreso. No depender del diálogo de primera apertura. Funcionamiento íntegro offline, texto en español.

Dos modos complementarios:

1. **Guía sobre mi espacio:** explica la vista actual, muestra el siguiente paso y permite regresar al índice. No ejecuta mutaciones de negocio al pulsar Siguiente ni cambia de biblioteca. Cerrar/repetir la guía conserva borradores. Ante una acción real de guardar/archivar, sólo la acción explícita del usuario usa el flujo normal.
2. **Práctica con ejemplo:** biblioteca de demostración claramente rotulada, PDF sintético y entidades separadas. Usa los servicios reales, no mocks que aparenten persistencia. La entrada/salida pasa por el coordinador de biblioteca. Guarda el contexto anterior y permite recuperarlo al reiniciar. Sólo se habilita cuando switch/recovery y aislamiento estén implementados y probados. No copiar ni modificar contenido personal para practicar.

Módulos de la entrega completa: (a) biblioteca y metadatos. (b) PDF/localizadores. (c) PRE/P1 y guardar frente a completar. (d) Claim/Evidence/Reference/importancia. (e) conceptos compartidos, relaciones y contexto. (f) revisión P2 y pendientes. (g) búsqueda/export/backup/restauración. (h) personalizar perfiles/símbolos/definiciones. Se puede abrir un módulo directamente.

Cada lección explica objetivo, acción concreta, resultado observable, error habitual y forma de recuperarse. Ejercicio clave: crear una afirmación y una evidencia distintas, localizar la fuente y enlazarlas con contexto, distinguiendo cita y explicación propia. «Lección terminada» significa que se recorrió el ejercicio, nunca «paper validado».

Especificación funcional TU-01…08:

- TU-01: disponible por botón tras primera ejecución. Repetir no necesita reiniciar la app.
- TU-02: Anterior/Siguiente/Salir, índice y paso actual. Teclado, foco restaurado, lector de pantalla y zoom. Ningún overlay tapa la acción que explica. Alternativa panel lateral.
- TU-03: progreso por lessonId/tutorialVersion, independiente de phases/answers. Reiniciar tutorial sólo reinicia ese progreso.
- TU-04: selección de lecciones por capacidades reales. Función aún ausente se describe como pendiente y no se usa para acreditar ejercicio completado.
- TU-05: resolver objetivos mediante identificadores estables de UI, no coordenadas. Si falta objetivo, ofrecer explicación/volver sin bloquear la app.
- TU-06: guardar/reabrir confirma datos reales. Cerrar guía preserva borradores. Ninguna escritura automática en la biblioteca personal.
- TU-07: práctica con aislamiento y regreso recuperable. Nunca borrar una raíz arbitraria ni reutilizar la biblioteca personal.
- TU-08: contenido y glosario empaquetados/versionados. Ejemplos usan el perfil seleccionado y las definiciones históricas correctas.

## 4. Metadatos y perfiles configurables

Configuración ofrece secciones **Bibliografía**, **Áreas y temas**, **Símbolos**, **Tipos y relaciones**, **Tutorial**. Formularios específicos, no editor de JSON o SQL.

Separar:

- **Venue:** revista/congreso/repositorio, nombre e identificador cuando se conozca. Editar catálogo no inventa cuartil, indexación ni calidad. Desconocidos explícitos.
- **Tipo documental/editorial:** artículo, revisión, preprint, tesis, informe, etc. Catálogo local con IDs estables, etiquetas/definiciones y archivado. Esto no es el tipo del venue.
- **Tipo de revisión:** survey, SLR, mapping, etc. Conservar ReviewType vigente para las reglas de PRE. Un subtipo personal debe mapear explícitamente a un tipo canónico o a other. No inferir que un artículo experimental satisface un workflow diseñado para reviews. Describir esta limitación en la UI y mantener una futura ampliación de workflow como decisión separada.
- **Dominio y temas:** dominio principal más temas múltiples seleccionables. Definiciones y alias para evitar duplicados léxicos. Ejemplo: sistemas de agua. Temas: detección de anomalías, OT, series temporales. Tags temáticos no crean Concept científicos automáticamente.

Un **perfil de investigación** agrupa vocabularios disponibles, símbolos, orden/favoritos y ayudas de captura. El perfil inicial es general. Un perfil IA/agua puede ajustar presentación y ejemplos. No añadir tipos Dataset/Metric/System a los doce tipos sin contrato propio.

Al cambiar perfil, los registros conservan sus IDs y definiciones de origen. Los nuevos defaults afectan a futuras capturas. Renombrar etiquetas no recategoriza papers. Cambiar clasificación de un paper es una edición real con revisión/auditoría e invalidación cuando corresponde. Archivar un término lo retira de nuevas selecciones, manteniendo legibles sus usos históricos.

## 5. Ontología configurable con historia

| Alternativa | Ventaja | Coste/decisión |
|---|---|---|
| Sólo iconos y alias del núcleo | Cambio reducido. | Insuficiente para definir nuevas relaciones y significados solicitados. |
| Núcleo estable + extensiones declarativas versionadas | Personalización real, validación y registros históricos interpretables. | **Recomendada**. Requiere ampliar contratos/persistencia/export y los consumidores. |
| Editor OWL/RDF e inferencia general | Máxima expresividad. | Complejidad fuera de la necesidad actual. No recomendado para v0.1. |

La UI de una relación muestra nombre, definición, dirección, extremos permitidos, ejemplo válido, contraejemplo y si es simétrica. El usuario puede ordenar/ocultar/favoritar relaciones, personalizar su presentación y crear relaciones propias entre tipos core. Ejemplo: `evalúa_en` Method→Condition, con contexto obligatorio según la definición. No admitir código ejecutable ni reglas libres.

**Identidad semántica:** DefinitionRef=(namespace, code, version). Core conserva core: supports, contradicts, etc.. Una definición personal tiene identidad separada. Cada relación guardada referencia exactamente una versión. Cambio de significado, dirección o matriz crea nueva versión. Registros antiguos siguen apuntando a la anterior. El usuario puede escribir notas explicativas personales sin editar la definición canónica. Si quiere cambiar la semántica core, crea una variante propia con vínculo descriptivo al término origen, no sobrescribe core.

No activar transitividad o inversión inferidas por defecto. `similar_to`, `part_of`, `causes` y `supports` no son intercambiables. Una definición permite una arista, pero no prueba su contenido. El núcleo conserva sus validaciones específicas. Las extensiones no satisfacen automáticamente requisitos de P2 ni reciben confianza por parecerse léxicamente a supports. Usan revisión humana genérica hasta aprobar un mapping explícito de política.

Publicación local de una definición: borrador → validar → publicar versión inmutable. Preview del cambio y conteo de usos, sin reescritura masiva. Reaplicar nueva definición a relaciones existentes exige operación futura separada con preview y auditoría. No conversión silenciosa. Retirar una definición bloquea usos nuevos, no rompe los viejos.

## 6. Arquitectura y contratos que deben cerrarse

Las decisiones de alcance, identidad e intercambio/backup se concretan en §9 tras revisión independiente. Esta sección enumera responsabilidades. Siguen pendientes las firmas ejecutables, los límites y la migración SQL coordinada.

Sin nuevo servidor ni harness. Dominio Rust valida vocabulario. Casos de uso transaccionales coordinan revisión optimista, receipt/auditoría y referencias. Adaptador SQLite en el actor existente. Rust sigue generando DTO TypeScript. UI sólo comandos tipados y capacidades declaradas.

| Responsabilidad | Datos propuestos | Contratos de aplicación por concretar |
|---|---|---|
| Tutorial | tutorialVersion, lessonId, status/lastStep. Contenido empaquetado | Consultar/reanudar/reiniciar progreso. Entrada/salida de práctica mediante biblioteca existente. |
| Perfil/presentación | profileId/revision, preferencias. MarkerDefinition con código/etiqueta/glifo/ayuda/targetSemántico | Leer/editar perfil, validar colisiones de atajos, archivar marca. Presets no cambian confidence/origin sin selección explícita. |
| Clasificación | términos versionados con categoría venueKind/documentKind/domain/topic. Asociaciones paper–término | CRUD acotado, búsqueda y asignación con CAS. Conservar valores legacy sin adivinar equivalencias. |
| Definiciones de relación | DefinitionRef, definición, matriz core, simetría, contexto requerido, estado | Listar/ver, guardar borrador, validar/publicar, retirar. Crear/leer relación con referencia versionada. |

`!` necesita asociación de marca a item, con identidad estable. No se incrusta como prefijo de body. Un preset FACT orienta la captura, no añade un booleano de verdad. Cita y comentario permanecen campos distintos. No se añadirá un segundo almacén canónico frontend.

Las tablas concretas y firmas ABI no se declaran cerradas en este documento. T05a aún no implementó0003: resolver el esquema ampliado antes de asignarlo. Nunca editar0001/0002 publicadas. Separar versión de esquema, ontología, tutorial y formato de export. Una ampliación de DTO exige regeneración/verificación de ambos extremos y decidir compatibilidad explícita. No asumir que cambiar contractVersion del sobre soluciona la semántica.

Export debe incluir definiciones semánticas históricas referenciadas y asociaciones nuevas. Revisar su allowlist y versión antes de añadir records. Preferencias visuales/progreso del tutorial no se mezclan con evidencia científica compartida. Backup local conserva configuración pertinente para recuperar la experiencia. Un importador que desconozca una definición no debe reinterpretarla como una core conocida: mostrar limitación explícita o rechazar atómicamente según el contrato de import aprobado. No se afirma import genérico ya disponible.

Migración de datos: preservar UUID, código/tipo core, texto y confianza. domain/venue antiguos se mantienen. Crear términos sólo con equivalencia textual explícita y sin fusionar sentidos. Verificar forward upgrade, reopen, backup/restore y downgrade en modo diagnóstico. El historial científico no depende de que el perfil visual original siga activo.

## 7. Orden de ejecución y ADR propuesto

**ADR-024 propuesto:** configuración mediante perfiles, marcas independientes y ontología declarativa versionada. Tutorial repetible separado del workflow científico. Afecta ADR-008, DOMAIN, CONTRACTS, DATA, SPECS, QUALITY, TASK05_PORTS, TASK06_DECISIONS y T07/T08/T09. Aún no aceptar ni implementar interfaces incompatibles a partir de esta propuesta.

1. Conservar y cerrar T04c como hito independiente. QA4 fue interrumpida: no hay resultado agregado/exit final recuperado, ni procesos app/driver presentes en la inspección del7octubre. No se acredita PASS. Merge sigue preparado sin confirmar.
2. Revisar esta propuesta y cerrar ADR-024 + matriz de requisitos/contratos/migración con revisión independiente. Rebasar el diseño sobre integración final antes de despachar producto.
3. Corte configuración base y tutorial Biblioteca/Reader/PRE-P1. Autor único, perfiles/metadata/presentación y contratos probados.
4. Replanificar T05/T06/T07 para captura/marcas/definiciones y consultas históricas. No crear una migración0003 que ignore este cambio aprobado cuando se cierre.
5. Extender tutorial a Knowledge/P2 cuando existan capacidades. Práctica completa después de switch/recovery. T08 incorpora cierre de referencias/definiciones en export y backup. T09 prueba recorrido completo.
6. Gate instalado final incluye tutorial offline, configuración/reinicio, cambios de perfil y lectura de registros antiguos. No sustituirlo con tests web.

Aceptación de configuración SC-01…08: guardar/reabrir perfiles. Revista/tipo/tema distintos. ! no cambia tipo/confianza. FACT no certifica. Relación inválida revierte sin receipt parcial. Publicar v2 conserva lectura v1. Retirar términos conserva enlaces. Export/restore mantiene IDs y definiciones. Tests de selección de símbolos, teclado/contraste y conflicto de atajos son funcionales, no snapshots decorativos.

## 8. Resultado y límites de esta investigación

La revisión posterior cerró conceptualmente las tres decisiones principales mediante D1–D3 en §9. El diseño recomendado está definido. Su aceptación como contrato implementable exige todavía actualizar las normas y los consumidores afectados.

La recomendación combina estándares de anotación/procedencia con distinciones de evaluación de evidencia. No existe una justificación en las fuentes consultadas para declarar universal nuestra simbología ni para calcular verdad desde una etiqueta. Tutorial y configuración son requisitos nuevos registrados. Este documento no afirma que estén implementados, probados con usuarios o científicamente validados. El alcance original de aplicación instalable y su ejecución por fases se mantiene.

## 9. Resoluciones propuestas tras revisión independiente

La revisión detectó tres decisiones necesarias antes de convertir la investigación en un brief implementable. Se concretan aquí. **son el diseño recomendado, pendiente de traslado coordinado a las normas**, no permiso para que un worker mezcle contratos viejos y nuevos.

**D1 / H1 ,  alcance.** Recomiendo incluir relaciones personales declarativas en la entrega ampliada solicitada, sin relegarlas a colores. Las trece relaciones core pasan a ser el catálogo inicial obligatorio, no el máximo del catálogo de usuario. Se mantienen los doce tipos de item. La ampliación permite nuevas relaciones entre ellos, no nuevos tipos/atributos ejecutables ni razonamiento OWL. T05a no se activa hasta revisar su migración/contratos contra esta decisión. El catálogo core sigue verificándose exactamente. Otra prueba separada valida extensiones. Esta decisión responde a la nueva petición humana y exige sustituir expresamente la restricción antigua de ADR-008/DOMAIN.

**D2 / H2 ,  identidad, lectura e intercambio.** Propuesta concreta para el ADR/ABI:

- Definición publicada con PK compuesta `(namespace, code, version)`, donde namespace es `core` o un UUID de vocabulario personal conservado por backup. Code es estable y version es una versión semántica explícita. Campos: etiqueta, definición, ejemplos/contraejemplos, matriz de extremos, simetría, requisito de contexto y hash backend de su contenido canónico. La versión publicada y su hash son inmutables. Estado de disponibilidad para nuevas selecciones se guarda aparte de su contenido semántico.
- Cada Relation referencia esas tres columnas con FK compuesta. El DTO nuevo usa `typeRef:{namespace,code,version}` en lugar de un `typeCode` libre. No mantener dos campos capaces de contradecirse. Publicar v2 añade una fila de la misma familia namespace/code. No mueve la FK de relaciones anteriores. Cambiar de familia usa otro code. Retirar una versión no la elimina.
- Lectura resuelve siempre la referencia exacta, incluso retirada. Una referencia ausente o hash inconsistente en la biblioteca es IntegrityFailure visible. No sustituir por la última versión ni por una relación del mismo nombre. Definición personal válida se renderiza mediante el mismo formulario declarativo y validación de matriz. No exige código por cada término.
- Preparar formato de export **2.0**: Relation recordVersion2 con typeRef y nuevo recordType relationDefinition para las versiones exactas necesarias. Exportar relaciones exige incluir sus definiciones y todos sus extremos. Manifest incluye archivos/hashes y las referencias exactas exportadas, no una sola versión por namespace. Renombrar perfil no altera identidad. Esta es una propuesta de evolución de formato, no un export ya implementado.
- Restore de backup exige todas las definiciones/FKs/hash y compatibilidad de esquema. Cualquier ausencia/corrupción rechaza antes del switch. Una definición personal con estructura soportada se admite aunque su namespace sea nuevo. Una estructura/versión de esquema no soportada se rechaza, sin descartar campos ni reinterpretar datos.
- No incorporar import JSONL general al alcance por esta investigación. Si se implementa después, v1 sólo podrá mapear los trece códigos conocidos a core1.0.0 mediante adaptación explícita. Códigos desconocidos rechazan antes de escribir. El backup mantiene su camino independiente y conserva todas las definiciones históricas, referenciadas o no.

**D3 / H3 ,  qué se recupera.** Los perfiles, catálogos, definiciones, marcas aplicadas y preferencias visuales del perfil pertenecen a la biblioteca y viajan en su snapshot de backup. Restore a una raíz nueva recupera exactamente ese estado, sin fusionarlo con otra biblioteca. El export científico incluye marcas aplicadas y las definiciones necesarias para interpretarlas, además de clasificaciones/relaciones. Omite favoritos, atajos, colores y progreso de aprendizaje. Deberán añadirse sus records cerrados al contrato2.0, no incrustarlos libremente en JSON.

El progreso tutorial es también por biblioteca y se incluye en backup para reanudar. No viaja en export científico. Se identifica por tutorialVersion y lessonId. Al restaurar con otra versión de la app se conserva el historial, pero sólo se reanuda automáticamente una versión de lección compatible. Las incompatibles se muestran como «realizadas en una versión anterior» y ofrecen empezar la nueva. Ningún paso antiguo acredita una lección nueva automáticamente. La biblioteca de práctica tiene progreso independiente. No se mezcla con el de investigación. El regreso desde práctica usa el mecanismo recuperable de cambio de biblioteca y sus rutas locales, que no forman parte del export compartible.

Quedan por redactar como contratos ejecutables las firmas exactas, límites/validaciones, records de marcas/clasificación y migración SQL. Este cierre conceptual resuelve las tres decisiones de diseño. No afirma que el documento sea todavía un brief de implementación completo.
