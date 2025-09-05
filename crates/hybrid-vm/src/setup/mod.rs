mod error;
pub use error::{Error, Result};
use rvemu::{bus::DRAM_BASE, dram::DRAM_SIZE, emulator::Emulator};

pub fn setup_from_elf(elf_data: &[u8], call_data: &[u8]) -> Result<Emulator> {
    let elf = goblin::elf::Elf::parse(elf_data)?;

    // Allocate 1MB for the call data
    let mut mem = vec![0; 1024 * 1024];
    {
        assert!(call_data.len() < mem.len() - 8);

        let (size_bytes, data_bytes) = mem.split_at_mut(8);
        size_bytes.copy_from_slice(&(call_data.len() as u64).to_le_bytes());
        data_bytes[..call_data.len()].copy_from_slice(call_data);
    }

    load_sections(&mut mem, &elf, elf_data);

    let mut emu = Emulator::new();

    emu.initialize_dram(mem);
    emu.initialize_pc(elf.header.e_entry);

    Ok(emu)
}

fn load_sections(mem: &mut Vec<u8>, elf: &goblin::elf::Elf, elf_data: &[u8]) {
    for ph in &elf.program_headers {
        if ph.p_type == goblin::elf::program_header::PT_LOAD {
            // The interpreter RAM is DRAM_SIZE starting at DRAM_BASE
            assert!(ph.p_vaddr >= DRAM_BASE);
            assert!(ph.p_memsz <= DRAM_SIZE);

            let start_vec = (ph.p_vaddr - DRAM_BASE) as usize;
            let start_offset = ph.p_offset as usize;

            let end_vec = start_vec + ph.p_memsz as usize;
            if mem.len() < end_vec {
                mem.resize(end_vec, 0);
            }

            // The data available to copy may be smaller than the required size
            let size_to_copy = ph.p_filesz as usize;
            mem[start_vec..(start_vec + size_to_copy)]
                .copy_from_slice(&elf_data[start_offset..(start_offset + size_to_copy)]);
        }
    }
}
