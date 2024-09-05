use super::File;
use crate::mm::UserBuffer;
use crate::sync::{Condvar, Mutex, MutexSpin, UPSafeCell};
use alloc::sync::Arc;
use bitflags::bitflags;

pub struct EventFd {
    counter: UPSafeCell<u64>,
    flags: EventFdFlags,
    mutex: Arc<MutexSpin>,
    condvar: Arc<Condvar>,
}

bitflags! {
    pub struct EventFdFlags: u32 {
        const NONE = 0;
        const EFD_SEMAPHORE = 1;
        const EFD_NONBLOCK = 1 << 11;
    }
}

impl EventFd {
    pub fn new(initval: u32, flags: EventFdFlags) -> Self {
        EventFd {
            counter: unsafe { UPSafeCell::new(initval as u64) },
            flags,
            mutex: Arc::new(MutexSpin::new()),
            condvar: Arc::new(Condvar::new()),
        }
    }
}

impl File for EventFd {
    fn readable(&self) -> bool {
        true
    }

    fn writable(&self) -> bool {
        true
    }

    fn read(&self, mut buf: UserBuffer) -> usize {
        self.mutex.lock();
        let mut counter = self.counter.exclusive_access();
        while *counter == 0 {
            if self.flags & EventFdFlags::EFD_NONBLOCK != EventFdFlags::NONE {
                self.mutex.unlock();
                return (-2 as isize) as usize;
            } else {
                drop(counter);
                self.condvar.wait(self.mutex.clone());
            }
            counter = self.counter.exclusive_access();
        }

        let mut ret: usize = 0;
        if self.flags & EventFdFlags::EFD_SEMAPHORE == EventFdFlags::NONE {
            let cnt = counter.to_ne_bytes();
            let mut read_len = 8;
            if buf.len() < 8 {
                self.mutex.unlock();
                return (-2 as isize) as usize;
            }
            for slice in buf.buffers.iter_mut() {
                let read_size = slice.len().min(read_len);
                slice[..read_size].copy_from_slice(&cnt[..read_size]);
                if read_len == read_size {
                    break;
                } else {
                    read_len -= read_size;
                }
            }
            *counter = 0;
        } else {
            *counter -= 1;
            ret = 1;
        }
        drop(counter);
        self.mutex.unlock();
        ret
    }

    fn write(&self, buf: UserBuffer) -> usize {
        assert!(buf.len() == 8);
        self.mutex.lock();

        let mut cnt = self.counter.exclusive_access();
        if self.flags & EventFdFlags::EFD_SEMAPHORE == EventFdFlags::NONE {
            let mut buffer = [0u8; 8];
            let mut written = 0;
            for slice in buf.buffers.iter() {
                let write_size = slice.len();
                buffer[written..(written + write_size)].copy_from_slice(slice);
                written += write_size;
            }
            let delta = u64::from_ne_bytes(buffer);
            *cnt += delta;
        } else {
            *cnt += 1;
        }

        if *cnt > 0 {
            self.condvar.signal();
        }
        drop(cnt);
        self.mutex.unlock();
        0
    }
}
