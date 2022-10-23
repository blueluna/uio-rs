use crate::error::Error;
use crate::internal::devfs_uio_open;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};

pub struct Interrupt {
    /// Device file handle
    device_file: File,
}

impl Interrupt {
    pub fn new(uio_number: usize) -> Result<Self, Error> {
        let device_file = devfs_uio_open(uio_number)?;
        Ok(Interrupt { device_file })
    }
    /// Enable interrupt
    pub fn enable(&mut self) -> Result<(), Error> {
        self.device_file.write(&1u32.to_le_bytes())?;
        Ok(())
    }

    /// Disable interrupt
    pub fn disable(&mut self) -> Result<(), Error> {
        self.device_file.write(&0u32.to_le_bytes())?;
        Ok(())
    }

    /// Wait for interrupt
    pub fn wait(&mut self) -> Result<u32, Error> {
        let mut bytes: [u8; 4] = [0, 0, 0, 0];
        self.device_file.read(&mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }
}

impl AsRawFd for Interrupt {
    fn as_raw_fd(&self) -> RawFd {
        self.device_file.as_raw_fd()
    }
}
