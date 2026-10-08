use super::actor::DbActor;
use crate::{
    application::settings::{LibraryStatus, LibrarySummary, RecoveryStatus, SettingsQuery},
    desktop::maintenance::MaintenanceCoordinator,
    transport::error::AppError,
};
use std::{future::Future, pin::Pin};
pub struct ActorSettingsQuery {
    actor: DbActor,
    maintenance: MaintenanceCoordinator,
    recovery: RecoveryStatus,
}
impl ActorSettingsQuery {
    pub fn new(actor: DbActor, maintenance: MaintenanceCoordinator) -> Self {
        Self {
            actor,
            maintenance,
            recovery: RecoveryStatus::default(),
        }
    }
    pub fn with_recovery(
        actor: DbActor,
        maintenance: MaintenanceCoordinator,
        recovery: RecoveryStatus,
    ) -> Self {
        Self {
            actor,
            maintenance,
            recovery,
        }
    }
}
impl SettingsQuery for ActorSettingsQuery {
    fn library_info(&self) -> LibrarySummary {
        let info = self.actor.info();
        LibrarySummary {
            library_id: info.library_id.0.clone(),
            display_name: info.display_name.clone(),
            root_label: info.root_label.clone(),
            schema_version: info.schema_version,
            writable: info.writable,
        }
    }
    fn library_status(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<LibraryStatus, AppError>> + Send + '_>> {
        Box::pin(async {
            let pending_operations = if self.recovery.failure_code().is_some() {
                // Keep diagnostics available even when enumeration itself failed. The shared
                // recovery state is authoritative until a later successful reconciliation.
                true
            } else if self.actor.info().writable {
                self.actor.submit(|c|Ok(c.query_row("SELECT EXISTS(SELECT 1 FROM import_operations WHERE state!='COMMITTED' AND NOT(state='FAILED' AND json_extract(error_json,'$.cleanup')='DONE'))",[],|r|r.get::<_,bool>(0))?)).await?
            } else {
                true
            };
            Ok(LibraryStatus {
                writable: self.actor.info().writable,
                active_operations: self.maintenance.active_operations() as i64,
                recovery_required: pending_operations || self.recovery.required(),
            })
        })
    }
}
