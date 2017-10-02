use std::os::raw::{c_int, c_void, c_char, c_uint};
use std::io;
use std::mem;
use std::ptr;

#[allow(non_camel_case_types)]
type size_t = usize;

extern "C" {
    #[cfg_attr(any(target_os = "macos", target_os = "android", target_os = "ios",
                    target_os = "dragonfly", target_os = "windows", target_env = "musl"),
                link_name = "je_mallctlnametomib")]
    fn mallctlnametomib(name: *const c_char, mibp: *mut size_t, miblenp: *mut size_t) -> c_int;

    #[cfg_attr(any(target_os = "macos", target_os = "android", target_os = "ios",
                    target_os = "dragonfly", target_os = "windows", target_env = "musl"),
                link_name = "je_mallctlbymib")]
    fn mallctlbymib(
        mib: *const size_t,
        miblen: size_t,
        oldp: *mut c_void,
        oldlenp: *mut size_t,
        newp: *mut c_void,
        newlen: size_t,
    ) -> c_int;
}

unsafe fn name_to_mib(name: &str, mib: &mut [size_t]) -> io::Result<()> {
    let mut len = mib.len();
    cvt(mallctlnametomib(
        name.as_ptr() as *const _,
        mib.as_mut_ptr(),
        &mut len,
    ))?;
    debug_assert_eq!(mib.len(), len);
    Ok(())
}

unsafe fn get<T>(mib: &[size_t]) -> io::Result<T> {
    let mut value = mem::uninitialized::<T>();
    let mut len = mem::size_of::<T>();
    cvt(mallctlbymib(
        mib.as_ptr(),
        mib.len(),
        &mut value as *mut _ as *mut _,
        &mut len,
        ptr::null_mut(),
        0
    ))?;
    debug_assert_eq!(len, mem::size_of::<T>());
    Ok(value)
}

unsafe fn get_set<T>(mib: &[size_t], mut value: T) -> io::Result<T> {
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
pub struct Epoch([size_t; 1]);

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
        unsafe { get_set(&self.0, 1) }
    }
}

/// A type providing access to the total number of bytes allocated by the application.
///
/// This statistic is cached, and is only refreshed when the epoch is advanced. See the [`Epoch`]
/// type for more information.
///
/// This corresponds to `stats.allocated` in jemalloc's API.
///
/// # Examples
///
/// ```rust
/// use jemalloc_ctl::{Epoch, AllocatedMemory};
///
/// let epoch = Epoch::new().unwrap();
/// let allocated = AllocatedMemory::new().unwrap();
///
/// let a = allocated.get().unwrap();
/// let _buf = vec![0; 1024 * 1024];
/// epoch.advance().unwrap();
/// let b = allocated.get().unwrap();
/// assert!(a < b);
/// ```
///
/// [`Epoch`]: struct.Epoch.html
#[derive(Copy, Clone)]
pub struct AllocatedMemory([size_t; 2]);

impl AllocatedMemory {
    /// Returns a new `AllocatedMemory`.
    pub fn new() -> io::Result<AllocatedMemory> {
        let mut mib = [0; 2];
        unsafe {
            name_to_mib("stats.allocated\0", &mut mib)?;
        }
        Ok(AllocatedMemory(mib))
    }

    /// Returns the total number of bytes allocated by the application.
    pub fn get(&self) -> io::Result<usize> {
        unsafe { get(&self.0) }
    }
}

/// A type providing access to the total number of bytes in active pages allocated by the
/// application.
///
/// This is a multiple of the page size, and greater than or equal to the value returned by
/// [`AllocatedMemory`].
///
/// This statistic is cached, and is only refreshed when the epoch is advanced. See the [`Epoch`]
/// type for more information.
///
/// This corresponds to `stats.active` in jemalloc's API.
///
/// # Examples
///
/// ```rust
/// use jemalloc_ctl::{Epoch, ActiveMemory};
///
/// let epoch = Epoch::new().unwrap();
/// let active = ActiveMemory::new().unwrap();
///
/// let a = active.get().unwrap();
/// let _buf = vec![0; 1024 * 1024];
/// epoch.advance().unwrap();
/// let b = active.get().unwrap();
/// assert!(a < b);
/// ```
///
/// [`Epoch`]: struct.Epoch.html
#[derive(Copy, Clone)]
pub struct ActiveMemory([size_t; 2]);

impl ActiveMemory {
    /// Returns a new `AllocatedMemory`.
    pub fn new() -> io::Result<ActiveMemory> {
        let mut mib = [0; 2];
        unsafe {
            name_to_mib("stats.active\0", &mut mib)?;
        }
        Ok(ActiveMemory(mib))
    }

