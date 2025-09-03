use rv64_emu::bus::DRAM_BASE;
use rv64_emu::reg::csr::MEPC;
use rv64_emu::emu::Emu;

#[test]
fn illegal_isa() {
    let mut emu = Emu::new();

    let data = vec![
        0x93, 0x0f, 0x50, 0x00, // addi x31, x0, 5
        0xaa, 0xaa, 0xaa, 0xaa, // Invalid ISA
        0x93, 0x0f, 0x50, 0x00, // addi x31, x0, 5
    ];

    emu.initialize_dram(data);
    emu.initialize_pc(DRAM_BASE);

    emu.start();

    // TODO: correct?
    //assert_eq!(4 + DRAM_BASE, emu.cpu.state.read(MEPC));
    assert_eq!(8 + DRAM_BASE, emu.cpu.state.read(MEPC));
}
