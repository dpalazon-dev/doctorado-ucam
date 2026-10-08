# T03 — checkpoint de revisión y QA

El corte de corrección 2 es 30d532714d053c09d4021bc89bafbb0168b1602a (producto e0a882267d1e5f39bb12f6bf9124d8e33edd4dd8), desde b36ae5ae0529ba429edf7a360ade7319e353e8fe. El orquestador leyó el delta completo de 15 archivos, los cuatro informes independientes y los logs finales. No está aprobado para integración.

La ejecución final de scripts/check.ps1 tras npm ci pasó: 59 pruebas frontend y 128 entradas Cargo, entre ellas dos helpers de procesos hijos. Tauri debug sin bundle pasó. Logs reales en `.worktrees/task-03-reader/work/evidence/task03-fix2-*`; el informe del autor archivado conserva dos erratas históricas: indica work/evidence bajo raíz y llama comportamientos a todas las entradas Cargo. Se solicita corrección editorial al autor, sin alterar retrospectivamente esta copia emitida.

Binario congelado de fix2: `work/qa/t03-native/candidate-32c7dc86/research-workbench.exe`, SHA256 `32C7DC86458F16D594EDFB5EABC5163EED05C015BC651418E6ACE2701C1D9CD7`. No se ha lanzado en QA nativa.

## Corrección 3 activada

Mismo autor y worktree, brief central task-03-fix-3-brief.md. Hallazgos reproducidos TypeScript: resolver save-local no retira conflicto; replay confirmado de una posición anterior marca Guardado cuando la visible sigue pendiente; desmontar/reabrir antes del resultado deja recuperación sin controles. Rust conserva exclusivamente R2a: señalizar admisión después del primer poll Pending del future real, no al construirlo. Es una deficiencia del test, sin defecto de propiedad identificado en producto. Se añade la regresión menor de candidates inválidos pedida por SPEC.

SPEC cierra refresco de recuperación F2-4. Seguridad cierra MIME mediante HTTP real Vite, sin ampliar allowlist/CSP; Rust y seguridad cierran lectura real pausada con receptor descartado y carrera Library–Reader. No reabrir estos alcances sin nueva evidencia.

## QA nativa exploratoria del corte fix1

Dos sesiones sobre el binario congelado SHA871187877DCA988FFDC0560C0C9B2C30A295F4B721005213626C3462A092C481 y PDF sintético, no sobre fix2:

- Primera sesión: timeout del helper de selector. Se corrigió su filtro que excluía ventanas del mismo PID sin owner. Esa causa era compatible con el timeout, no observada entonces.
- Un preflight posterior abortó sin abrir app al actualizarse WebView2 de154.0.4258.48 a154.0.4258.53. Se añadió driver oficial .53 local versionado, con firma Microsoft válida, conservando .48; solo cambiaron cuatro pins del runner revisado.
- Segunda sesión: run4a2c62d2ae2d471388eef9a1905381c4, runner SHA874F2552F3D11A61531C922A50DBD191E9A38612BF6566FEB7CF1530D82BFB19. Inicio, Biblioteca y diálogo Windows Abrir observados; HWND propio clase#32770 sin owner. UIA no identificó campo filename ni botón aceptar; exit1 antes de seleccionar PDF. Cleanup completo y puertos35501/35502 libres, procesos propios ausentes. JSON SHA CBA6870E4EEC5D37FF68FAC8F26DE401A0EC903D3BA0AE718AF2FE161A231CC1.

No se acreditan importación gráfica, protocolo PDF, canvas, página/zoom, reapertura o cierre normal de este corte por esas sesiones. La medición exit0 previa T02 conserva su alcance. Los informes QA y revisión se archivan aquí con su historial; evidencia pesada permanece en work/qa/t03-native. Se prepara diagnóstico acotado de controles propios, sin selección ni nueva ejecución todavía. Instalador NSIS y Windows limpio siguen pendientes.
