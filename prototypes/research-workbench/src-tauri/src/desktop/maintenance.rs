use crate::transport::error::{AppError, ErrorCode};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
#[derive(Default)]
struct State {
    maintenance: bool,
    closing: bool,
    active: usize,
}
#[derive(Clone, Default)]
pub struct MaintenanceCoordinator {
    state: Arc<(Mutex<State>, Condvar)>,
}
pub struct OperationPermit {
    state: Arc<(Mutex<State>, Condvar)>,
    maintenance: bool,
}
impl MaintenanceCoordinator {
    pub fn begin_operation(&self) -> Result<OperationPermit, AppError> {
        let mut s = self
            .state
            .0
            .lock()
            .map_err(|_| AppError::new(ErrorCode::Busy))?;
        if s.maintenance || s.closing {
            return Err(AppError::new(ErrorCode::Busy));
        }
        s.active += 1;
        Ok(OperationPermit {
            state: self.state.clone(),
            maintenance: false,
        })
    }
    pub fn begin_maintenance(&self) -> Result<OperationPermit, AppError> {
        let mut s = self
            .state
            .0
            .lock()
            .map_err(|_| AppError::new(ErrorCode::Busy))?;
        if s.maintenance || s.closing || s.active != 0 {
            return Err(AppError::new(ErrorCode::Busy));
        }
        s.maintenance = true;
        Ok(OperationPermit {
            state: self.state.clone(),
            maintenance: true,
        })
    }
    pub fn close(&self) {
        if let Ok(mut s) = self.state.0.lock() {
            s.closing = true;
            self.state.1.notify_all();
        }
    }
    pub fn active_operations(&self) -> usize {
        self.state.0.lock().map(|s| s.active).unwrap_or(0)
    }
    pub fn wait_for_idle(&self, timeout: Duration) -> Result<(), AppError> {
        let state = self
            .state
            .0
            .lock()
            .map_err(|_| AppError::new(ErrorCode::Busy))?;
        let (state, result) = self
            .state
            .1
            .wait_timeout_while(state, timeout, |s| s.active != 0 || s.maintenance)
            .map_err(|_| AppError::new(ErrorCode::Busy))?;
        if result.timed_out() && (state.active != 0 || state.maintenance) {
            Err(AppError::new(ErrorCode::Busy))
        } else {
            Ok(())
        }
    }
}
impl Drop for OperationPermit {
    fn drop(&mut self) {
        if let Ok(mut s) = self.state.0.lock() {
            if self.maintenance {
                s.maintenance = false;
            } else {
                s.active = s.active.saturating_sub(1);
            }
            self.state.1.notify_all();
        }
    }
}
