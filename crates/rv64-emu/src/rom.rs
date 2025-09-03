//! The rom module contains the read-only memory structure and implementation to read the memory.
//! ROM includes a device tree blob (DTB) compiled from a device tree source (DTS).
use crate::{
    bus::MROM_BASE,
    dtb::dtb,
    exception::Exception,
};

/// RV64EMU read only memory
#[derive(Debug)]
pub struct Rom {
    data: Vec<u8>,
}

impl Rom {
    /// Create a new `rom` object.
    pub fn new() -> Self {
        let mut dtb = match dtb() {
            Ok(dtb) => dtb,
            Err(e) => {
                // TODO: should fail?
                println!("WARNING: failed to read a device tree binary: {}", e);
                println!(
                    "WARNING: maybe need to install dtc commend `apt install device-tree-compiler`"
                );
                Vec::new()
            }
        };

        // TODO: set a reset vector correctly.
        // 0x20 is the size of a reset vector.
        let mut rom = vec![0; 32];
        rom.append(&mut dtb);
        let align = 0x1000;
        rom.resize((rom.len() + align - 1) / align * align, 0);

        Self { data: rom }
    }

    pub fn new_with_data(data: Vec<u8>) -> Rom {
        Rom { data }
    }

    /// Load `size`-bit data from the memory.
    pub fn read(&self, addr: u64, size: u8) -> Result<u64, Exception> {
        match size {
            BYTE => Ok(self.read8(addr)),
            HALFWORD => Ok(self.read16(addr)),
            WORD => Ok(self.read32(addr)),
            DOUBLEWORD => Ok(self.read64(addr)),
            _ => return Err(Exception::LoadAccessFault),
        }
    }

    /// Store `size`-bit data to the memory. Returns the exception because the ROM is read-only.
    pub fn write(&self, _addr: u64, _value: u64, _size: u8) -> Result<(), Exception> {
        Err(Exception::StoreAMOAccessFault)
    }

    /// Read a byte from the rom.
    fn read8(&self, addr: u64) -> u64 {
        let index = (addr - MROM_BASE) as usize;
        self.data[index] as u64
    }

    /// Read 2 bytes from the rom.
    fn read16(&self, addr: u64) -> u64 {
        let index = (addr - MROM_BASE) as usize;
        return (self.data[index] as u64) | ((self.data[index + 1] as u64) << 8);
    }

    /// Read 4 bytes from the rom.
    fn read32(&self, addr: u64) -> u64 {
        let index = (addr - MROM_BASE) as usize;
        return (self.data[index] as u64)
            | ((self.data[index + 1] as u64) << 8)
            | ((self.data[index + 2] as u64) << 16)
            | ((self.data[index + 3] as u64) << 24);
    }

    /// Read 8 bytes from the rom.
    fn read64(&self, addr: u64) -> u64 {
        let index = (addr - MROM_BASE) as usize;
        return (self.data[index] as u64)
            | ((self.data[index + 1] as u64) << 8)
            | ((self.data[index + 2] as u64) << 16)
            | ((self.data[index + 3] as u64) << 24)
            | ((self.data[index + 4] as u64) << 32)
            | ((self.data[index + 5] as u64) << 40)
            | ((self.data[index + 6] as u64) << 48)
            | ((self.data[index + 7] as u64) << 56);
    }
}
