//! Information about the jemalloc compile-time configuration
use std::io;
use std::os::raw::c_char;

use {get_str, get_str_mib, name_to_mib};

const MALLOC_CONF: *const c_char = b"config.malloc_conf\0" as *const _ as *const _;

/// Returns the embeddec configure-time-specified run-time options config.
///
/// The string will be empty unless `--with-malloc-conf` was specified during build configuration.
///
/// # Examples
///
/// ```
/// extern crate jemallocator;
/// extern crate jemalloc_ctl;
///
/// #[global_allocator]
/// static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
///
/// fn main() {
///     println!("default malloc conf: {}", jemalloc_ctl::config::malloc_conf().unwrap());
/// }
/// ```
pub fn malloc_conf() -> io::Result<&'static str> {
    unsafe { get_str(MALLOC_CONF) }
}

/// A type providing access to the embedded configure-time-specified run-time options config.
///
/// The string will be empty unless `--with-malloc-conf` was specified during build configuration.
///
/// # Examples
///
/// ```
/// extern crate jemallocator;
/// extern crate jemalloc_ctl;
///
/// use jemalloc_ctl::config::MallocConf;
///
/// #[global_allocator]
/// static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
///
/// fn main() {
///     let malloc_conf = MallocConf::new().unwrap();
///
///     println!("default malloc conf: {}", malloc_conf.get().unwrap());
/// }
/// ```
#[derive(Copy, Clone)]
pub struct MallocConf([usize; 2]);

impl MallocConf {
    /// Returns a new `MallocConf`.
    pub fn new() -> io::Result<MallocConf> {
        unsafe {
            let mut mib = [0; 2];
            name_to_mib(MALLOC_CONF, &mut mib)?;
            Ok(MallocConf(mib))
        }
    }

    /// Returns the embedded configure-time-specified run-time options config.
    pub fn get(&self) -> io::Result<&'static str> {
        unsafe { get_str_mib(&self.0) }
    }
}
