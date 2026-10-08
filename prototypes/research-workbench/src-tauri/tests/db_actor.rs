use research_workbench_core::{
    adapters::{
        sqlite::{actor::DbActor, receipts::with_receipt},
        windows::paths::LibraryRoot,
    },
    application::unit_of_work::with_transaction,
    transport::error::{AppError, ErrorCode},
};
use serde_json::json;
#[tokio::test]
async fn transaction_failure_rolls_back() {
    let dir = tempfile::tempdir().unwrap();
    let actor = DbActor::start(LibraryRoot::at(dir.path().join("library"))).unwrap();
    let result: Result<(), AppError> = actor
        .submit(|c| {
            with_transaction(c, |t| {
                t.execute(
                    "INSERT INTO app_settings(key,value_json) VALUES('rollback','1')",
                    [],
                )?;
                Err(AppError::new(ErrorCode::InvalidInput))
            })
        })
        .await;
    assert!(result.is_err());
    assert_eq!(
        actor
            .submit(|c| Ok(c.query_row(
                "SELECT count(*) FROM app_settings WHERE key='rollback'",
                [],
                |r| r.get::<_, u32>(0)
            )?))
            .await
            .unwrap(),
        0
    );
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
}
#[tokio::test]
async fn receipt_retry_returns_previous_result() {
    let dir = tempfile::tempdir().unwrap();
    let actor = DbActor::start(LibraryRoot::at(dir.path().join("library"))).unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let id2 = id.clone();
    let first = actor
        .submit(move |c| {
            with_receipt(c, &id, "test_change", &json!({"b":2,"a":1}), |t| {
                t.execute("INSERT INTO app_settings VALUES('once','1')", [])?;
                Ok(json!({"revision":1}))
            })
        })
        .await
        .unwrap();
    let second = actor
        .submit(move |c| {
            with_receipt(c, &id2, "test_change", &json!({"a":1,"b":2}), |_| {
                panic!("must not replay mutation")
            })
        })
        .await
        .unwrap();
    assert_eq!(first, second);
}
#[tokio::test]
async fn receipt_payload_mismatch_conflict() {
    let dir = tempfile::tempdir().unwrap();
    let actor = DbActor::start(LibraryRoot::at(dir.path().join("library"))).unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let other = id.clone();
    actor
        .submit(move |c| with_receipt(c, &id, "test_change", &json!({"n":1}), |_| Ok(json!(1))))
        .await
        .unwrap();
    let error = actor
        .submit(move |c| with_receipt(c, &other, "test_change", &json!({"n":2}), |_| Ok(json!(2))))
        .await
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::Conflict);
}
#[tokio::test]
async fn db_actor_capacity_64_returns_busy() {
    let dir = tempfile::tempdir().unwrap();
    let actor = DbActor::start(LibraryRoot::at(dir.path().join("library"))).unwrap();
    let (start_tx, start_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let first = actor.submit(move |_| {
        start_tx.send(()).unwrap();
        release_rx.recv().unwrap();
        Ok(())
    });
    start_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    let pending: Vec<_> = (0..64).map(|_| actor.submit(|_| Ok(()))).collect();
    let busy = actor.submit(|_| Ok(())).await.unwrap_err();
    assert_eq!(busy.code, ErrorCode::Busy);
    release_tx.send(()).unwrap();
    first.await.unwrap();
    for p in pending {
        p.await.unwrap();
    }
}
#[tokio::test]
async fn db_actor_owns_one_connection() {
    let dir = tempfile::tempdir().unwrap();
    let actor = DbActor::start(LibraryRoot::at(dir.path().join("library"))).unwrap();
    let first = actor
        .submit(|c| {
            c.execute_batch(
                "CREATE TEMP TABLE thread_marker(n INTEGER);INSERT INTO thread_marker VALUES(7);",
            )?;
            Ok(std::thread::current().id())
        })
        .await
        .unwrap();
    let second = actor
        .submit(|c| {
            assert_eq!(
                c.query_row("SELECT n FROM thread_marker", [], |r| r.get::<_, i32>(0))?,
                7
            );
            Ok(std::thread::current().id())
        })
        .await
        .unwrap();
    assert_eq!(first, second);
    assert_ne!(first, std::thread::current().id());
}

#[tokio::test]
async fn panic_cannot_leave_open_transaction() {
    let d = tempfile::tempdir().unwrap();
    let actor = DbActor::start(LibraryRoot::at(d.path().join("library"))).unwrap();
    let result = actor
        .submit::<(), _>(|c| {
            c.execute_batch("BEGIN IMMEDIATE; INSERT INTO app_settings VALUES('panic','1');")?;
            panic!("injected job failure");
        })
        .await;
    assert!(result.is_err());
    assert_eq!(
        actor
            .submit(|c| Ok(c.query_row(
                "SELECT count(*) FROM app_settings WHERE key='panic'",
                [],
                |r| r.get::<_, i64>(0)
            )?))
            .await
            .unwrap(),
        0
    );
}
#[tokio::test]
async fn shutdown_timeout_keeps_library_lock() {
    let d = tempfile::tempdir().unwrap();
    let root = LibraryRoot::at(d.path().join("library"));
    let actor = DbActor::start(root.clone()).unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let (start, started) = std::sync::mpsc::channel();
    let job = actor.submit(move |_| {
        start.send(()).unwrap();
        rx.recv().unwrap();
        Ok(())
    });
    started.recv().unwrap();
    assert_eq!(
        actor
            .shutdown(std::time::Duration::from_millis(20))
            .unwrap_err()
            .code,
        ErrorCode::Busy
    );
    assert!(
        research_workbench_core::adapters::windows::lock::LibraryLock::acquire(root.path())
            .is_err()
    );
    tx.send(()).unwrap();
    job.await.unwrap();
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
    assert!(
        research_workbench_core::adapters::windows::lock::LibraryLock::acquire(root.path()).is_ok()
    );
}

#[tokio::test]
async fn relative_database_pin_keeps_wal_database_writable_across_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let root = LibraryRoot::at(dir.path().join("library"));
    let actor = DbActor::start(root.clone()).unwrap();
    assert!(actor.info().writable);
    assert!(root.path().join(".rw-directory-pin").is_file());
    let mode = actor
        .submit(|connection| {
            let mode: String = connection.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
            connection.execute(
                "INSERT INTO app_settings(key,value_json) VALUES('managed-pin-wal','true')",
                [],
            )?;
            let _: (i32, i32, i32) =
                connection.query_row("PRAGMA wal_checkpoint(FULL)", [], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })?;
            Ok(mode)
        })
        .await
        .unwrap();
    assert_eq!(mode.to_ascii_lowercase(), "wal");
    actor.shutdown(std::time::Duration::from_secs(5)).unwrap();
    drop(actor);

    let reopened = DbActor::start(root).unwrap();
    let value = reopened
        .submit(|connection| {
            Ok(connection.query_row(
                "SELECT value_json FROM app_settings WHERE key='managed-pin-wal'",
                [],
                |row| row.get::<_, String>(0),
            )?)
        })
        .await
        .unwrap();
    assert_eq!(value, "true");
    reopened
        .shutdown(std::time::Duration::from_secs(5))
        .unwrap();
}
