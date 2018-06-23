//! Arena operations.
use std::io;
use std::os::raw::{c_char, c_uint};

use {get, get_mib, name_to_mib};

const NARENAS: *const c_char = b"arenas.narenas\0" as *const _ as *const _;

/// Returns the current limit on the number of arenas.
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
///     println!("number of arenas: {}", jemalloc_ctl::arenas::narenas().unwrap());
/// }
/// ```
pub fn narenas() -> io::Result<c_uint> {
    unsafe { get(NARENAS) }
}

/// A type providing access to the current limit on the number of arenas.
///
/// # Examples
///
/// ```
/// extern crate jemallocator;
/// extern crate jemalloc_ctl;
///
/// use jemalloc_ctl::arenas::NArenas;
///
/// #[global_allocator]
/// static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
///
/// fn main() {
///     let narenas = NArenas::new().unwrap();
///
///     println!("number of arenas: {}", narenas.get().unwrap());
/// }
/// ```
#[derive(Copy, Clone)]
pub struct NArenas([usize; 2]);

impl NArenas {
    /// Returns a new `NArenas`.
    pub fn new() -> io::Result<NArenas> {
        unsafe {
            let mut mib = [0; 2];
            name_to_mib(NARENAS, &mut mib)?;
            Ok(NArenas(mib))
        }
    }

    /// Returns the maximum number of arenas.
    pub fn get(&self) -> io::Result<c_uint> {
        unsafe { get_mib(&self.0) }
    }
}
