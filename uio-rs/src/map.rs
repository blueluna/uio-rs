use crate::description::MapDescription;
use crate::device::Device;
use crate::error::Error;
use crate::internal::get_page_size;
use byteorder::{ByteOrder, NativeEndian};
use memmap2::MmapOptions;

pub struct Map<'a> {
    /// UIO index
    uio_number: u16,
    /// Mapping index
    map_number: u16,
    /// Memory mapping
    mem_map: memmap2::MmapMut,
    /// UIO device
    _device: &'a Device,
}

impl<'a> Map<'a> {
    pub fn new(device: &'a Device, map_number: u16) -> Result<Self, Error> {
        let description = MapDescription::from_numbers(device.uio(), map_number)?;

        let offset = u64::from(description.map_number) * get_page_size();
        let mut options = MmapOptions::new();
        options.offset(offset).len(description.size);

        let mem_map = unsafe { options.map_mut(&device.file) }?;

        Ok(Self {
            uio_number: description.uio_number,
            map_number: description.map_number,
            mem_map,
            _device: device,
        })
    }

    pub fn uio(&self) -> u16 {
        self.uio_number
    }

    pub fn map(&self) -> u16 {
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
