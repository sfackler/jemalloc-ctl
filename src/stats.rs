//! Global allocator statistics.
//!
//! jemalloc tracks a wide variety of statistics. Many of them are cached, and only refreshed when
//! the jemalloc "epoch" is advanced. See the [`Epoch`] type for more information.
//!
//! [`Epoch`]: ../struct.Epoch.html

use std::io;

use {get_mib, name_to_mib};

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
/// use jemalloc_ctl::Epoch;
/// use jemalloc_ctl::stats::Allocated;
///
/// let epoch = Epoch::new().unwrap();
/// let allocated = Allocated::new().unwrap();
///
/// let a = allocated.get().unwrap();
/// let _buf = vec![0; 1024 * 1024];
/// epoch.advance().unwrap();
/// let b = allocated.get().unwrap();
/// assert!(a < b);
/// ```
///
/// [`Epoch`]: ../struct.Epoch.html
#[derive(Copy, Clone)]
pub struct Allocated([usize; 2]);

impl Allocated {
    /// Returns a new `Allocated`.
    pub fn new() -> io::Result<Allocated> {
        let mut mib = [0; 2];
        unsafe {
            name_to_mib("stats.allocated\0", &mut mib)?;
        }
        Ok(Allocated(mib))
    }

    /// Returns the total number of bytes allocated by the application.
    pub fn get(&self) -> io::Result<usize> {
        unsafe { get_mib(&self.0) }
    }
}

/// A type providing access to the total number of bytes in active pages allocated by the
/// application.
///
/// This is a multiple of the page size, and greater than or equal to the value returned by
/// [`Allocated`].
///
/// This statistic is cached, and is only refreshed when the epoch is advanced. See the [`Epoch`]
/// type for more information.
///
/// This corresponds to `stats.active` in jemalloc's API.
///
/// # Examples
///
/// ```rust
/// use jemalloc_ctl::Epoch;
/// use jemalloc_ctl::stats::Active;
///
/// let epoch = Epoch::new().unwrap();
/// let active = Active::new().unwrap();
///
/// let a = active.get().unwrap();
/// let _buf = vec![0; 1024 * 1024];
/// epoch.advance().unwrap();
/// let b = active.get().unwrap();
/// assert!(a < b);
/// ```
///
/// [`Epoch`]: ../struct.Epoch.html
/// [`Allocated`]: struct.Allocated.html
#[derive(Copy, Clone)]
pub struct Active([usize; 2]);

impl Active {
    /// Returns a new `Allocated`.
    pub fn new() -> io::Result<Active> {
        let mut mib = [0; 2];
        unsafe {
            name_to_mib("stats.active\0", &mut mib)?;
        }
        Ok(Active(mib))
    }

    /// Returns the total number of bytes in active pages allocated by the application.
    pub fn get(&self) -> io::Result<usize> {
        unsafe { get_mib(&self.0) }
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
/// use jemalloc_ctl::Epoch;
/// use jemalloc_ctl::stats::Metadata;
///
/// let epoch = Epoch::new().unwrap();
/// let metadata = Metadata::new().unwrap();
///
/// epoch.advance().unwrap();
/// let size = metadata.get().unwrap();
/// println!("{} bytes of jemalloc metadata", size);
/// ```
///
/// [`Epoch`]: ../struct.Epoch.html
#[derive(Copy, Clone)]
pub struct Metadata([usize; 2]);

impl Metadata {
    /// Returns a new `Metadata`.
    pub fn new() -> io::Result<Metadata> {
        let mut mib = [0; 2];
        unsafe {
            name_to_mib("stats.metadata\0", &mut mib)?;
        }
        Ok(Metadata(mib))
    }

    /// Returns the total number of bytes dedicated to jemalloc metadata.
    pub fn get(&self) -> io::Result<usize> {
        unsafe { get_mib(&self.0) }
    }
}

/// A type providing access to the total number of bytes in physically resident data pages mapped
/// by the allocator.
///
/// This consists of all pages dedicated to allocator metadata, pages backing active allocations,
/// and unused dirty pages. It may overestimate the true value because pages may not actually be
/// physically resident if they correspond to demand-zeroed virtual memory that has not yet been
/// touched. This is a multiple of the page size, and is larger than the value returned by
/// [`Active`].
///
/// This statistic is cached, and is only refreshed when the epoch is advanced. See the [`Epoch`]
/// type for more information.
///
/// This corresponds to `stats.resident` in jemalloc's API.
///
/// # Examples
///
/// ```rust
/// use jemalloc_ctl::Epoch;
/// use jemalloc_ctl::stats::Resident;
///
/// let epoch = Epoch::new().unwrap();
/// let resident = Resident::new().unwrap();
///
/// epoch.advance().unwrap();
/// let size = resident.get().unwrap();
/// println!("{} bytes of total resident data", size);
/// ```
///
/// [`Epoch`]: ../struct.Epoch.html
/// [`Active`]: struct.Active.html
#[derive(Copy, Clone)]
pub struct Resident([usize; 2]);

impl Resident {
    /// Returns a new `Resident`.
    pub fn new() -> io::Result<Resident> {
        let mut mib = [0; 2];
        unsafe {
            name_to_mib("stats.resident\0", &mut mib)?;
        }
        Ok(Resident(mib))
    }

    /// Returns the total number of bytes in physically resident data pages mapped by the allocator.
    pub fn get(&self) -> io::Result<usize> {
        unsafe { get_mib(&self.0) }
    }
}

/// A type providing access to the total number of bytes in active extents mapped by the allocator.
///
/// This does not include inactive extents, even those that contain unused dirty pages, so there
/// is no strict ordering between this and the value returned by [`Resident`]. This is a
/// multiple of the page size, and is larger than the value returned by [`Active`].
///
/// This statistic is cached, and is only refreshed when the epoch is advanced. See the [`Epoch`]
/// type for more information.
///
/// This corresponds to `stats.mapped` in jemalloc's API.
///
/// # Examples
///
/// ```rust
/// use jemalloc_ctl::Epoch;
/// use jemalloc_ctl::stats::Mapped;
///
/// let epoch = Epoch::new().unwrap();
/// let mapped = Mapped::new().unwrap();
///
/// epoch.advance().unwrap();
/// let size = mapped.get().unwrap();
/// println!("{} bytes of total mapped data", size);
/// ```
///
/// [`Epoch`]: ../struct.Epoch.html
/// [`Resident`]: struct.Resident.html
/// [`Active`]: struct.Active.html
#[derive(Copy, Clone)]
pub struct Mapped([usize; 2]);

impl Mapped {
    /// Returns a new `Mapped`.
    pub fn new() -> io::Result<Mapped> {
        let mut mib = [0; 2];
        unsafe {
            name_to_mib("stats.mapped\0", &mut mib)?;
        }
        Ok(Mapped(mib))
    }

    /// Returns the total number of bytes in active extents mapped by the allocator.
    pub fn get(&self) -> io::Result<usize> {
        unsafe { get_mib(&self.0) }
    }
}
