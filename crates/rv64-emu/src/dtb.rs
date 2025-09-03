//! # Device Tree Blob (DTB) Module
//!
//! This module provides functionality for generating, compiling, and reading Device Tree Blobs (DTB) for RISC-V emulation environments.
//!
//! ## Overview
//!
//! The DTB is a binary data structure describing the hardware components of a system, typically compiled from a Device Tree Source (DTS) file. This module automates the creation of a DTS file tailored for a RISC-V virtual platform, compiles it into a DTB using the Device Tree Compiler (`dtc`), and reads the resulting binary content for use in emulation.
//!
//! ## Features
//!
//! - **DTS Generation:** Automatically creates a DTS file (`rvemu.dts`) with a predefined hardware configuration suitable for RISC-V emulation.
//! - **DTB Compilation:** Invokes the external `dtc` tool to compile the DTS file into a DTB file (`rvemu.dtb`).
//! - **DTB Reading:** Reads the compiled DTB file and returns its binary contents for integration into the emulator.
//!
//! ## Usage
//!
//! All main functions are gated behind the `std` feature, as they require filesystem and process management capabilities.
//!
//! - `create_dts()`: Generates a new DTS file, overwriting any existing file.
//! - `compile_dts()`: Compiles the DTS file into a DTB file using `dtc`.
//! - `dtb()`: Orchestrates the creation and compilation process, then reads and returns the DTB binary.
//!
//! ## Notes
//!
//! - The DTS content is currently static and designed for a single-core RISC-V system. Future improvements may allow dynamic generation based on emulator configuration.
//! - The `dtc` tool must be available in the system's PATH for compilation to succeed.
//!
//! ## References
//!
//! - [Device Tree Compiler (dtc)](https://github.com/dgibson/dtc)
//! - [RISC-V ISA Simulator DTS Example](https://github.com/riscv/riscv-isa-sim/blob/66b44bfbedda562a32e4a2cd0716afbf731b69cd/riscv/dts.cc#L38-L54)
//!

use std::fs::File;
use std::io::prelude::*;
use std::io::Write;
use std::process::Command;

pub const DTS_FILE_NAME: &str = "rvemu.dts";
pub const DTB_FILE_NAME: &str = "rvemu.dtb";

/// Creates a new Device Tree Source (DTS) file.
///
/// This function writes a predefined DTS configuration to a file named `rvemu.dts`.
/// If the file already exists, its contents are overwritten. The DTS describes a
/// single-core RISC-V virtual platform, including CPU, memory, UART, virtio, and
/// interrupt controller nodes.
///
/// # Returns
/// * `Ok(())` on success.
/// * `Err(std::io::Error)` if the file cannot be created or written.
///
/// # Notes
/// The DTS content is currently static. Future versions may allow dynamic generation
/// based on emulator configuration.
///
/// # Example
/// ```
/// create_dts()?;
/// ```
pub fn create_dts() -> std::io::Result<()> {
    // TODO: Make this content more flexible depending on the number of cpus.
    // Reference code is https://github.com/riscv/riscv-isa-sim/blob/66b44bfbedda562a32e4a2cd0716afbf731b69cd/riscv/dts.cc#L38-L54
    let content = r#"/dts-v1/;

/ {
    #address-cells = <0x02>;
    #size-cells = <0x02>;
    compatible = "riscv-virtio";
    model = "riscv-virtio,qemu";

    chosen {
        bootargs = "root=/dev/vda ro console=ttyS0";
        stdout-path = "/uart@10000000";
    };

    uart@10000000 {
        interrupts = <0xa>;
        interrupt-parent = <0x03>;
        clock-frequency = <0x384000>;
        reg = <0x0 0x10000000 0x0 0x100>;
        compatible = "ns16550a";
    };

    virtio_mmio@10001000 {
        interrupts = <0x01>;
        interrupt-parent = <0x03>;
        reg = <0x0 0x10001000 0x0 0x1000>;
        compatible = "virtio,mmio";
    };

    cpus {
        #address-cells = <0x01>;
        #size-cells = <0x00>;
        timebase-frequency = <0x989680>;

        cpu-map {
            cluster0 {
                core0 {
                    cpu = <0x01>;
                };
            };
        };

        cpu@0 {
            phandle = <0x01>;
            device_type = "cpu";
            reg = <0x00>;
            status = "okay";
            compatible = "riscv";
            riscv,isa = "rv64imafdcsu";
            mmu-type = "riscv,sv48";

            interrupt-controller {
                #interrupt-cells = <0x01>;
                interrupt-controller;
                compatible = "riscv,cpu-intc";
                phandle = <0x02>;
            };
        };
    };

	memory@80000000 {
		device_type = "memory";
		reg = <0x0 0x80000000 0x0 0x8000000>;
	};

    soc {
        #address-cells = <0x02>;
        #size-cells = <0x02>;
        compatible = "simple-bus";
        ranges;

        interrupt-controller@c000000 {
            phandle = <0x03>;
            riscv,ndev = <0x35>;
            reg = <0x00 0xc000000 0x00 0x4000000>;
            interrupts-extended = <0x02 0x0b 0x02 0x09>;
            interrupt-controller;
            compatible = "riscv,plic0";
            #interrupt-cells = <0x01>;
            #address-cells = <0x00>;
        };

        clint@2000000 {
            interrupts-extended = <0x02 0x03 0x02 0x07>;
            reg = <0x00 0x2000000 0x00 0x10000>;
            compatible = "riscv,clint0";
        };
    };
};"#;

    let mut dts = File::create(DTS_FILE_NAME)?;
    dts.write_all(content.as_bytes())?;
    Ok(())
}

/// Compiles the Device Tree Source (DTS) file into a Device Tree Blob (DTB) file.
///
/// This function invokes the external Device Tree Compiler (`dtc`) to convert
/// `rvemu.dts` into a binary DTB file named `rvemu.dtb`.
///
/// # Returns
/// * `Ok(())` on successful compilation.
/// * `Err(std::io::Error)` if the compilation process fails.
///
/// # Requirements
/// The `dtc` tool must be installed and available in the system's PATH.
///
/// # Example
/// ```
/// compile_dts()?;
/// ```
pub fn compile_dts() -> std::io::Result<()> {
    // dtc -I dts -O dtb -o <FILE_NAME>.dtb <FILE_NAME>.dts
    Command::new("dtc")
        .args(&["-I", "dts", "-O", "dtb", "-o", DTB_FILE_NAME, DTS_FILE_NAME])
        .output()?;
    Ok(())
}

/// Reads the Device Tree Blob (DTB) file and returns its binary contents.
///
/// This function orchestrates the creation of the DTS file, compiles it to DTB,
/// and then reads the DTB file into a byte vector. The resulting binary can be
/// used to initialize or configure a RISC-V emulator.
///
/// # Returns
/// * `Ok(Vec<u8>)` containing the DTB binary data.
/// * `Err(std::io::Error)` if any step fails (DTS creation, compilation, or reading).
///
/// # Example
/// ```
/// let dtb_bytes = dtb()?;
pub fn dtb() -> std::io::Result<Vec<u8>> {
    create_dts()?;
    compile_dts()?;

    let mut dtb = Vec::new();
    File::open(DTB_FILE_NAME)?.read_to_end(&mut dtb)?;

    Ok(dtb)
}
