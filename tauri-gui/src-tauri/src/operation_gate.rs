use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};

static ACTIVE_OPERATION: Mutex<Option<ActiveOperation>> = Mutex::new(None);
static NEXT_TOKEN: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, PartialEq, Eq)]
struct ActiveOperation {
    token: u64,
    name: String,
}

#[derive(Debug)]
pub struct OperationGuard {
    token: u64,
}

pub fn try_begin(name: impl Into<String>) -> Result<OperationGuard, String> {
    let name = name.into();
    let mut active = ACTIVE_OPERATION
        .lock()
        .map_err(|_| "OPERATION_BUSY: operation gate is unavailable".to_string())?;
    if let Some(operation) = active.as_ref() {
        return Err(format!("OPERATION_BUSY: {}", operation.name));
    }

    let token = NEXT_TOKEN.fetch_add(1, Ordering::Relaxed);
    *active = Some(ActiveOperation { token, name });
    Ok(OperationGuard { token })
}

#[cfg(test)]
pub fn active_operation() -> Option<String> {
    ACTIVE_OPERATION
        .lock()
        .ok()
        .and_then(|active| active.as_ref().map(|operation| operation.name.clone()))
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        let Ok(mut active) = ACTIVE_OPERATION.lock() else {
            return;
        };
        if active
            .as_ref()
            .is_some_and(|operation| operation.token == self.token)
        {
            *active = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_one_mutating_operation_runs_at_a_time() {
        let first = try_begin("configure_codex").expect("first operation should start");
        let error =
            try_begin("assistant_update_install").expect_err("second operation should be rejected");
        assert_eq!(error, "OPERATION_BUSY: configure_codex");
        assert_eq!(active_operation().as_deref(), Some("configure_codex"));

        drop(first);
        let second =
            try_begin("assistant_update_install").expect("gate should reopen after guard drop");
        assert_eq!(
            active_operation().as_deref(),
            Some("assistant_update_install")
        );
        drop(second);
        assert_eq!(active_operation(), None);
    }
}
