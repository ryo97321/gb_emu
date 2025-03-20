mod cpu;
mod mmu;

use cpu::CPU;
use mmu::MMU;

fn main() {

    // Read TEST ROM
    let rom_data = std::fs::read("test_rom/cpu_instrs.gb").expect("Failed open ROM");

    // Make MMU & CPU
    let mmu = MMU::new(rom_data);
    let mut cpu = CPU::new(mmu);

    // Exec ROM
    loop {
        cpu.step();
    }
}

