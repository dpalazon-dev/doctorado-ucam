use super::migrations::{self, SCHEMA_VERSION};
use crate::{
    adapters::windows::{lock::LibraryLock, paths::LibraryRoot},
    transport::{
        dto::LibraryInfoDto,
        error::{AppError, ErrorCode},
    },
};
use rusqlite::{Connection, OptionalExtension};
use std::{
    fs,
    sync::{
        Arc, Mutex,
        mpsc::{self, SyncSender, TrySendError},
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use tokio::sync::oneshot;
type Job = Box<dyn FnOnce(&mut Connection) + Send + 'static>;
struct Inner {
    sender: Mutex<Option<SyncSender<Job>>>,
    done: Mutex<mpsc::Receiver<()>>,
    thread: Mutex<Option<JoinHandle<()>>>,
    info: LibraryInfoDto,
    root: LibraryRoot,
}
#[derive(Clone)]
pub struct DbActor {
    inner: Arc<Inner>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LibraryManifest {
    format_version: u8,
    library_id: String,
}
type PreparedActor = (
    Connection,
    Option<LibraryLock>,
    Option<tempfile::TempDir>,
    LibraryInfoDto,
    LibraryRoot,
);
fn prepare(mut root: LibraryRoot) -> Result<PreparedActor, AppError> {
    // Bind the root chain from retained handles first. The native relative open, not a path
    // existence check, determines whether the database is already present.
    root.create_and_bind()?;
    let existed = root.database_exists()?;
    let probe = if existed {
        Some(super::schema_probe::probe(&root)?)
    } else {
        None
    };
    let version = probe.as_ref().map(|probe| probe.version).unwrap_or(0);
    let future = version > SCHEMA_VERSION;
    let (mut c, lock, scratch) = if future {
        let probe = probe.ok_or_else(|| AppError::new(ErrorCode::IntegrityFailure))?;
        (probe.connection, None, probe.scratch)
    } else {
        let lock = LibraryLock::acquire(root.path())?;
        if let Some(probe) = &probe {
            probe.verify_unchanged(&root)?;
        } else {
            // This is deliberately after the single-writer lock. FILE_CREATE is atomic and a
            // collision means another actor won initialization; never open that file as SQLite.
            root.create_database_after_lock()?;
        }
        drop(probe);
        let connection = Connection::open(root.database())?;
        if migrations::schema_version(&connection)? != version {
            return Err(AppError::new(ErrorCode::Busy));
        }
        (connection, Some(lock), None)
    };
    c.busy_timeout(Duration::from_millis(5000))?;
    c.pragma_update(None, "foreign_keys", "ON")?;
    let mut writable = !future;
    if future {
        c.pragma_update(None, "query_only", true)?;
    }
    if !future {
        c.pragma_update(None, "journal_mode", "WAL")?;
        c.pragma_update(None, "synchronous", "FULL")?;
        if let Err(e) = migrations::migrate(&mut c, &root, migrations::MIGRATIONS) {
            if matches!(
                e.code,
                ErrorCode::MigrationFailed | ErrorCode::IntegrityFailure | ErrorCode::BackupFailed
            ) {
                writable = false;
                c.pragma_update(None, "query_only", true)?;
            } else {
                return Err(e);
            }
        }
    }
    let manifest_path = root.path().join("library.json");
    let db_id: Option<String> = c
        .query_row(
            "SELECT library_id FROM library_identity WHERE singleton=1",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?;
    let id = if manifest_path.exists() {
        let m: LibraryManifest = serde_json::from_slice(&fs::read(&manifest_path)?)
            .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?;
        if m.format_version != 1
            || uuid::Uuid::parse_str(&m.library_id)
                .map(|x| x.to_string() != m.library_id)
                .unwrap_or(true)
        {
            return Err(AppError::new(ErrorCode::IntegrityFailure));
        }
        m.library_id
    } else {
        if !writable {
            return Err(AppError::new(ErrorCode::IntegrityFailure));
        }
        db_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
    };

    match db_id {
        Some(stored) if stored != id => return Err(AppError::new(ErrorCode::IntegrityFailure)),
        None if writable => {
            c.execute("INSERT INTO library_identity VALUES(1,?1)", [&id])?;
        }
        None => return Err(AppError::new(ErrorCode::IntegrityFailure)),
        _ => {}
    }
    if writable {
        for name in ["documents", "staging", "recovery"] {
            fs::create_dir_all(root.path().join(name))?;
        }
        if !manifest_path.exists() {
            let bytes = serde_json::to_vec_pretty(&LibraryManifest {
                format_version: 1,
                library_id: id.clone(),
            })
            .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?;
            let temp = root.path().join("library.json.new");
            {
                use std::io::Write;
                let mut file = fs::File::create(&temp)?;
                file.write_all(&bytes)?;
                file.sync_all()?;
            }
            fs::rename(temp, manifest_path)?;
        }
    }
    migrations::verify_integrity(&c)?;
    let info = LibraryInfoDto {
        library_id: crate::transport::dto::UUID(id),
        display_name: "Biblioteca local".into(),
        root_label: root.display_label().into(),
        schema_version: migrations::schema_version(&c)?,
        writable,
    };
    Ok((c, lock, scratch, info, root))
}
impl DbActor {
    pub fn start(root: LibraryRoot) -> Result<Self, AppError> {
        let (sender, receiver) = mpsc::sync_channel::<Job>(64);
        let (start_tx, start_rx) = mpsc::sync_channel(1);
        let (done_tx, done_rx) = mpsc::channel();
        let handle = thread::Builder::new()
            .name("research-db".into())
            .spawn(move || {
                let prepared = prepare(root);
                match prepared {
                    Ok((mut connection, lock, scratch, info, root)) => {
                        let _ = start_tx.send(Ok((info, root)));
                        for job in receiver {
                            job(&mut connection);
                        }
                        if !connection.is_autocommit() {
                            let _ = connection.execute_batch("ROLLBACK");
                        }
                        let _ = connection.close();
                        drop(lock);
                        drop(scratch);
                    }
                    Err(error) => {
                        let _ = start_tx.send(Err(error));
                    }
                }
                let _ = done_tx.send(());
            })?;
        let (info, root) = start_rx
            .recv_timeout(Duration::from_secs(30))
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))??;
        Ok(Self {
            inner: Arc::new(Inner {
                sender: Mutex::new(Some(sender)),
                done: Mutex::new(done_rx),
                thread: Mutex::new(Some(handle)),
                info,
                root,
            }),
        })
    }
    pub fn info(&self) -> &LibraryInfoDto {
        &self.inner.info
    }
    pub fn library_root(&self) -> &LibraryRoot {
        &self.inner.root
    }
    /// Enqueue eagerly, before the returned future is polled. Saturation never blocks the caller.
    pub fn submit<
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, AppError> + Send + 'static,
    >(
        &self,
        job: F,
    ) -> impl std::future::Future<Output = Result<T, AppError>> + Send + use<T, F> {
        let (tx, rx) = oneshot::channel();
        let work: Job = Box::new(move |connection| {
            let mut result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| job(connection)))
                    .unwrap_or_else(|_| Err(AppError::new(ErrorCode::IntegrityFailure)));
            if !connection.is_autocommit() {
                let _ = connection.execute_batch("ROLLBACK");
                result = Err(AppError::new(ErrorCode::IntegrityFailure));
            }
            let _ = tx.send(result);
        });
        let error = match self.inner.sender.lock() {
            Ok(sender) => match sender.as_ref() {
                Some(sender) => sender.try_send(work).err().map(|e| match e {
                    TrySendError::Full(_) => AppError::new(ErrorCode::Busy),
                    TrySendError::Disconnected(_) => AppError::new(ErrorCode::StorageUnavailable),
                }),
                None => Some(AppError::new(ErrorCode::Busy)),
            },
            Err(_) => Some(AppError::new(ErrorCode::StorageUnavailable)),
        };
        async move {
            if let Some(error) = error {
                return Err(error);
            }
            rx.await
                .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
        }
    }
    /// On timeout the owning thread still holds connection and OS lock; neither is released early.
    pub fn shutdown(&self, timeout: Duration) -> Result<(), AppError> {
        self.inner
            .sender
            .lock()
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
            .take();
        if self
            .inner
            .thread
            .lock()
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
            .is_none()
        {
            return Ok(());
        }
        self.inner
            .done
            .lock()
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
            .recv_timeout(timeout)
            .map_err(|_| AppError::new(ErrorCode::Busy))?;
        if let Some(handle) = self
            .inner
            .thread
            .lock()
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
            .take()
        {
            handle
                .join()
                .map_err(|_| AppError::new(ErrorCode::IntegrityFailure))?;
        }
        Ok(())
    }
}
