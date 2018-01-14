//! jemalloc control and introspection.
//!
//! jemalloc offers a powerful introspection and control interface through the `mallctl` function.
//! It can be used to tune the allocator, take heap dumps, and retrieve statistics. This crate
//! provides a typed API over that interface.
//!
//! While `mallctl` takes a string to specify an operation (e.g. `stats.allocated` or
//! stats.arenas.15.muzzy_decay_ms`), the overhead of repeatedly parsing those strings is not ideal.
//! Fortunately, jemalloc offers the ability to translate the string ahead of time into a
//! "Management Information Base" (MIB) to speed up future lookups.
//!
//! This crate provides a type for each `mallctl` operation. Its constructor performs the MIB
//! lookup, so the struct should be saved if the same operation is going to be repeatedly performed.
//!
//! # Warning
//!
//! This library is forced to assume that jemalloc is present and simply link to some of its
//! functions. This will result in linker errors when building a binary that doesn't actually use
//! jemalloc as its allocator.
//!
//! # Examples
//!
//! Repeatedly printing allocation statistics:
//!
//! ```no_run
//! use std::thread;
//! use std::time::Duration;
//! use jemalloc_ctl::Epoch;
//! use jemalloc_ctl::stats::{Allocated, Resident};
//!
//! let epoch = Epoch::new().unwrap();
//! let allocated = Allocated::new().unwrap();
//! let resident = Resident::new().unwrap();
//!
//! loop {
//!     // many statistics are cached and only updated when the epoch is advanced.
//!     epoch.advance().unwrap();
//!
//!     let allocated = allocated.get().unwrap();
//!     let resident = resident.get().unwrap();
//!     println!("{} bytes allocated/{} bytes resident", allocated, resident);
//!     thread::sleep(Duration::from_secs(10));
//! }
//! ```
#![doc(html_root_url = "https://docs.rs/jemalloc-ctl/0.1")]
#![warn(missing_docs)]
use std::os::raw::{c_char, c_int, c_void};
use std::io;
use std::mem;
use std::ptr;
use std::ffi::CStr;

pub mod arenas;
pub mod config;
pub mod opt;
pub mod stats;
pub mod stats_print;
pub mod thread;

extern "C" {
    #[cfg_attr(any(target_os = "macos", target_os = "android", target_os = "ios",
                     target_os = "dragonfly", target_os = "windows", target_env = "musl"),
               link_name = "je_mallctlnametomib")]
    fn mallctlnametomib(name: *const c_char, mibp: *mut usize, miblenp: *mut usize) -> c_int;

    #[cfg_attr(any(target_os = "macos", target_os = "android", target_os = "ios",
                     target_os = "dragonfly", target_os = "windows", target_env = "musl"),
               link_name = "je_mallctlbymib")]
    fn mallctlbymib(
        mib: *const usize,
        miblen: usize,
        oldp: *mut c_void,
        oldlenp: *mut usize,
        newp: *mut c_void,
        newlen: usize,
    ) -> c_int;

    #[cfg_attr(any(target_os = "macos", target_os = "android", target_os = "ios",
                     target_os = "dragonfly", target_os = "windows", target_env = "musl"),
               link_name = "je_malloc_stats_print")]
    fn malloc_stats_print(
        write_cb: Option<unsafe extern "C" fn(*mut c_void, *const c_char)>,
        cbopaque: *mut c_void,
        opts: *const c_char,
    );
}

unsafe fn name_to_mib(name: &str, mib: &mut [usize]) -> io::Result<()> {
    debug_assert!(name.ends_with("\0"));
    let mut len = mib.len();
    cvt(mallctlnametomib(
        name.as_ptr() as *const _,
        mib.as_mut_ptr(),
        &mut len,
    ))?;
    debug_assert_eq!(mib.len(), len);
    Ok(())
}

unsafe fn get_mib<T>(mib: &[usize]) -> io::Result<T> {
    let mut value = mem::uninitialized::<T>();
    let mut len = mem::size_of::<T>();
    cvt(mallctlbymib(
        mib.as_ptr(),
        mib.len(),
        &mut value as *mut _ as *mut _,
        &mut len,
        ptr::null_mut(),
        0,
    ))?;
    debug_assert_eq!(len, mem::size_of::<T>());
    Ok(value)
}

unsafe fn get_str_mib(mib: &[usize]) -> io::Result<&'static str> {
    let ptr: *const c_char = get_mib(mib)?;
    let cstr = CStr::from_ptr(ptr);
    cstr.to_str()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

unsafe fn get_set_mib<T>(mib: &[usize], mut value: T) -> io::Result<T> {
    let mut len = mem::size_of::<T>();
    cvt(mallctlbymib(
        mib.as_ptr(),
        mib.len(),
        &mut value as *mut _ as *mut _,
        &mut len,
        &mut value as *mut _ as *mut _,
        len,
    ))?;
    debug_assert_eq!(len, mem::size_of::<T>());
    Ok(value)
}

fn cvt(ret: c_int) -> io::Result<()> {
    if ret == 0 {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(ret as i32))
    }
}

/// A type providing access to the jemalloc epoch.
///
/// Many of the statistics tracked by jemalloc are cached. The epoch controls when they are
/// refreshed.
///
/// # Example
///
/// Advancing the epoch:
///
/// ```
/// use jemalloc_ctl::Epoch;
///
/// let epoch = Epoch::new().unwrap();
///
/// let a = epoch.advance().unwrap();
/// let b = epoch.advance().unwrap();
/// assert_eq!(a + 1, b);
#[derive(Copy, Clone)]
pub struct Epoch([usize; 1]);

impl Epoch {
    /// Returns a new `Epoch`.
    pub fn new() -> io::Result<Epoch> {
        let mut mib = [0; 1];
        unsafe {
            name_to_mib("epoch\0", &mut mib)?;
        }
        Ok(Epoch(mib))
    }

    /// Advances the epoch, returning it.
    ///
    /// The epoch advances by 1 every time it is advanced, so the value can be used to determine if
    /// another thread triggered a referesh.
    pub fn advance(&self) -> io::Result<u64> {
        unsafe { get_set_mib(&self.0, 1) }
    }
}

/// A type providing access to the jemalloc version string.
///
/// # Note
///
/// The version of jemalloc currently shipped with the Rust distribution has a bogus version string.
///
/// # Example
///
/// ```
/// use jemalloc_ctl::Version;
///
/// let version = Version::new().unwrap();
///
/// println!("jemalloc version {}", version.get().unwrap());
#[derive(Copy, Clone)]
pub struct Version([usize; 1]);

impl Version {
    /// Returns a new `Version`.
    pub fn new() -> io::Result<Version> {
        let mut mib = [0; 1];
        unsafe {
            name_to_mib("version\0", &mut mib)?;
        }
        Ok(Version(mib))
    }

    /// Returns the jemalloc version string.
    pub fn get(&self) -> io::Result<&'static str> {
        unsafe { get_str_mib(&self.0) }
    }
}
