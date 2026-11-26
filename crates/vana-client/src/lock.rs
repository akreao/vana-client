//! Locks that outlive a panic. A thread that panics while holding a lock poisons it, and `lock().unwrap()` then
//! turns that one thread's panic (a worker's) into a crash of the whole client on the next lock. These take the
//! guard either way: the data is what the panicking thread left, as with an unpoisoned lock.

use std::sync::{Mutex, MutexGuard, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub trait Locked<T: ?Sized> {
    fn locked(&self) -> MutexGuard<'_, T>;
}

impl<T: ?Sized> Locked<T> for Mutex<T> {
    fn locked(&self) -> MutexGuard<'_, T> {
        self.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

pub trait RwLocked<T: ?Sized> {
    fn read_locked(&self) -> RwLockReadGuard<'_, T>;
    fn write_locked(&self) -> RwLockWriteGuard<'_, T>;
}

impl<T: ?Sized> RwLocked<T> for RwLock<T> {
    fn read_locked(&self) -> RwLockReadGuard<'_, T> {
        self.read().unwrap_or_else(PoisonError::into_inner)
    }

    fn write_locked(&self) -> RwLockWriteGuard<'_, T> {
        self.write().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// A worker that panics holding the lock: the next lock still gets the data instead of panicking.
    #[test]
    fn poisoned_locks_still_lock() {
        let m = Arc::new(Mutex::new(1));
        let r = Arc::new(RwLock::new(2));
        let (m2, r2) = (m.clone(), r.clone());
        let _ = std::thread::spawn(move || {
            let _g = m2.lock().unwrap();
            let _w = r2.write().unwrap();
            panic!("worker");
        })
        .join();
        assert!(m.is_poisoned() && r.is_poisoned());
        *m.locked() += 1;
        assert_eq!(*m.locked(), 2);
        *r.write_locked() += 1;
        assert_eq!(*r.read_locked(), 3);
    }
}
