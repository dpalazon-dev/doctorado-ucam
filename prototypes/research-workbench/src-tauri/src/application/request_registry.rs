use crate::transport::{
    dto::UUID,
    error::{AppError, ErrorCode},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};

#[derive(Clone, Default)]
pub struct RequestRegistry {
    inflight: Arc<Mutex<HashMap<String, Weak<tokio::sync::Mutex<()>>>>>,
}
pub struct RequestPermit {
    _guard: tokio::sync::OwnedMutexGuard<()>,
}

impl RequestRegistry {
    pub async fn acquire(&self, request_id: &UUID) -> Result<RequestPermit, AppError> {
        let lock = {
            let mut requests = self
                .inflight
                .lock()
                .map_err(|_| AppError::new(ErrorCode::Busy))?;
            requests.retain(|_, weak| weak.strong_count() > 0);
            requests
                .get(&request_id.0)
                .and_then(Weak::upgrade)
                .unwrap_or_else(|| {
                    let lock = Arc::new(tokio::sync::Mutex::new(()));
                    requests.insert(request_id.0.clone(), Arc::downgrade(&lock));
                    lock
                })
        };
        Ok(RequestPermit {
            _guard: lock.lock_owned().await,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn same_request_id_is_excluded_until_the_first_permit_drops() {
        let registry = RequestRegistry::default();
        let id = UUID("00000000-0000-4000-8000-000000000001".into());
        let first = registry.acquire(&id).await.unwrap();
        let second_registry = registry.clone();
        let second_id = id.clone();
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            let _ = started_tx.send(());
            second_registry.acquire(&second_id).await.unwrap()
        });
        started_rx.await.unwrap();
        tokio::task::yield_now().await;
        assert!(!task.is_finished());
        drop(first);
        let second = task.await.unwrap();
        drop(second);
    }
}
