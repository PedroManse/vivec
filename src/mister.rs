//! MIS.TER
//! Most Inefficient Save. Terribly Efficient Read.
//!
//! Based on left_right crate but with data replacement instead of modification

use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, AtomicU8, Ordering};

pub enum ReclaimPtr<T> {
    Shared(T),
    Exclusive(T),
}

pub trait StoreShare<T>: Sized + 'static {
    fn into_ptr(self) -> *mut T;
    unsafe fn from_ptr(ptr: *mut T) -> ReclaimPtr<Self>;
    unsafe fn share_data_ptr(ptr: *mut T) -> *const T {
        ptr
    }
    unsafe fn drop_shared_data_ptr(_: *mut T) {}
}

impl<T: 'static> StoreShare<T> for Box<T> {
    fn into_ptr(self) -> *mut T {
        Box::into_raw(self)
    }
    unsafe fn from_ptr(ptr: *mut T) -> ReclaimPtr<Self> {
        unsafe { ReclaimPtr::Exclusive(Box::from_raw(ptr)) }
    }
}

impl<T: 'static> StoreShare<T> for Arc<T> {
    fn into_ptr(self) -> *mut T {
        Arc::into_raw(self) as *mut T
    }
    unsafe fn from_ptr(ptr: *mut T) -> ReclaimPtr<Self> {
        unsafe { ReclaimPtr::Shared(Arc::from_raw(ptr)) }
    }
    unsafe fn share_data_ptr(ptr: *mut T) -> *const T {
        unsafe { Arc::increment_strong_count(ptr) };
        ptr
    }
    unsafe fn drop_shared_data_ptr(ptr: *mut T) {
        unsafe { Arc::decrement_strong_count(ptr) };
    }
}

pub struct ReaderGuard<'mr, T, ST: StoreShare<T>> {
    mr: &'mr mut ReadHandler<T, ST>,
    share_ptr: *const T,
}

impl<'mr, T, ST: StoreShare<T>> Deref for ReaderGuard<'mr, T, ST> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.share_ptr }
    }
}

// lock is odd = reading
// lock is even = not reading
pub struct ReadHandler<T, ST> {
    shared_ptr: Arc<AtomicPtr<T>>,
    lock: Arc<AtomicU8>,
    store_method: PhantomData<ST>,
}

unsafe impl<T, ST> Send for ReadHandler<T, ST> where T: Sync {}

impl<T, ST: StoreShare<T>> ReadHandler<T, ST> {
    pub fn hold(&mut self) -> ReaderGuard<'_, T, ST> {
        self.lock.fetch_add(1, Ordering::SeqCst);
        let data_ptr = self.shared_ptr.load(Ordering::SeqCst);
        let data_ptr = unsafe { ST::share_data_ptr(data_ptr) };
        ReaderGuard {
            mr: self,
            share_ptr: data_ptr,
        }
    }
}

impl<'mr, T, ST: StoreShare<T>> Drop for ReaderGuard<'mr, T, ST> {
    fn drop(&mut self) {
        self.mr.lock.fetch_add(1, Ordering::SeqCst);
        unsafe { ST::drop_shared_data_ptr(self.share_ptr as *mut T) };
    }
}

pub struct WriteHandler<T, ST> {
    shared_ptr: Arc<AtomicPtr<T>>,
    read_locks: Vec<Arc<AtomicU8>>,
    store_method: PhantomData<ST>,
}

impl<T, ST: StoreShare<T>> WriteHandler<T, ST> {
    pub fn new(data: ST) -> Self {
        let shared_ptr = Arc::new(AtomicPtr::new(data.into_ptr()));
        let read_locks = Vec::new();
        Self {
            shared_ptr,
            read_locks,
            store_method: PhantomData,
        }
    }

    pub fn read_handler(&mut self) -> ReadHandler<T, ST> {
        let lock = Arc::new(AtomicU8::new(0));
        self.read_locks.push(Arc::clone(&lock));
        ReadHandler {
            shared_ptr: Arc::clone(&self.shared_ptr),
            lock,
            store_method: PhantomData,
        }
    }

