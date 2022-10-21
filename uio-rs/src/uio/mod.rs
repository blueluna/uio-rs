use crate::error::Error;
use byteorder::{ByteOrder, NativeEndian};
use log;
use memmap2::MmapOptions;
use std::fs::{self, File, OpenOptions};

fn devfs_uio_file_path(uio_number: usize) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("/dev/uio{}", uio_number))
}

fn sysfs_uio_dir(uio_number: usize) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("/sys/class/uio/uio{}", uio_number))
}

fn sysfs_uio_maps_dir(uio_number: usize) -> std::path::PathBuf {
    sysfs_uio_dir(uio_number).join("maps")
}

fn sysfs_uio_map_dir(uio_number: usize, map_number: usize) -> std::path::PathBuf {
    sysfs_uio_maps_dir(uio_number).join(format!("map{}", map_number))
}

fn read_value<P: AsRef<std::path::Path>>(path: P) -> Result<String, Error> {
    let text = fs::read_to_string(path)?;
    Ok(String::from(text.trim()))
}

fn read_hexadecimal_u64<P: AsRef<std::path::Path>>(path: P) -> Result<u64, Error> {
    let text = read_value(path)?;
    let text = if text.starts_with("0x") {
        let (_, b) = text.split_at(2);
        b
    } else {
        &text
    };
    u64::from_str_radix(text, 16).map_err(|e| e.into())
}

fn read_hexadecimal_usize<P: AsRef<std::path::Path>>(path: P) -> Result<usize, Error> {
    let text = read_value(path)?;
    let text = if text.starts_with("0x") {
        let (_, b) = text.split_at(2);
        b
    } else {
        &text
    };
    usize::from_str_radix(text, 16).map_err(|e| e.into())
}

pub struct Device {
    /// UIO number
    uio_number: usize,
}

impl Device {
    pub fn new(uio_number: usize) -> Result<Device, Error> {
        let file_path = devfs_uio_file_path(uio_number);
        match file_path.try_exists()? {
            true => Ok(Device { uio_number }),
            false => Err(Error::NoDevice),
        }
    }

    pub fn map_descriptors(&self) -> Result<Vec<MapDescription>, Error> {
        let maps_dir = sysfs_uio_maps_dir(self.uio_number);
        let mut mappings = vec![];
        for entry in fs::read_dir(maps_dir)? {
            let entry = entry?;
            let entry_name = match entry.file_name().into_string() {
                Ok(name) => name,
                Err(_) => continue,
            };
            if entry.path().is_dir() && entry_name.starts_with("map") {
                let (_, prefix) = entry_name.split_at(3);
                let map_identity = match usize::from_str_radix(prefix, 10) {
                    Ok(n) => n,
                    Err(_) => usize::MAX,
                };
                if map_identity < usize::MAX {
                    if let Ok(mapping) = MapDescription::from_numbers(self.uio_number, map_identity)
                    {
                        mappings.push(mapping);
                    }
                }
            }
        }
        Ok(mappings)
    }
}

pub struct MapDescription {
    /// UIO number
    pub(crate) uio_number: usize,
    /// Map number
    pub(crate) map_number: usize,
    /// Map base address
    pub(crate) base_address: usize,
    /// Map size
    pub(crate) size: usize,
    /// Map offset
    pub(crate) offset: u64,
}

impl MapDescription {
    pub fn from_numbers(uio_number: usize, map_number: usize) -> Result<MapDescription, Error> {
        let map_path = sysfs_uio_map_dir(uio_number, map_number);
        let base_address = read_hexadecimal_usize(map_path.join("addr"))?;
        let size = read_hexadecimal_usize(map_path.join("size"))?;
        let offset = read_hexadecimal_u64(map_path.join("offset"))?;
        Ok(MapDescription {
            uio_number,
            map_number,
            base_address,
            size,
            offset,
        })
    }
}

impl core::fmt::Display for MapDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "iou{} map{} @{:016x} size {:016x} offset {:016x}",
            self.uio_number, self.map_number, self.base_address, self.size, self.offset
        )
    }
}

pub struct Mapping {
    /// UIO index
    uio_number: usize,
    /// Mapping index
    map_number: usize,
    /// Device file handle
    device_file: File,
    /// Memory mapping
    mem_map: memmap2::MmapMut,
}

impl Mapping {
    pub fn new(uio_number: usize, map_number: usize) -> Result<Self, Error> {
        let description = MapDescription::from_numbers(uio_number, map_number)?;
        Mapping::from_description(&description)
    }

    pub fn from_description(description: &MapDescription) -> Result<Self, Error> {
        let dev_file_path = devfs_uio_file_path(description.uio_number);
        let mut device_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(dev_file_path)?;
        let mut options = MmapOptions::new();
        options.offset(description.offset).len(description.size);
        let mem_map = unsafe { options.map_mut(&device_file) }?;
        Ok(Self {
            uio_number: description.uio_number,
            map_number: description.map_number,
            device_file,
            mem_map,
        })
    }

    pub fn read_u32(&self, offset: usize) -> Result<u32, Error> {
        let end = offset + std::mem::size_of::<u32>();
        if end > self.mem_map.len() {
            return Err(Error::OutOfBound);
        }
        Ok(NativeEndian::read_u32(&self.mem_map[offset..end]))
    }

    pub fn write_u32(&mut self, offset: usize, value: u32) -> Result<(), Error> {
        let end = offset + std::mem::size_of::<u32>();
        if end > self.mem_map.len() {
            return Err(Error::OutOfBound);
        }
        NativeEndian::write_u32(&mut self.mem_map[offset..end], value);
        Ok(())
    }
}