    /// Returns the total number of bytes in active pages allocated by the application.
    pub fn get(&self) -> io::Result<usize> {
        unsafe { get(&self.0) }
    }
}

/// A type providing access to the total number of bytes dedicated to jemalloc metadata.
///
/// This statistic is cached, and is only refreshed when the epoch is advanced. See the [`Epoch`]
/// type for more information.
///
/// This corresponds to `stats.metadata` in jemalloc's API.
///
/// # Examples
///
/// ```rust
/// use jemalloc_ctl::{Epoch, MetadataMemory};
///
/// let epoch = Epoch::new().unwrap();
/// let metadata = MetadataMemory::new().unwrap();
///
/// epoch.advance().unwrap();
/// let size = metadata.get().unwrap();
/// println!("{} bytes of jemalloc metadata", size);
/// ```
///
/// [`Epoch`]: struct.Epoch.html
#[derive(Copy, Clone)]
pub struct MetadataMemory([size_t; 2]);

impl MetadataMemory {
    /// Returns a new `MetadataMemory`.
    pub fn new() -> io::Result<MetadataMemory> {
        let mut mib = [0; 2];
        unsafe {
            name_to_mib("stats.metadata\0", &mut mib)?;
        }
        Ok(MetadataMemory(mib))
    }

    /// Returns the total number of bytes dedicated to jemalloc metadata.
    pub fn get(&self) -> io::Result<usize> {
        unsafe { get(&self.0) }
    }
}

/// A type providing access to the total number of bytes in physically resident data pages mapped
/// by the allocator.
///
/// This consists of all pages dedicated to allocator metadata, pages backing active allocations,
/// and unused dirty pages. It may overestimate the true value because pages may not actually be
/// physically resident if they correspond to demand-zeroed virtual memory that has not yet been
/// touched. This is a multiple of the page size, and is larger than the value returned by
/// [`ActiveMemory`].
///
/// This statistic is cached, and is only refreshed when the epoch is advanced. See the [`Epoch`]
/// type for more information.
///
/// This corresponds to `stats.resident` in jemalloc's API.
///
/// # Examples
///
/// ```rust
/// use jemalloc_ctl::{Epoch, ResidentMemory};
///
/// let epoch = Epoch::new().unwrap();
/// let resident = ResidentMemory::new().unwrap();
///
/// epoch.advance().unwrap();
/// let size = resident.get().unwrap();
/// println!("{} bytes of total resident data", size);
/// ```
///
/// [`Epoch`]: struct.Epoch.html
/// [`ActiveMemory`]: struct.ActiveMemory.html
#[derive(Copy, Clone)]
pub struct ResidentMemory([size_t; 2]);

impl ResidentMemory {
    /// Returns a new `ResidentMemory`.
    pub fn new() -> io::Result<ResidentMemory> {
        let mut mib = [0; 2];
        unsafe {
            name_to_mib("stats.resident\0", &mut mib)?;
        }
        Ok(ResidentMemory(mib))
    }

    /// Returns the total number of bytes in physically resident data pages mapped by the allocator.
    pub fn get(&self) -> io::Result<usize> {
        unsafe { get(&self.0) }
    }
}

/// A type providing access to the total number of bytes in active extents mapped by the allocator.
///
/// This does not include inactive extents, even those that contain unused dirty pages, so there
/// is no strict ordering between this and the value returned by [`ResidentMemory`]. This is a
/// multiple of the page size, and is larger than the value returned by [`ActiveMemory`].
///
/// This statistic is cached, and is only refreshed when the epoch is advanced. See the [`Epoch`]
/// type for more information.
///
/// This corresponds to `stats.mapped` in jemalloc's API.
///
/// # Examples
///
/// ```rust
/// use jemalloc_ctl::{Epoch, MappedMemory};
///
/// let epoch = Epoch::new().unwrap();
/// let mapped = MappedMemory::new().unwrap();
///
/// epoch.advance().unwrap();
/// let size = mapped.get().unwrap();
/// println!("{} bytes of total mapped data", size);
/// ```
///
/// [`Epoch`]: struct.Epoch.html
/// [`ResidentMemory`]: struct.ResidentMemory.html
/// [`ActiveMemory`]: struct.ActiveMemory.html
#[derive(Copy, Clone)]
pub struct MappedMemory([size_t; 2]);

impl MappedMemory {
    /// Returns a new `MappedMemory`.
    pub fn new() -> io::Result<MappedMemory> {
        let mut mib = [0; 2];
        unsafe {
            name_to_mib("stats.mapped\0", &mut mib)?;
        }
        Ok(MappedMemory(mib))
    }

    /// Returns the total number of bytes in active extents mapped by the allocator.
    pub fn get(&self) -> io::Result<usize> {
        unsafe { get(&self.0) }
    }
}
