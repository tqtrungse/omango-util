// Copyright (c) 2024 Trung Tran <tqtrungse@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU32, Ordering},
};

use crate::backoff::Backoff;
use crate::hint::unlikely;

const WRITE_NUMBER: u32 = 1_u32 << 30;

pub struct RwSpinlock<T> {
    flag: AtomicU32,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for RwSpinlock<T> {}

unsafe impl<T: Send> Sync for RwSpinlock<T> {}

impl<T> RwSpinlock<T> {
    #[inline(always)]
    pub fn new(value: T) -> Self {
        Self {
            flag: AtomicU32::new(0),
            value: UnsafeCell::new(value),
        }
    }

    pub fn try_write(&self) -> Option<RwSpinlockGuard<T>> {
        if self.flag.compare_exchange_weak(
            0,
            WRITE_NUMBER,
            Ordering::Acquire,
            Ordering::Relaxed,
        ).is_ok() {
            return Some(RwSpinlockGuard { parent: self });
        }
        None
    }

    pub fn try_read(&self) -> Option<RwSpinlockGuard<T>> {
        let pre_value = self.flag.fetch_add(1, Ordering::Relaxed);
        if pre_value < WRITE_NUMBER {
            return Some(RwSpinlockGuard { parent: self });
        }
        None
    }

    pub fn write(&self) -> RwSpinlockGuard<T> {
        let backoff = Backoff::default();
        loop {
            // "compare_exchange" performance is better than "swap".
            // The reason for using a weak "compare_exchange" is explained here:
            // https://github.com/Amanieu/parking_lot/pull/207#issuecomment-575869107
            if self.flag.compare_exchange_weak(
                0,
                WRITE_NUMBER,
                Ordering::Acquire,
                Ordering::Relaxed,
            ).is_ok() {
                break;
            }

            while self.flag.load(Ordering::Relaxed) != 0 {
                // Waits the lock is unlocked to reduce CPU cache coherence.
                backoff.spin();
            }
        }
        RwSpinlockGuard { parent: self }
    }

    pub fn read(&self) -> RwSpinlockGuard<T> {
        let backoff = Backoff::default();
        loop {
            let pre_value = self.flag.fetch_add(1, Ordering::Relaxed);
            if pre_value < WRITE_NUMBER {
                break;
            }

            while self.flag.load(Ordering::Relaxed) != 0 {
                // Waits the lock is unlocked to reduce CPU cache coherence.
                backoff.spin();
            }
        }
        RwSpinlockGuard { parent: self }
    }
}

pub struct RwSpinlockGuard<'a, T> {
    parent: &'a RwSpinlock<T>,
}

impl<T> Drop for RwSpinlockGuard<'_, T> {
    #[inline(always)]
    fn drop(&mut self) {
        if unlikely(self.parent.flag.load(Ordering::Relaxed) >= WRITE_NUMBER) {
            self.parent.flag.store(0, Ordering::Release);
        } else {
            self.parent.flag.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

impl<T> Deref for RwSpinlockGuard<'_, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { &*self.parent.value.get() }
    }
}

impl<T> DerefMut for RwSpinlockGuard<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.parent.value.get() }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    
    use super::*;

    #[allow(unused_variables)]
    #[test]
    fn test_read_unlock() {
        let m = RwSpinlock::<i32>::new(0);
        {
            let r1 = m.read();
            {
                let r2 = m.read();
                let r2 = m.read();
                assert!(m.try_write().is_none());
            }
            assert!(m.try_write().is_none());
        }
        assert!(m.try_write().is_some());
    }

    #[allow(unused_variables)]
    #[test]
    fn test_write_unlock() {
        let m = RwSpinlock::<i32>::new(0);
        {
            let w1 = m.write();
            assert!(m.try_read().is_none());
        }
        assert!(m.try_read().is_some());
    }
    
    #[test]
    fn test_rw_arc() {
        let arc = Arc::new(RwSpinlock::new(0));
        let arc2 = arc.clone();
        let (tx, rx) = std::sync::mpsc::channel();
    
        std::thread::spawn(move || {
            let mut lock = arc2.write();
            for _ in 0..10 {
                let tmp = *lock;
                *lock = -1;
                std::thread::yield_now();
                *lock = tmp + 1;
            }
            tx.send(()).unwrap();
        });
    
        // Readers try to catch the writer in the act
        let mut children = Vec::new();
        for _ in 0..5 {
            let arc3 = arc.clone();
            children.push(std::thread::spawn(move || {
                let lock = arc3.read();
                assert!(*lock >= 0);
            }));
        }
    
        // Wait for children to pass their asserts
        for r in children {
            assert!(r.join().is_ok());
        }
    
        // Wait for writer to finish
        rx.recv().unwrap();
        let lock = arc.read();
        assert_eq!(*lock, 10);
    }
    
    #[test]
    fn test_rw_access_in_unwind() {
        let arc = Arc::new(RwSpinlock::new(1));
        let arc2 = arc.clone();
        let _ = std::thread::spawn(move || {
            struct Unwinder {
                i: Arc<RwSpinlock<isize>>,
            }
            impl Drop for Unwinder {
                fn drop(&mut self) {
                    let mut lock = self.i.write();
                    *lock += 1;
                }
            }
            let _u = Unwinder { i: arc2 };
            panic!();
        })
            .join();
        let lock = arc.read();
        assert_eq!(*lock, 2);
    }
    
    #[test]
    fn test_rwlock_unsized() {
        let rw: &RwSpinlock<[i32;3]> = &RwSpinlock::new([1, 2, 3]);
        {
            let b = &mut *rw.write();
            b[0] = 4;
            b[2] = 5;
        }
        let comp: &[i32] = &[4, 2, 5];
        assert_eq!(&*rw.read(), comp);
    }

    #[allow(clippy::assertions_on_constants)]
    #[test]
    fn test_rwlock_try_write() {
        let lock = RwSpinlock::new(0isize);
        let read_guard = lock.read();
    
        let write_result = lock.try_write();
        match write_result {
            None => (),
            Some(_) => assert!(
                false,
                "try_write should not succeed while read_guard is in scope"
            ),
        }
    
        drop(read_guard);
    }
    
    #[test]
    fn test_rw_try_read() {
        let m = RwSpinlock::new(0);
        std::mem::forget(m.write());
        assert!(m.try_read().is_none());
    }
}