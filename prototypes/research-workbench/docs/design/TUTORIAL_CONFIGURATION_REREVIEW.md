# Segunda revisión: cierre conceptual de H1–H3

Fecha: 7 de octubre de 2026  
Resultado: **PASS documental con una precisión editorial recomendada.** La sección 9 resuelve conceptualmente los tres hallazgos anteriores. No es una aprobación de implementación ni exige que una propuesta exploratoria contenga ya firmas o SQL.

## Revisión de los hallazgos

- **H1 / D1 — cerrado.** Las trece relaciones core quedan como catálogo inicial obligatorio, no como máximo; las extensiones declarativas solo relacionan los doce tipos core. La sección exige sustituir ADR-008/DOMAIN y revisar T05 antes de activar T05a, evitando mezclar silenciosamente el contrato vigente con la propuesta.
- **H2 / D2 — cerrado a nivel de diseño.** Una referencia compuesta a namespace, code y version preserva la definición histórica exacta. La versión publicada queda inmutable, la disponibilidad se separa de la semántica, la lectura no sustituye por la última definición, y export 2.0 transporta definiciones necesarias con sus relaciones. Restore rechaza referencias ausentes o incompatibles. También queda explícito que no se añade import JSONL general a esta tarea y qué hacer con códigos v1 desconocidos.
- **H3 / D3 — cerrado.** Perfiles, catálogos, marcas aplicadas, preferencias y progreso son por biblioteca y viajan en backup; export científico omite preferencias y progreso, pero conserva las marcas aplicadas y sus definiciones interpretativas. El progreso de práctica queda aislado, y la reanudación entre versiones de lección no acredita automáticamente contenido nuevo.

## Precisión editorial

Las secciones 5–7 y 9 son consistentes en sus decisiones, pero las secciones 6 y 8 todavía contienen frases redactadas como pendientes —por ejemplo, que los contratos deben “cerrarse”, revisar versiones/allowlists, o que la ampliación “necesita cerrar contratos”— sin señalar que D1–D3 ya proponen el cierre conceptual. Antes de circular el documento como recomendación, ajustar esas frases para diferenciar: decisiones conceptuales ya recomendadas en sección 9 y contratos normativos ejecutables aún pendientes. Esto evita que un lector confunda la falta de firmas/DDL con una decisión de alcance aún abierta.

## Límites

Revisé solo el cierre H1–H3 y la coherencia interna de la propuesta actualizada, contrastándola con el informe previo. No revisé fuentes web, contratos ejecutables actualizados, implementación, migración, interoperabilidad real, ni comportamiento de backup/restore. D1–D3 no sustituyen la actualización coordinada de ADR, DOMAIN, CONTRACTS, DATA, T05 y export/backup, y no habilitan por sí solos su implementación.
