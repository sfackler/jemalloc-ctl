//! Thread specific operations.
use std::io;

use {name_to_mib, get_mib};

/// A type providing access to the total number of bytes allocated by the current thread.
///
/// Unlike [`stats::Allocated`], the value returned by this type is not the number of bytes
/// *currently* allocated, but rather the number of bytes that have *ever* been allocated by this
/// thread.
///
/// The `get` method doesn't return the value directly, but actually a pointer to the value. This
/// allows for very fast repeated lookup, since there is no function call overhead. The pointer type
/// cannot be sent to other threads, but `Allocated::get` can be called on different threads and
/// will return the appropriate pointer for each of them.
///
/// # Example
///
/// ```
/// use jemalloc_ctl::thread::Allocated;
///
/// let allocated = Allocated::new().unwrap();
/// let allocated = allocated.get().unwrap();
///
/// let a = allocated.get();
/// let buf = vec![0; 1024 * 1024];
/// let b = allocated.get();
/// drop(buf);
/// let c = allocated.get();
///
/// assert!(a < b);
/// assert_eq!(b, c);
/// ```
///
/// [`stats::Allocated`]: ../stats/struct.Allocated.html
#[derive(Copy, Clone)]
pub struct Allocated([usize; 2]);

impl Allocated {
    /// Returns a new `Allocated`.
    pub fn new() -> io::Result<Allocated> {
        let mut mib = [0; 2];
        unsafe {
            name_to_mib("thread.allocatedp\0", &mut mib)?;
        }
        Ok(Allocated(mib))
    }

    /// Returns a thread-local pointer to the total number of bytes allocated by this thread.
    pub fn get(&self) -> io::Result<ThreadLocal<u64>> {
        unsafe {
            let ptr = get_mib::<*mut u64>(&self.0)?;
            Ok(ThreadLocal(ptr))
        }
    }
}

/// A type providing access to the total number of bytes deallocated by the current thread.
///
/// The `get` method doesn't return the value directly, but actually a pointer to the value. This
/// allows for very fast repeated lookup, since there is no function call overhead. The pointer type
/// cannot be sent to other threads, but `Deallocated::get` can be called on different threads and
/// will return the appropriate pointer for each of them.
///
/// # Example
///
/// ```
/// use jemalloc_ctl::thread::Deallocated;
///
/// let deallocated = Deallocated::new().unwrap();
/// let deallocated = deallocated.get().unwrap();
///
/// let a = deallocated.get();
/// let buf = vec![0; 1024 * 1024];
/// let b = deallocated.get();
/// drop(buf);
/// let c = deallocated.get();
///
/// assert_eq!(a, b);
/// assert!(b < c);
/// ```
#[derive(Copy, Clone)]
pub struct Deallocated([usize; 2]);

impl Deallocated {
    /// Returns a new `Deallocated`.
    pub fn new() -> io::Result<Deallocated> {
        let mut mib = [0; 2];
        unsafe {
            name_to_mib("thread.deallocatedp\0", &mut mib)?;
        }
        Ok(Deallocated(mib))
    }

    /// Returns a thread-local pointer to the total number of bytes deallocated by this thread.
    pub fn get(&self) -> io::Result<ThreadLocal<u64>> {
        unsafe {
            let ptr = get_mib::<*mut u64>(&self.0)?;
            Ok(ThreadLocal(ptr))
        }
    }
}

/// A a thread-local pointer.
///
/// It is neither `Sync` nor `Send`.
// NB we need *const here specifically since it's !Sync + !Send
#[derive(Copy, Clone)]
pub struct ThreadLocal<T>(*const T);

impl<T> ThreadLocal<T>
where
    T: Copy,
{
    /// Returns the current value at the pointer.
    #[inline]
    pub fn get(&self) -> T {
        unsafe { *self.0 }
    }
}
