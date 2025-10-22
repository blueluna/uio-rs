use crate::error::Error;
use crate::{DeviceDescription, MapDescription};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};

/// UIO device
pub struct Device {
    /// UIO device number
    pub(crate) uio_number: u16,
    /// device file
    pub(crate) file: File,
}

impl Device {
    /// Create UIO device from UIO number
    pub fn new(uio_number: u16) -> Result<Device, Error> {
        let path = std::path::PathBuf::from(format!("/dev/uio{}", uio_number));
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        match file.try_lock() {
            Ok(_) => Ok(Device { uio_number, file }),
            Err(_) => Err(Error::DeviceLock),
        }
    }
    /// Create UIO device from UIO name
    ///
    /// The name needs to uniquely match a UIO device name, otherwise a NotFound error will be raised
    pub fn try_from_name(name: &str) -> Result<Device, Error> {
        let description = DeviceDescription::find_unique(name)?;
        Device::new(description.uio())
    }

    /// Get the UIO device number
    pub fn uio(&self) -> u16 {
        self.uio_number
    }

    /// Enumerate UIO device maps
    pub fn maps(&self) -> Vec<MapDescription> {
        match MapDescription::enumerate(self.uio_number) {
            Ok(descriptions) => descriptions,
            Err(_) => vec![],
        }
    }

    /// Enable interrupt
    pub fn interrupt_enable(&mut self) -> Result<(), Error> {
        let bytes: [u8; 4] = 1u32.to_ne_bytes();
        self.file.write(&bytes)?;
        Ok(())
    }

    /// Disable interrupt
    pub fn interrupt_disable(&mut self) -> Result<(), Error> {
        self.file.write(&0u32.to_ne_bytes())?;
        Ok(())
    }

    /// Wait for interrupt
    pub fn interrupt_wait(&mut self) -> Result<u32, Error> {
        let mut bytes: [u8; 4] = [0, 0, 0, 0];
        self.file.read(&mut bytes)?;
        Ok(u32::from_ne_bytes(bytes))
    }
}

impl AsRawFd for Device {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}
