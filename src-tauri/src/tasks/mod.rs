use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Manages cancellation tokens for in-progress scans.
#[derive(Default)]
pub struct ScanManager {
    tokens: Mutex<HashMap<i64, Arc<AtomicBool>>>,
}

impl ScanManager {
    /// Creates a new, empty manager.
    pub fn new() -> Self {
        Self {
            tokens: Mutex::new(HashMap::new()),
        }
    }

    /// Registers a new scan run and returns the cancellation token for it.
    pub fn register(&self, run_id: i64) -> Arc<AtomicBool> {
        let token = Arc::new(AtomicBool::new(false));
        let mut tokens = self.tokens.lock().expect("scan manager lock");
        tokens.insert(run_id, token.clone());
        token
    }

    /// Requests cancellation for the given run id. Returns `true` if the run
    /// was known (and therefore cancelled), `false` otherwise.
    pub fn cancel(&self, run_id: i64) -> bool {
        let tokens = self.tokens.lock().expect("scan manager lock");
        match tokens.get(&run_id) {
            Some(token) => {
                token.store(true, Ordering::Relaxed);
                true
            }
            None => false,
        }
    }

    /// Removes a run from the manager after it finishes.
    pub fn remove(&self, run_id: i64) {
        let mut tokens = self.tokens.lock().expect("scan manager lock");
        tokens.remove(&run_id);
    }
}

/// Manages cancellation tokens for in-progress analysis runs.
#[derive(Default)]
pub struct AnalysisManager {
    tokens: Mutex<HashMap<i64, Arc<AtomicBool>>>,
}

impl AnalysisManager {
    pub fn new() -> Self {
        Self {
            tokens: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, workspace_id: i64) -> Arc<AtomicBool> {
        let token = Arc::new(AtomicBool::new(false));
        let mut tokens = self.tokens.lock().expect("analysis manager lock");
        tokens.insert(workspace_id, token.clone());
        token
    }

    pub fn cancel(&self, workspace_id: i64) -> bool {
        let tokens = self.tokens.lock().expect("analysis manager lock");
        match tokens.get(&workspace_id) {
            Some(token) => {
                token.store(true, Ordering::Relaxed);
                true
            }
            None => false,
        }
    }

    pub fn is_running(&self, workspace_id: i64) -> bool {
        let tokens = self.tokens.lock().expect("analysis manager lock");
        tokens.contains_key(&workspace_id)
    }

    pub fn remove(&self, workspace_id: i64) {
        let mut tokens = self.tokens.lock().expect("analysis manager lock");
        tokens.remove(&workspace_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_cancel() {
        let manager = ScanManager::new();
        let token = manager.register(1);
        assert!(!token.load(Ordering::Relaxed));
        assert!(manager.cancel(1));
        assert!(token.load(Ordering::Relaxed));
    }

    #[test]
    fn cancel_unknown_run_returns_false() {
        let manager = ScanManager::new();
        assert!(!manager.cancel(99));
    }
}
