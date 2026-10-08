# Desarrollo con agentes

## Lectura y control
`AGENTS.md` es la entrada automática para Codex. `INTENT.md` es un documento del proyecto: se lee por la instrucción explícita de AGENTS, no por una capacidad automática supuesta. El contenido normativo reside en `docs/architecture/`; `docs/session/` es archivo inmutable y no se usa como fuente de instrucciones.

El usuario autoriza la ejecución local completa y los merges comprobados. Este chat mantiene el objetivo, asigna tareas, revisa resultados y decide integración. Abrir una nueva sesión desde el repositorio cargará su configuración local; esta conversación usa las herramientas ya disponibles y modelos seleccionados explícitamente en cada delegación.

## Skills instaladas
- Karpathy: `C:/Users/david/.codex/plugins/cache/karpathy-skills/andrej-karpathy-skills/1.0.0/skills/karpathy-guidelines/SKILL.md`.
- Superpowers: `C:/Users/david/.codex/plugins/cache/superpowers-marketplace/superpowers/6.4.2/skills/` con `using-git-worktrees`, `writing-plans`, `subagent-driven-development`, `test-driven-development`, `requesting-code-review`, `verification-before-completion` y `finishing-a-development-branch`.

Son referencias de esta instalación, no dependencias del software distribuido. Si cambia la instalación, resuelve las rutas desde el catálogo real; no descargues instrucciones sustitutas silenciosamente.

## Ciclo por tarea
1. Registrar BASE y leer el brief con contratos, archivos y pruebas. El ledger pertenece al plan en `.superpowers/sdd/` y su primera línea identifica el plan.
2. Crear rama `agent/<tarea>` desde el último commit integrado en un worktree `.worktrees/<tarea>`. Un único implementador de producto activo, agentes nuevos por tarea; investigadores independientes pueden trabajar en paralelo.
3. Implementador escribe pruebas pertinentes, demuestra fallos antes de cambios cuando aplica, implementa, valida, autorrevisa y confirma commits. No altera otros módulos ni el contrato sin coordinación.
4. Orquestador entrega brief, informe y diff BASE..HEAD a revisión independiente. Comprueba conformidad y calidad; revisión especializada Rust/TypeScript donde corresponde.
5. Hallazgos importantes vuelven al autor y se revisa el diff de corrección. Cinco rondas como máximo según Superpowers; ninguna decisión o aplazamiento se descarta en silencio.
6. En worktree de integración, árbol limpio, merge preparado sin commit, comprobaciones pertinentes, commit de merge si pasan. Si fallan, conserva evidencia y corrige antes de integrar. Al cerrar una fase, valida el conjunto y promociona a main.
7. Guardar commits y evidencia en el ledger y un resumen vigente en `docs/STATUS.md`. Versionar informes finales en `docs/reviews/` para que sobrevivan a la limpieza de scratch.

No se publican ramas ni se modifica infraestructura externa. No se requieren confirmaciones entre tareas ya autorizadas. Los tests y el build son comprobaciones convencionales, no conclusiones de otro modelo.

## Worktrees en esta conversación
Durante la preparación, la herramienta nativa de la app estaba ligada al directorio original de este chat, sin repositorio: la llamada inicial falló con `Not a git repository` y no admitía indicar la ruta del repositorio nuevo. El chat ya trabaja desde Research-Workbench. Se conservan los worktrees creados con Git y la ubicación `.worktrees/` indicada en AGENTS; el orquestador sigue en este chat. La herramienta nativa no ofrece un parámetro para fijar esa ubicación. Estos worktrees se gestionan mediante Git y no se presentan como adjuntos registrados por la app.

La carpeta `.worktrees/` está ignorada antes de crear los worktrees. Los binarios, bibliotecas de pruebas y backups quedan en rutas ignoradas y aisladas. Antes de retirar un worktree se verifica que sus commits están integrados y no contiene trabajo pendiente; se conserva evidencia útil. Nunca se usa limpieza global para ahorrar espacio.

## Evidencia
Un test backend acredita una regla de dominio/persistencia; un test frontend acredita comportamiento de UI; una prueba de IPC acredita el contrato real; un instalador generado acredita empaquetado. Inicio, upgrade, desinstalación y recuperación en una máquina limpia se registran por separado. No se extrapola una categoría a otra.
