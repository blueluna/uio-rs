use crate::description::MapDescription;
use crate::device::Device;
use crate::error::Error;
use crate::internal::get_page_size;
use byteorder::{ByteOrder, NativeEndian};
use memmap2::{MmapOptions, MmapRaw};
use std::slice;

pub struct Map<'a> {
    /// UIO index
    uio_number: u16,
    /// Mapping index
    map_number: u16,
    /// Mapping size
    map_size: usize,
    /// Memory mapping
    mem_map: MmapRaw,
    /// UIO device
    _device: &'a Device,
}

impl<'a> Map<'a> {
    pub fn new(device: &'a Device, map_number: u16) -> Result<Self, Error> {
        let description = MapDescription::from_numbers(device.uio(), map_number)?;

        let offset = u64::from(description.map_number) * get_page_size();
        let map_size = description.size;
        let mut options = MmapOptions::new();
        options.offset(offset).len(map_size);

        let mem_map = options.map_raw(&device.file)?;
        // Following is required for not getting "double accesses" to registers
        mem_map.advise(memmap2::Advice::Random)?;

        Ok(Self {
            uio_number: description.uio_number,
            map_number: description.map_number,
            map_size,
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

    pub fn read_u32(&self, byte_offset: usize) -> Result<u32, Error> {
        let chunk = self.read_exact(byte_offset, std::mem::size_of::<u32>())?;
        Ok(NativeEndian::read_u32(&chunk))
    }

    pub fn write_u32(&mut self, byte_offset: usize, value: u32) -> Result<(), Error> {
        let destination = self.map_slice_mut(byte_offset, std::mem::size_of::<u32>())?;
        NativeEndian::write_u32(destination, value);
        Ok(())
    }

    pub fn write_u64(&mut self, byte_offset: usize, value: u64) -> Result<(), Error> {
        let destination = self.map_slice_mut(byte_offset, std::mem::size_of::<u64>())?;
        NativeEndian::write_u64(destination, value);
        Ok(())
    }

    pub fn write_u128(&mut self, byte_offset: usize, value: u128) -> Result<(), Error> {
        let destination = self.map_slice_mut(byte_offset, std::mem::size_of::<u128>())?;
        NativeEndian::write_u128(destination, value);
        Ok(())
    }

    pub fn read_exact(&self, byte_offset: usize, bytes: usize) -> Result<&[u8], Error> {
        if (byte_offset + bytes) > self.map_size {
            return Err(Error::OutOfBound);
        }
        Ok(unsafe {
            slice::from_raw_parts(self.mem_map.as_ptr().wrapping_byte_add(byte_offset), bytes)
        })
    }

    pub fn read_all(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.mem_map.as_ptr(), self.map_size) }
    }

    fn map_slice_mut(&mut self, byte_offset: usize, bytes: usize) -> Result<&mut [u8], Error> {
        if (byte_offset + bytes) > self.map_size {
            return Err(Error::OutOfBound);
        }
        Ok(unsafe {
            slice::from_raw_parts_mut(
                self.mem_map.as_mut_ptr().wrapping_byte_add(byte_offset),
                bytes,
            )
        })
    }
}
