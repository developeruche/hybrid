//! The rom module contains the read-only memory structure and implementation to read the memory.
//! ROM includes a device tree blob (DTB) compiled from a device tree source (DTS).

/// RV64EMU read only memory
#[derive(Debug)]
pub struct Rom {
    data: Vec<u8>,
}
