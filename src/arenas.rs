//! Arena operations.
use std::io;
use std::os::raw::c_uint;

use {name_to_mib, get_mib};

/// A type providing access to the current limit on the number of arenas.
///
/// # Examples
///
/// ```
/// use jemalloc_ctl::arenas::NArenas;
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
            name_to_mib("arenas.narenas\0", &mut mib)?;
            Ok(NArenas(mib))
        }
    }

    /// Returns the maximum number of arenas.
    pub fn get(&self) -> io::Result<c_uint> {
        unsafe { get_mib(&self.0) }
    }
}