    pub fn swap(&mut self, data: ST) -> ST {
        eprint!("\nUPDATED DATA POINTER!");
        let old_ptr = self.shared_ptr.swap(data.into_ptr(), Ordering::SeqCst);
        let reclamimed = unsafe { ST::from_ptr(old_ptr) };
        match reclamimed {
            ReclaimPtr::Exclusive(smart_poinetr) => {
                eprint!("\nMANUAL RECLAIM");
                eprint!("\n - WAIT FOR WORKERS:");
                self.wait();
                eprint!("\n - NOW RECLAIM:");
                smart_poinetr
            }
            ReclaimPtr::Shared(sp) => sp,
        }
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

#[cfg(test)]
mod test {
    use super::{StoreShare, WriteHandler};
    use std::fmt::Debug;
    use std::sync::Arc;
    use std::thread::JoinHandle;
    use std::time::Duration;

    #[derive(Debug, Clone)]
    pub struct Record(usize);
    impl Drop for Record {
        fn drop(&mut self) {
            eprint!("\ndrop(Record({}))", self.0);
        }
    }

    fn make_data<ST: StoreShare<Record>, F: FnMut(Record) -> ST>(f: F) -> Vec<ST> {
        [1, 2, 3].into_iter().map(Record).map(f).collect()
    }

    fn read_on<T: Debug + Sync + 'static, ST: StoreShare<T>>(
        wh: &mut WriteHandler<T, ST>,
        wait_before: std::time::Duration,
        wait_hold: std::time::Duration,
        thread_count: usize,
        read_count: usize,
    ) -> Vec<JoinHandle<()>> {
        let mut threads = Vec::new();
        for t in 1..=thread_count {
            let mut rh = wh.read_handler();
            let t = std::thread::spawn(move || {
                for r in 1..=read_count {
                    std::thread::sleep(wait_before);
                    let rl = rh.hold();
                    std::thread::sleep(wait_hold);
                    eprint!(" t{t}r{r}: {:?}", *rl);
                }
            });
            threads.push(t);
        }
        threads
    }

    #[test]
    fn many_reads_and_update() {
        let box_data = make_data(Box::new);
        _many_reads_and_update(box_data);
        eprint!("\n\n");
        let arc_data = make_data(Arc::new);
        _many_reads_and_update(arc_data);
    }

    #[test]
    fn swap_data_points() {
        _swap_data_points(Box::new(Record(1)), Box::new(Record(2)));
        eprint!("\n\n");
        //_swap_data_points(Arc::new(Record(1)), Arc::new(Record(2)));
    }

    fn _swap_data_points<ST: StoreShare<Record> + Debug>(data1: ST, mut data: ST) {
        let mut wh = super::WriteHandler::<_, ST>::new(data1);
        let _ = read_on(
            &mut wh,
            Duration::from_millis(0),
            Duration::from_millis(5),
            4,
            160,
        );
        let mut wait_sum = std::time::Duration::ZERO;
        for _ in 0..50 {
            let now = std::time::Instant::now();
            eprint!("\nWANT TO UPDATE WITH {:?}", data);
            data = wh.swap(data);
            let elapsed = now.elapsed();
            eprint!("\ntE: {:?}", elapsed);
            wait_sum += elapsed;
        }
        eprint!("\nTE: {:?}", wait_sum);
        //for t in threads {
        //    let _ = t.join();
        //}
    }

    fn _many_reads_and_update<ST: StoreShare<Record> + Debug>(mut data: Vec<ST>) {
        let first = data.remove(0);
        let mut wh = super::WriteHandler::<_, ST>::new(first);

        let threads = read_on(
            &mut wh,
            Duration::from_millis(1),
            Duration::from_millis(5),
            4,
            16,
        );

        std::thread::sleep(std::time::Duration::from_millis(4));
        for d in data {
            std::thread::sleep(std::time::Duration::from_millis(4));
            let now = std::time::Instant::now();
            eprint!("\nWANT TO UPDATE WITH {:?}", d);
            wh.swap(d);
            eprint!("\nE: {:?}", now.elapsed());
        }
        for t in threads {
            let _ = t.join();
        }
    }
}
