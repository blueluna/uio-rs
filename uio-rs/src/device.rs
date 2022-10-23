use crate::error::Error;
use crate::internal::devfs_uio_file_path;
use crate::MapDescription;

/// UIO device
pub struct Device {
    /// UIO number
    uio_number: usize,
}

impl Device {
    /// Create UIO device from UIO number
    pub fn new(uio_number: usize) -> Result<Device, Error> {
        let file_path = devfs_uio_file_path(uio_number);
        match file_path.try_exists()? {
            true => Ok(Device { uio_number }),
            false => Err(Error::NoDevice),
        }
    }

    /// Get the UIO device number
    pub fn uio(&self) -> usize {
        self.uio_number
    }

    /// Enumerate UIO device maps
    pub fn maps(&self) -> Vec<MapDescription> {
        match MapDescription::enumerate(self.uio_number) {
            Ok(descriptions) => descriptions,
            Err(_) => vec![],
        }
    }
}
