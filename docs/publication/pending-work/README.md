# Pending T04c UI

This patch preserves the unintegrated Workbench PRE/P1 workflow UI from source commit `67446412f9c2f82cab63ccb5bf02d3dff695bb84`. It is not applied to the imported product and is not a completed native acceptance result.

If you want to continue this work, first create your own branch from the consolidated repository. From its root, verify applicability:

```powershell
git apply --check --directory=prototypes/research-workbench docs/publication/pending-work/T04c-workflow-ui.patch
```

Apply it only on that working branch, then rerun the prototype checks and complete the pending native GUI acceptance. The patch includes its author reports; historical evidence remains under the prototype's docs/reviews/task-04/ directory. No personal library or installer is included.
