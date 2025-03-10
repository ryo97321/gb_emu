mod cpu;
mod mmu;

use cpu::CPU;
use mmu::MMU;

fn main() {
    // Initialize Rom DATA
    let mut rom_data = vec![0x00; 0x8000];

    // LD A, 0x20
    rom_data[0x0100] = 0x3E;
    rom_data[0x0101] = 0x20;

    // LD B, 0x01
    rom_data[0x0102] = 0x06;
    rom_data[0x0103] = 0x01;

    // LD C, 0x02
    rom_data[0x0104] = 0x0E;
    rom_data[0x0105] = 0x02;

    // LD D, 0x03
    rom_data[0x0106] = 0x16;
    rom_data[0x0107] = 0x03;

    // LD E, 0x04
    rom_data[0x0108] = 0x1E;
    rom_data[0x0109] = 0x04;

    // LD H, 0xFF
    rom_data[0x010A] = 0x26;
    rom_data[0x010B] = 0xFF;

    // LD L, 0x81
    rom_data[0x010C] = 0x2E;
    rom_data[0x010D] = 0x81;

    // LDH (0xFF81), A
    rom_data[0x010E] = 0xEA;
    rom_data[0x010F] = 0x81;
    rom_data[0x0110] = 0xFF;

    // ADD A, (HL)
    rom_data[0x0111] = 0x86;

    // JP 0x0100
    rom_data[0x0112] = 0xC3;
    rom_data[0x0113] = 0x00;
    rom_data[0x0114] = 0x01;

    // Make MMU & CPU
    let mmu = MMU::new(rom_data);
    let mut cpu = CPU::new(mmu);

    // Exec ROM
    let n_op = 10; // 命令の数
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

