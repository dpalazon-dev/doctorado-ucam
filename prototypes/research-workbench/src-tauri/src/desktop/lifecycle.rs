use super::maintenance::MaintenanceCoordinator;
use crate::{
    adapters::{sqlite::actor::DbActor, windows::paths::resolve_data_root},
    transport::error::AppError,
};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
#[derive(Clone, Copy)]
pub enum LogEvent {
    Started,
    Stopped,
    Diagnostic,
    ShutdownBusy,
}
impl LogEvent {
    fn label(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Stopped => "stopped",
            Self::Diagnostic => "diagnostic",
            Self::ShutdownBusy => "shutdown_busy",
        }
    }
}
/// Closed structured logger: no source paths, scientific text, arbitrary messages or DTOs accepted.
pub struct SafeLogger {
    root: PathBuf,
    gate: Mutex<()>,
}
impl SafeLogger {
    pub fn new(root: PathBuf) -> Result<Self, AppError> {
        fs::create_dir_all(&root)?;
        Ok(Self {
            root,
            gate: Mutex::new(()),
        })
    }
    pub fn record(&self, event: LogEvent) -> Result<(), AppError> {
        let _guard = self.gate.lock().map_err(|_| {
            crate::transport::error::AppError::new(
                crate::transport::error::ErrorCode::StorageUnavailable,
            )
        })?;
        let current = self.root.join("desktop.log");
        if fs::metadata(&current)
            .map(|m| m.len() > 65536)
            .unwrap_or(false)
        {
            for n in (1..=2).rev() {
                let src = self.root.join(format!("desktop.{n}.log"));
                let target = self.root.join(format!("desktop.{}.log", n + 1));
                if target.exists() {
                    fs::remove_file(&target)?;
                }
                if src.exists() {
                    fs::rename(src, target)?;
                }
            }
            fs::rename(&current, self.root.join("desktop.1.log"))?;
        }
        let mut file = OpenOptions::new().create(true).append(true).open(current)?;
        writeln!(
            file,
            "{} {}",
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            event.label()
        )?;
        Ok(())
    }
}
pub struct DesktopState {
    pub settings: std::sync::Arc<dyn crate::application::settings::SettingsQuery>,
    pub actor: DbActor,
    pub maintenance: MaintenanceCoordinator,
    pub logger: Arc<SafeLogger>,
    pub exit: ExitCoordinator,
    pub library: std::sync::Mutex<Option<Arc<crate::modules::library::service::LibraryService>>>,
    pub reader: std::sync::Mutex<Option<Arc<crate::modules::reader::service::ReaderService>>>,
    pub workflow: std::sync::Mutex<Option<Arc<crate::modules::workflow::service::WorkflowService>>>,
}
impl DesktopState {
    pub fn start(
        settings_factory: impl FnOnce(
            DbActor,
            MaintenanceCoordinator,
        )
            -> std::sync::Arc<dyn crate::application::settings::SettingsQuery>,
    ) -> Result<Self, AppError> {
        let root = resolve_data_root()?;
        let log_root = root.logs()?;
        let actor = DbActor::start(root)?;
        let logger = SafeLogger::new(log_root)?;
        logger.record(if actor.info().writable {
            LogEvent::Started
        } else {
            LogEvent::Diagnostic
        })?;
        let maintenance = MaintenanceCoordinator::default();
        let settings = settings_factory(actor.clone(), maintenance.clone());
        Ok(Self {
            settings,
            actor,
            maintenance,
            logger: Arc::new(logger),
            exit: ExitCoordinator::default(),
            library: std::sync::Mutex::new(None),
            reader: std::sync::Mutex::new(None),
            workflow: std::sync::Mutex::new(None),
        })
    }
    pub fn register_library(
        &self,
        library: Arc<crate::modules::library::service::LibraryService>,
    ) -> Result<(), AppError> {
        *self
            .library
            .lock()
            .map_err(|_| AppError::new(crate::transport::error::ErrorCode::StorageUnavailable))? =
            Some(library);
        Ok(())
    }
    pub fn library(
        &self,
    ) -> Result<Arc<crate::modules::library::service::LibraryService>, AppError> {
        self.library
            .lock()
            .map_err(|_| AppError::new(crate::transport::error::ErrorCode::StorageUnavailable))?
            .clone()
            .ok_or_else(|| AppError::new(crate::transport::error::ErrorCode::Busy))
    }
    pub fn register_reader(
        &self,
        reader: Arc<crate::modules::reader::service::ReaderService>,
    ) -> Result<(), AppError> {
        *self
            .reader
            .lock()
            .map_err(|_| AppError::new(crate::transport::error::ErrorCode::StorageUnavailable))? =
            Some(reader);
        Ok(())
    }
    pub fn reader(&self) -> Result<Arc<crate::modules::reader::service::ReaderService>, AppError> {
        self.reader
            .lock()
            .map_err(|_| AppError::new(crate::transport::error::ErrorCode::StorageUnavailable))?
            .clone()
            .ok_or_else(|| AppError::new(crate::transport::error::ErrorCode::Busy))
    }
    pub fn register_workflow(
        &self,
        workflow: Arc<crate::modules::workflow::service::WorkflowService>,
    ) -> Result<(), AppError> {
        *self
            .workflow
            .lock()
            .map_err(|_| AppError::new(crate::transport::error::ErrorCode::StorageUnavailable))? =
            Some(workflow);
        Ok(())
    }
    pub fn workflow(
        &self,
    ) -> Result<Arc<crate::modules::workflow::service::WorkflowService>, AppError> {
        self.workflow
            .lock()
            .map_err(|_| AppError::new(crate::transport::error::ErrorCode::StorageUnavailable))?
            .clone()
            .ok_or_else(|| AppError::new(crate::transport::error::ErrorCode::Busy))
    }
    pub fn shutdown(&self) -> Result<(), AppError> {
        self.maintenance.close();
        self.maintenance.wait_for_idle(Duration::from_secs(5))?;
        let result = self.actor.shutdown(Duration::from_secs(5));
        let _ = self.logger.record(if result.is_ok() {
            LogEvent::Stopped
        } else {
            LogEvent::ShutdownBusy
        });
        result
    }
    /// False means the event loop must prevent exit. A worker drains the actor,
    /// retaining the state/lock while each bounded wait expires; never force exit.
    pub fn request_exit(
        &self,
        on_ready: impl FnOnce() + Send + 'static,
        on_waiting: impl Fn(AppError) + Send + 'static,
    ) -> bool {
        self.request_exit_with_spawn(on_ready, on_waiting, |job| {
            std::thread::Builder::new()
                .name("research-close".into())
                .spawn(job)
        })
    }
    fn request_exit_with_spawn(
        &self,
        on_ready: impl FnOnce() + Send + 'static,
        on_waiting: impl Fn(AppError) + Send + 'static,
        spawn: impl FnOnce(
            Box<dyn FnOnce() + Send + 'static>,
        ) -> std::io::Result<std::thread::JoinHandle<()>>,
    ) -> bool {
        if self.exit.ready.load(Ordering::Acquire) {
            return true;
        }
        if self.exit.running.swap(true, Ordering::AcqRel) {
            return false;
        }
        self.maintenance.close();
        let actor = self.actor.clone();
        let maintenance = self.maintenance.clone();
        let logger = self.logger.clone();
        let exit = self.exit.clone();
        // The callback remains available if the OS rejects thread creation.
        // Mutex preserves the public Send bound without requiring a Sync callback.
        let diagnostic = Arc::new(Mutex::new(on_waiting));
        let worker_diagnostic = diagnostic.clone();
        let started = spawn(Box::new(move || {
            let mut attempts = 0;
            let mut notified = false;
            loop {
                if let Err(error) = maintenance.wait_for_idle(exit.attempt_timeout) {
                    attempts += 1;
                    if attempts >= exit.diagnostic_after && !notified {
                        notified = true;
                        let _ = logger.record(LogEvent::ShutdownBusy);
                        if let Ok(callback) = worker_diagnostic.lock() {
                            callback(error);
                        }
                    }
                    continue;
                }
                match actor.shutdown(exit.attempt_timeout) {
                    Ok(()) => {
                        let _ = logger.record(LogEvent::Stopped);
                        exit.ready.store(true, Ordering::Release);
                        on_ready();
                        return;
                    }
                    Err(error) if error.code == crate::transport::error::ErrorCode::Busy => {
                        attempts += 1;
                        if attempts >= exit.diagnostic_after && !notified {
                            notified = true;
                            let _ = logger.record(LogEvent::ShutdownBusy);
                            if let Ok(callback) = worker_diagnostic.lock() {
                                callback(error);
                            }
                        }
                    }
                    Err(error) => {
                        let _ = logger.record(LogEvent::ShutdownBusy);
                        if let Ok(callback) = worker_diagnostic.lock() {
                            callback(error);
                        }
                        exit.running.store(false, Ordering::Release);
                        return;
                    }
                }
            }
        }));
        if started.is_err() {
            self.exit.running.store(false, Ordering::Release);
            let _ = self.logger.record(LogEvent::ShutdownBusy);
            if let Ok(callback) = diagnostic.lock() {
                callback(AppError::new(
                    crate::transport::error::ErrorCode::StorageUnavailable,
                ));
            }
        }
        false
    }
}
#[derive(Clone)]
pub struct ExitCoordinator {
    ready: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    attempt_timeout: Duration,
    diagnostic_after: usize,
}
impl ExitCoordinator {
    pub fn new(attempt_timeout: Duration, diagnostic_after: usize) -> Self {
        Self {
            ready: Arc::new(AtomicBool::new(false)),
            running: Arc::new(AtomicBool::new(false)),
            attempt_timeout,
            diagnostic_after,
        }
    }
}
impl Default for ExitCoordinator {
    fn default() -> Self {
        Self::new(Duration::from_millis(250), 20)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::{
        sqlite::settings::ActorSettingsQuery,
        windows::{lock::LibraryLock, paths::LibraryRoot},
    };
    use std::sync::mpsc;
    #[test]
    fn failed_drainer_creation_preserves_exit_lock_and_allows_retry() {
        let dir = tempfile::tempdir().unwrap();
        let root = LibraryRoot::at(dir.path().join("library"));
        let actor = DbActor::start(root.clone()).unwrap();
        let maintenance = MaintenanceCoordinator::default();
        let state = DesktopState {
            settings: Arc::new(ActorSettingsQuery::new(actor.clone(), maintenance.clone())),
            actor: actor.clone(),
            maintenance,
            logger: Arc::new(SafeLogger::new(dir.path().join("logs")).unwrap()),
            exit: ExitCoordinator::new(Duration::from_secs(5), 1),
            library: std::sync::Mutex::new(None),
            reader: std::sync::Mutex::new(None),
            workflow: std::sync::Mutex::new(None),
        };
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let blocked = actor.submit(move |_| {
            started_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            Ok(())
        });
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let queued = actor.submit(|c| {
            c.execute("INSERT INTO app_settings VALUES('spawn-test','true')", [])?;
            Ok(())
        });
        let (failure_tx, failure_rx) = mpsc::channel();
        let first = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.request_exit_with_spawn(
                || panic!("failed spawn must not approve exit"),
                move |error| {
                    failure_tx.send(error.code).unwrap();
                },
                |_job| Err(std::io::Error::other("synthetic private OS message")),
            )
        }));
        assert!(first.is_ok(), "thread creation failure must not panic");
        assert!(!first.unwrap());
        assert_eq!(
            failure_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            crate::transport::error::ErrorCode::StorageUnavailable
        );
        assert!(!state.exit.ready.load(Ordering::Acquire));
        assert!(!state.exit.running.load(Ordering::Acquire));
        assert!(LibraryLock::acquire(root.path()).is_err());
        let (ready_tx, ready_rx) = mpsc::channel();
        assert!(!state.request_exit(
            move || {
                ready_tx.send(()).unwrap();
            },
            |_| panic!("retry should drain")
        ));
        assert!(ready_rx.try_recv().is_err());
        assert!(LibraryLock::acquire(root.path()).is_err());
        release_tx.send(()).unwrap();
        ready_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        tauri::async_runtime::block_on(blocked).unwrap();
        tauri::async_runtime::block_on(queued).unwrap();
        assert!(state.request_exit(|| panic!("already ready"), |_| panic!("already ready")));
        let lock = LibraryLock::acquire(root.path()).unwrap();
        drop(lock);
        let c = rusqlite::Connection::open(root.database()).unwrap();
        assert_eq!(
            c.query_row(
                "SELECT value_json FROM app_settings WHERE key='spawn-test'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "true"
        );
        assert!(
            !std::fs::read_to_string(dir.path().join("logs/desktop.log"))
                .unwrap()
                .contains("synthetic private")
        );
    }
}
