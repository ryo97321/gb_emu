mod cpu;
mod mmu;

use cpu::CPU;
use mmu::MMU;

fn main() {
    // Initialize Rom DATA
    let mut rom_data = vec![0x00; 0x8000];
   
    // LD A, 0x42;
    rom_data[0x0100] = 0x3E;
    rom_data[0x0101] = 0x42;
    
    // LD B, 0x10;
    rom_data[0x0102] = 0x06;
    rom_data[0x0103] = 0x10;
    
    // ADD A, 0x11;
    rom_data[0x0104] = 0x80;
    rom_data[0x0105] = 0x11;

    // SUB A, 0x11;
    rom_data[0x0106] = 0x90;
    rom_data[0x0107] = 0x11;

    // LD BC, 0x1234
    rom_data[0x0108] = 0x01;
    rom_data[0x0109] = 0x34;
    rom_data[0x010A] = 0x12;

    // LD DE, 0x5678
    rom_data[0x010B] = 0x11;
    rom_data[0x010C] = 0x78;
    rom_data[0x010D] = 0x56;

    // JP 0x0100; (low: 0x00, high: 0x01)
    rom_data[0x010E] = 0xC3;
    rom_data[0x010F] = 0x00;
    rom_data[0x0110] = 0x01;

    // Make MMU & CPU
    let mmu = MMU::new(rom_data);
    let mut cpu = CPU::new(mmu);

    // Exec ROM
    let n_op = 7; // 命令の数
    for _ in 0..n_op {
        cpu.step();
        println!("A: 0x{:02X}, B: 0x{:02X}, C: 0x{:02X}, D: 0x{:02X}, E: 0x{:02X}, PC: 0x{:04X}", cpu.regs.a, cpu.regs.b, cpu.regs.c, cpu.regs.d, cpu.regs.e, cpu.regs.pc);
    }
}

