use crate::error::Error;
use crate::internal::{
    read_hexadecimal_u64, read_hexadecimal_usize, read_string, read_u64, sysfs_uio_dir,
    sysfs_uio_map_dir, sysfs_uio_maps_dir,
};
use std::fs;

/// UIO device description
pub struct DeviceDescription {
    uio_number: u16,
    name: String,
    version: String,
}

impl DeviceDescription {
    /// Create UIO device description from UIO number
    pub fn new(uio_number: u16) -> Result<DeviceDescription, Error> {
        let dir_path = sysfs_uio_dir(uio_number);
        match dir_path.try_exists() {
            Ok(true) => {
                let name = read_string(dir_path.join("name"))?;
                let version = read_string(dir_path.join("version"))?;
                Ok(DeviceDescription {
                    uio_number,
                    name,
                    version,
                })
            }
            Ok(false) | Err(_) => Err(Error::NoDevice),
        }
    }

    /// Get the UIO device number
    pub fn uio(&self) -> u16 {
        self.uio_number
    }

    /// Get the UIO device name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the UIO device version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get the UIO device event count
    pub fn event_count(&self) -> u64 {
        read_u64(sysfs_uio_dir(self.uio_number).join("event")).unwrap_or(0)
    }

    /// Enumerate UIO device maps
    pub fn maps(&self) -> Vec<MapDescription> {
        match MapDescription::enumerate(self.uio_number) {
            Ok(descriptions) => descriptions,
            Err(_) => vec![],
        }
    }

    pub fn enumerate() -> Vec<DeviceDescription> {
        (0..u16::MAX)
            .map_while(|v| DeviceDescription::new(v).ok())
            .collect()
    }

    pub fn find_unique(name: &str) -> Result<DeviceDescription, Error> {
        let mut matches = 0usize;
        let mut fist_match = None;
        for description in DeviceDescription::enumerate() {
            if description.name == name {
                if fist_match.is_none() {
                    fist_match = Some(description)
                }
                matches = matches.saturating_add(1);
            }
        }
        if let Some(description) = fist_match {
            if matches == 1 {
                Ok(description)
            } else {
                Err(Error::NotFound)
            }
        } else {
            Err(Error::NotFound)
        }
    }
}

impl std::fmt::Display for DeviceDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "uio{} {} {}", self.uio_number, self.name, self.version)
    }
}

pub struct MapDescription {
    /// UIO number
    pub(crate) uio_number: u16,
    /// Map number
    pub(crate) map_number: u16,
    /// Map base address
    pub(crate) base_address: usize,
    /// Map size
    pub(crate) size: usize,
    /// Map offset
    pub(crate) offset: u64,
}

impl MapDescription {
    pub fn from_numbers(uio_number: u16, map_number: u16) -> Result<MapDescription, Error> {
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

    pub fn enumerate(uio_number: u16) -> Result<Vec<MapDescription>, Error> {
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
                let map_identity = match u16::from_str_radix(prefix, 10) {
                    Ok(n) => n,
                    Err(_) => u16::MAX,
                };
                if map_identity < u16::MAX {
                    if let Ok(mapping) = MapDescription::from_numbers(uio_number, map_identity) {
                        mappings.push(mapping);
                    }
                }
            }
        }
        Ok(mappings)
    }

    pub fn map(&self) -> u16 {
        self.map_number
    }

    pub fn base_address(&self) -> usize {
        self.base_address
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Get the UIO map name
    pub fn name(&self) -> Result<String, Error> {
        return read_string(sysfs_uio_map_dir(self.uio_number, self.map_number).join("name"))
            .into();
    }
}

impl std::fmt::Display for MapDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "uio{} map{} @{:016x} {:016x} +{:016x}",
            self.uio_number, self.map_number, self.base_address, self.size, self.offset
        )
    }
}
