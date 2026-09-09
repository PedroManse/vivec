//! MIS.TER
//! Most Inefficient Save. Terribly Efficient Read.
//!
//! Based on left_right crate but with data replacement instead of modification

use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, AtomicU8, Ordering};

pub struct ReaderGuard<'mr, T> {
    mr: &'mr mut ReadHandler<T>,
    data_ptr: *const T,
}

impl<'mr, T> Deref for ReaderGuard<'mr, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data_ptr }
    }
}

// lock is odd = reading
// lock is even = not reading
pub struct ReadHandler<T> {
    shared_ptr: Arc<AtomicPtr<T>>,
    lock: Arc<AtomicU8>,
}

impl<T> ReadHandler<T> {
    pub fn hold(&mut self) -> ReaderGuard<'_, T> {
        self.lock.fetch_add(1, Ordering::SeqCst);
        let data_ptr = self.shared_ptr.load(Ordering::SeqCst);
        ReaderGuard { mr: self, data_ptr }
    }
}

impl<'mr, T> Drop for ReaderGuard<'mr, T> {
    fn drop(&mut self) {
        self.mr.lock.fetch_add(1, Ordering::SeqCst);
    }
}

pub struct WriteHandler<T> {
    shared_ptr: Arc<AtomicPtr<T>>,
    read_locks: Vec<Arc<AtomicU8>>,
}

impl<T> WriteHandler<T> {
    pub fn new(data: T) -> Self {
        let heap_ptr = Box::into_raw(Box::new(data));
        let shared_ptr = Arc::new(AtomicPtr::new(heap_ptr));
        let read_locks = Vec::new();
        Self {
            shared_ptr,
            read_locks,
        }
    }

    pub fn read_handler(&mut self) -> ReadHandler<T> {
        let lock = Arc::new(AtomicU8::new(0));
        self.read_locks.push(Arc::clone(&lock));
        ReadHandler {
            shared_ptr: Arc::clone(&self.shared_ptr),
            lock,
        }
    }

    pub fn update(&mut self, data: T) {
        let heap_ptr = Box::into_raw(Box::new(data));
        self.shared_ptr.store(heap_ptr, Ordering::SeqCst);
        eprintln!("data updated");
        self.wait();
        eprintln!("readers updated");
    }

    fn wait(&mut self) {
        // MAX => not yet tested
        // 0 => reading new
        // N => reading old
        let mut old_reading_number = vec![u8::MAX; self.read_locks.len()];
        let mut dirty = true;
        while dirty {
            dirty = false;
            for (idx, lock) in self.read_locks.iter().enumerate() {
                if old_reading_number[idx] == 0 {
                    continue;
                }

                let reading_number = lock.load(Ordering::SeqCst);

                if old_reading_number[idx] == u8::MAX {
                    old_reading_number[idx] = reading_number;
                }

                // set OK if even
                if reading_number % 2 == 0 {
                    old_reading_number[idx] = 0;
                    continue;
                }
                if old_reading_number[idx] < reading_number {
                    continue;
                }

                dirty = true;
                break;
            }
        }
    }
}
