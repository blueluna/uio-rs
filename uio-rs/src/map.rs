use crate::error::Error;
use crate::internal::{
    devfs_uio_open, read_hexadecimal_u64, read_hexadecimal_usize, sysfs_uio_map_dir,
    sysfs_uio_maps_dir,
};
use byteorder::{ByteOrder, NativeEndian};
use memmap2::MmapOptions;
use std::fs;

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

    pub fn enumerate(uio_number: usize) -> Result<Vec<MapDescription>, Error> {
        let maps_dir = sysfs_uio_maps_dir(uio_number);
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
                    if let Ok(mapping) = MapDescription::from_numbers(uio_number, map_identity) {
                        mappings.push(mapping);
                    }
                }
            }
        }
        Ok(mappings)
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

pub struct Map {
    /// UIO index
    uio_number: usize,
    /// Mapping index
    map_number: usize,
    /// Memory mapping
    mem_map: memmap2::MmapMut,
}

impl Map {
    pub fn new(uio_number: usize, map_number: usize) -> Result<Self, Error> {
        let description = MapDescription::from_numbers(uio_number, map_number)?;
        Map::from_description(&description)
    }

    pub fn from_description(description: &MapDescription) -> Result<Self, Error> {
        let device_file = devfs_uio_open(description.uio_number)?;
        let mut options = MmapOptions::new();
        options.offset(description.offset).len(description.size);
        let mem_map = unsafe { options.map_mut(&device_file) }?;
        Ok(Self {
            uio_number: description.uio_number,
            map_number: description.map_number,
            mem_map,
        })
    }

    pub fn uio(&self) -> usize {
        self.uio_number
    }

    pub fn map(&self) -> usize {
        self.map_number
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
