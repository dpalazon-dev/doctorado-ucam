use research_workbench_core::{
    adapters::{
        sqlite::actor::DbActor,
        windows::{lock::LibraryLock, paths::LibraryRoot},
    },
    desktop::{
        lifecycle::{DesktopState, ExitCoordinator, SafeLogger},
        maintenance::MaintenanceCoordinator,
    },
};
use std::{
    sync::{Arc, mpsc},
    time::Duration,
};
#[test]
fn desktop_close_keeps_exit_and_lock_until_admitted_jobs_finish() {
    let dir = tempfile::tempdir().unwrap();
    let root = LibraryRoot::at(dir.path().join("library"));
    let actor = DbActor::start(root.clone()).unwrap();
    let maintenance = MaintenanceCoordinator::default();
    let state = DesktopState {
        settings: Arc::new(
            research_workbench_core::adapters::sqlite::settings::ActorSettingsQuery::new(
                actor.clone(),
                maintenance.clone(),
            ),
        ),
        actor: actor.clone(),
        maintenance,
        logger: Arc::new(SafeLogger::new(dir.path().join("logs")).unwrap()),
        exit: ExitCoordinator::new(Duration::from_millis(1), 1),
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
    let second = actor.submit(|c| {
        c.execute(
            "INSERT INTO app_settings VALUES('close-test','\"done\"')",
            [],
        )?;
        Ok(())
    });
    let (ready_tx, ready_rx) = mpsc::channel();
    let (waiting_tx, waiting_rx) = mpsc::channel();
    assert!(!state.request_exit(
        move || {
            ready_tx.send(()).unwrap();
        },
        move |_| {
            waiting_tx.send(()).unwrap();
        }
    ));
    waiting_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(ready_rx.try_recv().is_err());
    assert!(LibraryLock::acquire(root.path()).is_err());
    assert!(state.maintenance.begin_operation().is_err());
    release_tx.send(()).unwrap();
    ready_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    tauri::async_runtime::block_on(blocked).unwrap();
    tauri::async_runtime::block_on(second).unwrap();
    assert!(state.request_exit(|| panic!("already closed"), |_| panic!("already closed")));
    let lock = LibraryLock::acquire(root.path()).unwrap();
    drop(lock);
    let c = rusqlite::Connection::open(root.database()).unwrap();
    assert_eq!(
        c.query_row(
            "SELECT value_json FROM app_settings WHERE key='close-test'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "\"done\""
    );
}

fn state_for(root: &LibraryRoot, dir: &tempfile::TempDir) -> DesktopState {
    let actor = DbActor::start(root.clone()).unwrap();
    let maintenance = MaintenanceCoordinator::default();
    DesktopState {
        settings: Arc::new(
            research_workbench_core::adapters::sqlite::settings::ActorSettingsQuery::new(
                actor.clone(),
                maintenance.clone(),
            ),
        ),
        actor,
        maintenance,
        logger: Arc::new(SafeLogger::new(dir.path().join("logs")).unwrap()),
        exit: ExitCoordinator::new(Duration::from_millis(1), 1),
        library: std::sync::Mutex::new(None),
        reader: std::sync::Mutex::new(None),
        workflow: std::sync::Mutex::new(None),
    }
}

#[test]
fn desktop_close_waits_for_admitted_filesystem_operation() {
    let dir = tempfile::tempdir().unwrap();
    let root = LibraryRoot::at(dir.path().join("library"));
    let state = state_for(&root, &dir);
    let permit = state.maintenance.begin_operation().unwrap();
    let (work_started_tx, work_started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        work_started_tx.send(()).unwrap();
        release_rx.recv().unwrap();
        drop(permit);
    });
    work_started_rx
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    let (ready_tx, ready_rx) = mpsc::channel();
    assert!(!state.request_exit(move || ready_tx.send(()).unwrap(), |_| {}));
    assert!(ready_rx.recv_timeout(Duration::from_millis(40)).is_err());
    assert!(LibraryLock::acquire(root.path()).is_err());
    release_tx.send(()).unwrap();
    ready_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    worker.join().unwrap();
}

#[test]
fn desktop_close_waits_for_recovery_maintenance() {
    let dir = tempfile::tempdir().unwrap();
    let root = LibraryRoot::at(dir.path().join("library"));
    let state = state_for(&root, &dir);
    let permit = state.maintenance.begin_maintenance().unwrap();
    let (work_started_tx, work_started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        work_started_tx.send(()).unwrap();
        release_rx.recv().unwrap();
        drop(permit);
    });
    work_started_rx
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    let (ready_tx, ready_rx) = mpsc::channel();
    assert!(!state.request_exit(move || ready_tx.send(()).unwrap(), |_| {}));
    assert!(ready_rx.recv_timeout(Duration::from_millis(40)).is_err());
    assert!(LibraryLock::acquire(root.path()).is_err());
    release_tx.send(()).unwrap();
    ready_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    worker.join().unwrap();
}
