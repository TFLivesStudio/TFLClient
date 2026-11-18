//! Shared admission gate for updater installation and critical operations.
use std::sync::{Arc, LazyLock};
use tokio::sync::{OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};

static GATE: LazyLock<Arc<RwLock<()>>> = LazyLock::new(|| Arc::new(RwLock::new(())));

pub fn operation() -> Result<OwnedRwLockReadGuard<()>, String> {
    GATE.clone()
        .try_read_owned()
        .map_err(|_| "Se está instalando una actualización. Espera a que termine.".into())
}

pub fn install() -> Result<OwnedRwLockWriteGuard<()>, String> {
    GATE.clone()
        .try_write_owned()
        .map_err(|_| "Espera a que terminen las tareas activas antes de actualizar.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "Runs separately from parallel operations"]
    fn installation_and_new_operations_exclude_each_other() {
        let active = operation().unwrap();
        assert!(install().is_err());
        drop(active);
        let installing = install().unwrap();
        assert!(operation().is_err());
        drop(installing);
        assert!(operation().is_ok());
    }
}
