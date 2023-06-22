use std::fmt::Debug;

/// This models the locked state of the wallet. Locking and unlocking
/// is restricted to the [`Self::lock()`] and [`Self::unlock()`] methods.
/// Optionally, a callback can be set to get notified whenever the locked
/// state changes.
pub struct WalletLock {
    is_locked: bool,
    update_callback: Option<Box<dyn Fn(bool) + Send + Sync>>,
}

impl WalletLock {
    pub fn new(is_locked: bool) -> Self {
        WalletLock {
            is_locked,
            update_callback: None,
        }
    }

    fn call_update_callback(&self) {
        if let Some(update_callback) = &self.update_callback {
            update_callback(self.is_locked)
        }
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    pub fn lock(&mut self) {
        if self.is_locked {
            return;
        }

        self.is_locked = true;
        self.call_update_callback();
    }

    pub fn unlock(&mut self) {
        if !self.is_locked {
            return;
        }

        self.is_locked = false;
        self.call_update_callback();
    }

    pub fn set_lock_callback<F>(&mut self, callback: F)
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.update_callback.replace(Box::new(callback));
    }

    pub fn clear_lock_callback(&mut self) {
        self.update_callback.take();
    }
}

impl Debug for WalletLock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalletLock")
            .field("is_locked", &self.is_locked)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    #[test]
    fn test_wallet_lock() {
        let callback_is_locked: Arc<Mutex<Option<bool>>> = Arc::new(Mutex::new(None));

        let mut lock = WalletLock::new(false);
        assert!(!lock.is_locked());

        lock.unlock();
        assert!(!lock.is_locked());

        lock.lock();
        assert!(lock.is_locked());

        lock.lock();
        assert!(lock.is_locked());

        let callback_is_locked_clone = Arc::clone(&callback_is_locked);
        lock.set_lock_callback(move |is_locked| *callback_is_locked_clone.lock().unwrap() = Some(is_locked));

        lock.lock();
        assert!(lock.is_locked());
        assert!(matches!(callback_is_locked.lock().unwrap().as_ref(), None));

        lock.unlock();
        assert!(!lock.is_locked());
        assert!(matches!(callback_is_locked.lock().unwrap().as_ref(), Some(false)));

        lock.lock();
        assert!(lock.is_locked());
        assert!(matches!(callback_is_locked.lock().unwrap().as_ref(), Some(true)));

        lock.clear_lock_callback();
        lock.unlock();
        assert!(!lock.is_locked());
        assert!(matches!(callback_is_locked.lock().unwrap().as_ref(), Some(true)));
    }
}
