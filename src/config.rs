//! Information about the jemalloc compile-time configuration
use std::io;

use {name_to_mib, get_str};

/// A type providing access to the embedded configure-time-specified run-time options config.
///
/// The string will be empty unless `--with-malloc-conf` was specified during build configuration.
///
/// # Examples
///
/// ```
/// use jemalloc_ctl::config::MallocConf;
///
/// let malloc_conf = MallocConf::new().unwrap();
///
/// println!("default malloc conf: {}", malloc_conf.get().unwrap());
/// ```
#[derive(Copy, Clone)]
pub struct MallocConf([usize; 2]);

impl MallocConf {
    /// Returns a new `MallocConf`.
    pub fn new() -> io::Result<MallocConf> {
        unsafe {
            let mut mib = [0; 2];
            name_to_mib("config.malloc_conf\0", &mut mib)?;
            Ok(MallocConf(mib))
        }
    }

    /// Returns the embedded configure-time-specified run-time options config.
    pub fn get(&self) -> io::Result<&'static str> {
        unsafe { get_str(&self.0) }
    }
}
