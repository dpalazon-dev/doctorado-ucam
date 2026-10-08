# Guía vigente de desarrollo asistido

Revisada el **1 de octubre de 2026** contra documentación primaria y skills instaladas. Describe el método del proyecto; no acredita que una configuración escrita esté activa ni que una funcionalidad exista. Entorno/dependencias reales y verificaciones pertenecen a `docs/STATUS.md` y reportes. La implementación sigue `docs/plans/IMPLEMENTATION.md` y contratos normativos, no firmas abreviadas del plan histórico.

## Instrucciones y configuración de Codex

Codex construye instrucciones desde el home y desde la raíz del proyecto hacia cwd; en cada nivel prioriza AGENTS.override.md sobre AGENTS.md y las instrucciones más cercanas se aplican después. La documentación indica límite combinado por defecto de 32 KiB. Aquí AGENTS mantiene reglas breves y remite a arquitectura/brief; los trabajadores leen esos documentos explícitamente. No modificar home global ni límite para este proyecto. Fuente: [AGENTS.md oficial](https://learn.chatgpt.com/docs/agent-configuration/agents-md).

La configuración oficial usa `[agents]`, `enabled`, `default_subagent_model`, `default_subagent_reasoning_effort` y `max_concurrent_threads_per_session`; este último excluye el hilo principal y `max_threads` es alias legado. Roles pueden referenciar `config_file` y describir responsabilidad; la configuración de rol y los overrides de spawn afectan resolución de modelo/esfuerzo. No inferir modelo efectivo solo de defaults: comprobar configuración y spawn/session evidencia. Fuentes: [referencia de configuración](https://learn.chatgpt.com/docs/config-file/config-reference), [subagentes](https://learn.chatgpt.com/docs/agent-configuration/subagents).

El acuerdo es Sol coordinador y Luna para implementaciones estrechas; tareas de integración complejas pueden recibir worker Sol mediante ruling explícito del coordinador, manteniendo defaults Luna. La documentación diferencia Sol para trabajo exigente/múltiples pasos y Luna para tareas claras/acotadas. Una intención de delegar no crea aislamiento de Git: cada worker necesita paths/ownership y worktree asignados. No editar configuración global, no instalar frameworks o SDKs globales. Fuente: [elección de modelos y roles](https://learn.chatgpt.com/docs/agent-configuration/subagents).

## Worktrees y ownership

Git worktree permite varios checkouts con ramas independientes compartiendo repositorio. Antes de crear/reutilizar, comprobar `git status --short`, `git worktree list --porcelain`, baseline SHA y branch/paths; usar worktree administrado cuando herramienta reconoce repo, Git CLI como fallback registrado si no. No compartir rama ni asumir que checkout aislado evita conflictos de archivos comunes. No eliminar directorio a mano para limpiar: conservar cambios/reporte y retirar solo worktree limpio ya integrado. Fuente: [git-worktree oficial](https://git-scm.com/docs/git-worktree).

Sol decide secuencia, freezes y reasignación de shared files; durante ejecución SDD asigna integración de código a un worker explícito y revisa resultado, sin escribir producto en el hilo coordinador. Workers no revierten cambios ajenos, no redefinen contratos ni actúan fuera del ownership. Commits/merge locales siguen protocolo autorizado de Sol; no push/publicación/remoto por defecto. Un informe de worker termina tarea técnica, no acredita gate de producto.

## SPEC → contrato → tarea → prueba

Spec Kit distingue requisitos, planificación técnica, tareas, análisis de coherencia e implementación/convergencia; checklist de requisitos y evidencia de implementación son artefactos diferentes. Se adopta esa separación documental, sin instalar Spec Kit ni duplicar la arquitectura en otro sistema. Cada task consume contrato/baseline, produce archivos/APIs/tests definidos y termina en revisión. Fuente: [Agentic SDD oficial de Spec Kit](https://github.github.com/spec-kit/reference/agentic-sdd.html).

Superpowers describe diseño, worktrees, plan, ejecución, pruebas y revisión como workflow; su instalación local gobierna los detalles de skill leída. La documentación upstream puede avanzar frente al cache instalado; nunca se presupone equivalencia de versiones ni se reinstala para este trabajo. Se mantiene ejecución continua autorizada; preguntas de permission solo cuando existe una razón concreta aplicable. Fuente: [repositorio oficial Superpowers](https://github.com/obra/superpowers).

## Skills locales usadas y reglas concretas

- [writing-plans instalado](C:/Users/david/.codex/plugins/cache/superpowers-marketplace/superpowers/6.4.2/skills/writing-plans/SKILL.md): plan por tareas con archivos exactos, consume/produce, pruebas y self-review. Ubicación solicitada `docs/plans/` reemplaza su default. La aceptación/método ya autorizados gobiernan el handoff; no pedir otra aprobación rutinaria.
- [subagent-driven-development instalado](C:/Users/david/.codex/plugins/cache/superpowers-marketplace/superpowers/6.4.2/skills/subagent-driven-development/SKILL.md): implementador acotado, revisión por tarea de SPEC/calidad y revisión final; coordinador documenta rulings y secuencia. Root ya mantiene el ledger del proyecto; no introducir segunda lista de tareas.
- [Karpathy instalado](C:/Users/david/.codex/plugins/cache/karpathy-skills/andrej-karpathy-skills/1.0.0/skills/karpathy-guidelines/SKILL.md): explicitar supuestos, resolver la necesidad mínima, cambios quirúrgicos y criterios verificables. No añade APIs/frameworks especulativos ni interpreta texto libre para decidir gates.

Estas skills son herramientas de desarrollo. La aplicación v0.1 no ejecuta agentes/LLMs. Instructions directas de usuario, AGENTS y arquitectura aceptada prevalecen sobre defaults de la skill; discrepancia material se registra en STATUS como ruling y se corrige en todos los consumidores.

## Evidencia de integración y entrega

Antes de integrar cada tarea: contrato y ownership sin drift, tests afectados en PASS, revisión real Rust/TypeScript/general y seguridad donde corresponde; después checks de composición e IPC. Reportar commands/exit codes/fixture/commit y evidencia `automated|manual|simulated|pending`. DTOs y fixtures se regeneran/verifican, no dos tipos paralelos mantenidos a mano.

Windows instalado/offline se demuestra con NSIS exacto/SHA y entorno limpio sin herramientas ni WebView2, pruebas de continuidad/upgrade/backup/migración/uninstall/reinstall y finalmente flujo PRE–P2/portabilidad v0.1. Mocks, build y exe de checkout no satisfacen ese gate. Si falta entorno limpio, seguir trabajo independiente y conservar gate pending; no rebajar el criterio ni anunciar instalación comprobada. Criterios del proyecto: `docs/architecture/QUALITY.md` y SPEC-001.
