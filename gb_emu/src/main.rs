mod cpu;
mod mmu;

use cpu::CPU;
use mmu::MMU;

fn main() {
    // Initialize Rom DATA
    let mut rom_data = vec![0x00; 0x8000];

    // LD A, 0x49
    rom_data[0x0100] = 0x3E;
    rom_data[0x0101] = 0x49;

    // LDH (0xC000), A; (high: 0xC0, low: 0x00)
    rom_data[0x0102] = 0xEA;
    rom_data[0x0103] = 0x00; // low
    rom_data[0x0104] = 0xC0; // high
    
    // LDH (0xFF81), A; -> 0xFF81 = 0x49
    rom_data[0x0105] = 0xE0;
    rom_data[0x0106] = 0x81;

    // LD C, 0x83; -> C = 0x83
    rom_data[0x0107] = 0x0E;
    rom_data[0x0108] = 0x83;

    // LDH (C), A;
    // LDH (0xFF83), A -> 0xFF83 = 0x49
    rom_data[0x0109] = 0xE2;

    // LD C, 0x85 -> C = 0x85
    rom_data[0x010A] = 0x0E;
    rom_data[0x010B] = 0x85;

    // LDH A, (C)
    // LDH A, (0xFF85)
    rom_data[0x010C] = 0xF2;

    // JP 0x0100; (low: 0x00, high: 0x01)
    rom_data[0x010D] = 0xC3;
    rom_data[0x010E] = 0x00;
    rom_data[0x010F] = 0x01;

    // Make MMU & CPU
    let mmu = MMU::new(rom_data);
    let mut cpu = CPU::new(mmu);

    // Exec ROM
    let n_op = 8; // 命令の数
    for _ in 0..n_op {
        cpu.step();
        println!("A: 0x{:02X}, B: 0x{:02X}, C: 0x{:02X}, D: 0x{:02X}, E: 0x{:02X}, H: 0x{:02X}, L: 0x{:02X}, SP: 0x{:04X}, PC: 0x{:04X}", cpu.regs.a, cpu.regs.b, cpu.regs.c, cpu.regs.d, cpu.regs.e, cpu.regs.h, cpu.regs.l, cpu.regs.sp, cpu.regs.pc);
        println!("0xC000: 0x{:04X}", cpu.mmu.read_byte(0xC000));
        println!("0xFF81: 0x{:04X}", cpu.mmu.read_byte(0xFF81));
        println!("0xFF83: 0x{:04X}", cpu.mmu.read_byte(0xFF83));
        println!("0xFF85: 0x{:02X}", cpu.mmu.read_byte(0xFF85));
        println!("---");
    }
}

