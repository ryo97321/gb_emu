use crate::mmu::MMU;

// CPUのレジスタ構造
#[derive(Debug)]
pub struct Registers {
    pub a: u8, // アキュムレータ
    pub f: u8, // フラグレジスタ(0bZNHC0000)
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16, // スタックポインタ
    pub pc: u16, // プログラムカウンタ
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0x01, f: 0xB0, //初期値（ゲームボーイの仕様）
            b: 0x00, c: 0x13,
            d: 0x00, e: 0xD8,
            h: 0x01, l: 0x4D,
            sp: 0xFFFE,
            pc: 0x0100, //ROMのエントリーポイント
        }
    }
}

// LR35902 CPU 定義
pub struct CPU {
    pub regs: Registers, // レジスタ
    pub mmu: MMU,        //メモリ管理ユニット
}

impl CPU {
    pub fn new(mmu: MMU) -> Self {
        Self {
            regs: Registers::new(),
            mmu,
        }
    }

    // CPUを1クロック実行
    pub fn step(&mut self) {
        let opcode = self.fetch();
        self.execute(opcode);
    }

    // 命令フェッチ
    fn fetch(&mut self) -> u8 {
        let opcode = self.mmu.read_byte(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        opcode
    }

    fn ld_r16mem(&mut self, r16_high: u8, r16_low: u8) {
        let addr = ((r16_high as u16) << 8) | (r16_low as u16);
        self.mmu.write_byte(addr, self.regs.a);
    }

    fn add_a(&mut self, r8_value: u8) {
        self.regs.a += r8_value;
    }

    fn adc_a(&mut self, r8_value: u8) {
        let a = self.regs.a;
        let carry = if self.regs.f & 0x10 != 0 { 1 } else { 0 };

        let result = a.wrapping_add(r8_value).wrapping_add(carry);

        self.regs.f = 0x00;
        if result == 0 {
            self.regs.f |= 0x80; // Z
        }
        if ((a & 0x0F) + (r8_value & 0x0F) + carry) > 0x0F {
            self.regs.f |= 0x20; // H
        }
        if ((a as u16) + (r8_value as u16) + (carry as u16)) > 0xFF {
            self.regs.f |= 0x10; // C
        }

        self.regs.a = result;
    }

    fn execute(&mut self, opcode: u8) {
        match opcode {
            0x00 => { /* Nothing */ }
            0x02 => self.ld_r16mem(self.regs.b, self.regs.c), // LD [BC], A
            0x12 => self.ld_r16mem(self.regs.d, self.regs.e), // LD [DE], A
            0x22 => { // LD [HL+], A
                let addr = ((self.regs.h as u16) << 8) | (self.regs.l as u16);
                self.mmu.write_byte(addr, self.regs.a);
                let hl = addr.wrapping_add(1);
                self.regs.h = (hl >> 8) as u8;
                self.regs.l = (hl & 0xFF) as u8;
            }
            0x32 => { // LD [HL-], A
                let addr = ((self.regs.h as u16) << 8) | (self.regs.l as u16);
                self.mmu.write_byte(addr, self.regs.a);
                let hl = addr.wrapping_sub(1);
                self.regs.h = (hl >> 8) as u8;
                self.regs.l = (hl & 0xFF) as u8;
            }
            0x80 => self.add_a(self.regs.b), // ADD A, B
            0x81 => self.add_a(self.regs.c), // ADD A, C
            0x82 => self.add_a(self.regs.d), // ADD A, D
            0x83 => self.add_a(self.regs.e), // ADD A, E
            0x84 => self.add_a(self.regs.h), // ADD A, H
            0x85 => self.add_a(self.regs.l), // ADD A, L
            0x86 => { // ADD A, (HL)
                let address = ((self.regs.h as u16) << 8) | (self.regs.l as u16);
                let value = self.mmu.read_byte(address);
                self.regs.a += value;
            }
            0x87 => self.add_a(self.regs.a), // ADD A, A
            0x88 => self.adc_a(self.regs.b), // ADC A, B
            0x89 => self.adc_a(self.regs.c), // ADC A, C
            0x8A => self.adc_a(self.regs.d), // ADC A, D
            0x8B => self.adc_a(self.regs.e), // ADC A, E
            0x8C => self.adc_a(self.regs.h), // ADC A, H
            0x8D => self.adc_a(self.regs.l), // ADC A, L
            0x3E => { let value = self.fetch(); self.regs.a = value; } // LD A, n
            0x06 => { let value = self.fetch(); self.regs.b = value; } // LD B, n
            0x0E => { let value = self.fetch(); self.regs.c = value; } // LD C, n
            0x16 => { let value = self.fetch(); self.regs.d = value; } // LD D, n
            0x1E => { let value = self.fetch(); self.regs.e = value; } // LD E, n
            0x26 => { let value = self.fetch(); self.regs.h = value; } // LD H, n
            0x2E => { let value = self.fetch(); self.regs.l = value; } // LD L, n
            0xC3 => { // JP nn (絶対ジャンプ)
                let low = self.fetch();
                let high = self.fetch();
                self.regs.pc = ((high as u16) << 8) | (low as u16);
            }
            0xC6 => { // ADD A, n
                let value = self.fetch();
                self.regs.a += value;
            }
            0xD6 => { // SUB A, n
                let value = self.fetch();
                self.regs.a -= value;
            }
            0x01 => { // LD BC, nn
                let low = self.fetch();
                let high = self.fetch();
                self.regs.b = high;
                self.regs.c = low;
            }
            0x11 => { // LD DE, nn
                let low = self.fetch();
                let high = self.fetch();
                self.regs.d = high;
                self.regs.e = low;
            }
            0x21 => { // LD HL, nn
                let low = self.fetch();
                let high = self.fetch();
                self.regs.h = high;
                self.regs.l = low;
            }
            0x31 => { // LD SP, nn
                let low = self.fetch();
                let high = self.fetch();
                self.regs.sp = u16::from_le_bytes([low, high]);
            }
            0xE2 => { // LDH (C), A
                let addr = 0xFF00 | (self.regs.c as u16);
                self.mmu.write_byte(addr, self.regs.a);
            }
            0xE0 => { // LDH (n), A
                let offset = self.fetch();
                let addr = 0xFF00 | (offset as u16); // 0xFF00 + n
                self.mmu.write_byte(addr, self.regs.a);
            }
            0xEA => { // LDH (nn), A
                let low = self.fetch();
                let high = self.fetch();
                let addr = u16::from_le_bytes([low, high]);
                self.mmu.write_byte(addr, self.regs.a);
            }
            0xF2 => { // LDH A, (C)
                let addr = 0xFF00 | (self.regs.c as u16);
                let value = self.mmu.read_byte(addr);
                self.regs.a = value;
            }
            _ => {
                eprintln!("未実装の命令: 0x{:02X}", opcode);
            }
        }
    }
}

