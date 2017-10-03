//! Information about the run-time jemalloc configuration.
//!
//! These settings are controlled by the `MALLOC_CONF` environment variable.
use std::io;
use std::os::raw::c_uint;

use {name_to_mib, get_str, get};

/// A type providing access to the dss (`sbrk(2)`) allocation precedence as related to `mmap(2)`
/// allocation.
///
/// The following settings are supported if `sbrk(2)` is supported by the operating system:
/// "disabled", "primary", and "secondary"; otherwise only "disabled" is supported. The default is
/// "secondary" if `sbrk(2)` is supported by the operating system; "disabled" otherwise.
///
/// # Examples
///
/// ```
/// use jemalloc_ctl::opt::Dss;
///
/// let dss = Dss::new().unwrap();
///
/// println!("dss priority: {}", dss.get().unwrap());
/// ```
#[derive(Copy, Clone)]
pub struct Dss([usize; 2]);

impl Dss {
    /// Returns a new `Dss`.
    pub fn new() -> io::Result<Dss> {
        unsafe {
            let mut mib = [0; 2];
            name_to_mib("opt.dss\0", &mut mib)?;
            Ok(Dss(mib))
        }
    }

    /// Returns the dss allocation precedence.
    pub fn get(&self) -> io::Result<&'static str> {
        unsafe { get_str(&self.0) }
    }
}

/// A type providing access to the maximum number of arenas to use for automatic multiplexing of
/// threads and arenas.
///
/// The default is four times the number of CPUs, or one if there is a single CPU.
///
/// # Examples
///
/// ```
/// use jemalloc_ctl::opt::NArenas;
///
/// let narenas = NArenas::new().unwrap();
///
/// println!("number of arenas: {}", narenas.get().unwrap());
/// ```
#[derive(Copy, Clone)]
pub struct NArenas([usize; 2]);

impl NArenas {
    /// Returns a new `NArenas`.
    pub fn new() -> io::Result<NArenas> {
        unsafe {
            let mut mib = [0; 2];
            name_to_mib("opt.narenas\0", &mut mib)?;
            Ok(NArenas(mib))
        }
    }

    /// Returns the maximum number of arenas.
    pub fn get(&self) -> io::Result<c_uint> {
        unsafe { get(&self.0) }
    }
}
