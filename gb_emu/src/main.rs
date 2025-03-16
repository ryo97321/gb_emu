mod cpu;
mod mmu;

use cpu::CPU;
use mmu::MMU;

fn main() {
    // Initialize Rom DATA
    let mut rom_data = vec![0x00; 0x8000];

    // NOP
    rom_data[0x0100] = 0x00;

    // DEC BC
    rom_data[0x0101] = 0x0B;

    // DEC DE
    rom_data[0x0102] = 0x1B;

    // DEC HL
    rom_data[0x0103] = 0x2B;

    // DEC SP
    rom_data[0x0104] = 0x3B;

    // JP 0x0100
    rom_data[0x0105] = 0xC3;
    rom_data[0x0106] = 0x00;
    rom_data[0x0107] = 0x01;

    // Make MMU & CPU
    let mmu = MMU::new(rom_data);
    let mut cpu = CPU::new(mmu);

    cpu.regs.b = 0xFF;
    cpu.regs.c = 0x01;

    cpu.regs.d = 0xEE;
    cpu.regs.e = 0x02;

    cpu.regs.h = 0xDD;
    cpu.regs.l = 0x03;

    cpu.regs.sp = 0xCC04;

    // Exec ROM
    let n_op = 6; // 命令の数
    for _ in 0..n_op {
        cpu.step();
        println!("A: 0x{:02X}, B: 0x{:02X}, C: 0x{:02X}, D: 0x{:02X}, E: 0x{:02X}, H: 0x{:02X}, L: 0x{:02X}, SP: 0x{:04X}, PC: 0x{:04X}", cpu.regs.a, cpu.regs.b, cpu.regs.c, cpu.regs.d, cpu.regs.e, cpu.regs.h, cpu.regs.l, cpu.regs.sp, cpu.regs.pc);
        println!("F: 0x{:02X}", cpu.regs.f);
        println!("0xC000: 0x{:04X}", cpu.mmu.read_byte(0xC000));
        println!("0xFF81: 0x{:04X}", cpu.mmu.read_byte(0xFF81));
        println!("0xFF82: 0x{:04X}", cpu.mmu.read_byte(0xFF82));
        println!("0xFFFF: 0x{:02X}", cpu.mmu.read_byte(0xFFFF));
        println!("---");
    }
}

