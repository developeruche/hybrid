//! The rom module contains the read-only memory structure and implementation to read the memory.
//! ROM includes a device tree blob (DTB) compiled from a device tree source (DTS).

use crate::dtb::dtb;

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
}
