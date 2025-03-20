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

#[derive(PartialEq)]
enum RegisterType {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    BC,
    DE,
    HL,
    SP,
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

    fn ld_a_r16mem(&mut self, r16_high: u8, r16_low: u8) {
        let addr = ((r16_high as u16) << 8) | (r16_low as u16);
        let value = self.mmu.read_byte(addr);
        self.regs.a = value;
    }

    fn get_inc_r16_value(&mut self, r16_high: u8, r16_low: u8) -> u16 {
        let value = ((r16_high as u16) << 8) | (r16_low as u16);
        value.wrapping_add(1)
    }

    fn get_dec_r16_value(&mut self, r16_high: u8, r16_low: u8) -> u16 {
        let value = ((r16_high as u16) << 8) | (r16_low as u16);
        value.wrapping_sub(1)
    }

    fn add_hl_r16(&mut self, r16_high: u8, r16_low: u8, register_type: RegisterType) {
        let value: u16;
        if register_type == RegisterType::SP {
            value = self.regs.sp;
        } else {
            value = ((r16_high as u16) << 8) | (r16_low as u16);
        }
        let hl = ((self.regs.h as u16) << 8) | (self.regs.l as u16);
        let result = hl.wrapping_add(value);

        self.regs.f &= 0x80; // Z以外クリア
        if (hl & 0x0FFF) + (value & 0x0FFF) > 0x0FFF {
            self.regs.f |= 0x20; // H
        }
        if (hl as u32) + (value as u32) > 0xFFFF {
            self.regs.f |= 0x10; // C
        }

        self.regs.h = (result >> 8) as u8;
        self.regs.l = (result & 0xFF) as u8;
    }

    fn inc_r8(&mut self, register_type: RegisterType) {
        let mut value: u8 = 0;
        let mut addr: u16 = 0;
        match register_type {
            RegisterType::A => value = self.regs.a,
            RegisterType::B => value = self.regs.b,
            RegisterType::C => value = self.regs.c,
            RegisterType::D => value = self.regs.d,
            RegisterType::E => value = self.regs.e,
            RegisterType::H => value = self.regs.h,
            RegisterType::L => value = self.regs.l,
            RegisterType::HL => {
                addr = ((self.regs.h as u16) << 8) | (self.regs.l as u16);
                value = self.mmu.read_byte(addr);
            }
            _ => {}
        }
        let result = value.wrapping_add(1);

        self.regs.f &= 0x10; // C以外クリア
        if result == 0x00 {
            self.regs.f |= 0x80; // Z
        }
        if (value & 0x0F) == 0x0F {
            self.regs.f |= 0x20; // H
        }

        match register_type {
            RegisterType::A => self.regs.a = result, 
            RegisterType::B => self.regs.b = result,
            RegisterType::C => self.regs.c = result,
            RegisterType::D => self.regs.d = result,
            RegisterType::E => self.regs.e = result,
            RegisterType::H => self.regs.h = result,
            RegisterType::L => self.regs.l = result,
            RegisterType::HL => self.mmu.write_byte(addr, result),
            _ => {}
        }
    }

    fn dec_r8(&mut self, register_type: RegisterType) {
        let mut value: u8 = 0;
        let mut addr: u16 = 0;
        match register_type {
            RegisterType::A => value = self.regs.a,
            RegisterType::B => value = self.regs.b,
            RegisterType::C => value = self.regs.c,
            RegisterType::D => value = self.regs.d,
            RegisterType::E => value = self.regs.e,
            RegisterType::H => value = self.regs.h,
            RegisterType::L => value = self.regs.l,
            RegisterType::HL => {
                addr = ((self.regs.h as u16) << 8) | (self.regs.l as u16);
                value = self.mmu.read_byte(addr);
            }
            _ => {}
        }
        let result = value.wrapping_sub(1);

        self.regs.f &= 0x10; // C以外クリア
        if result == 0x00 {
            self.regs.f |= 0x80;     // Z
        }
        self.regs.f |= 0x40;         // N
        if (value & 0x10) == 0x10 {
            self.regs.f |= 0x20;     // H
        }

        match register_type {
            RegisterType::A => self.regs.a = result, 
            RegisterType::B => self.regs.b = result,
            RegisterType::C => self.regs.c = result,
            RegisterType::D => self.regs.d = result,
            RegisterType::E => self.regs.e = result,
            RegisterType::H => self.regs.h = result,
            RegisterType::L => self.regs.l = result,
            RegisterType::HL => self.mmu.write_byte(addr, result),
            _ => {}
        }
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

    fn rlca(&mut self) {
        let a = self.regs.a;
        let carry = (a & 0x80) >> 7;

        self.regs.a = (a << 1) | carry;

        self.regs.f = 0x00;
        if carry == 1 {
            self.regs.f |= 0x10;
        }
    }

    fn rrca(&mut self) {
        let a = self.regs.a;
        let carry = a & 0x01;

        self.regs.a = (a >> 1) | (carry << 7);

        self.regs.f = 0x00;
        if carry == 1 {
            self.regs.f |= 0x10;
        }
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
            0x0A => self.ld_a_r16mem(self.regs.b, self.regs.c), // LD A, [BC]
            0x1A => self.ld_a_r16mem(self.regs.d, self.regs.e), // LD A, [DE]
            0x2A => { // LD A, [HL+]
                self.ld_a_r16mem(self.regs.h, self.regs.l);
                let addr = ((self.regs.h as u16) << 8) | (self.regs.l as u16);
                let hl = addr.wrapping_add(1);
                self.regs.h = (hl >> 8) as u8;
                self.regs.l = (hl & 0xFF) as u8;
            }
            0x3A => { // LD A, [HL-]
                self.ld_a_r16mem(self.regs.h, self.regs.l);
                let addr = ((self.regs.h as u16) << 8) | (self.regs.l as u16);
                let hl = addr.wrapping_sub(1);
                self.regs.h = (hl >> 8) as u8;
                self.regs.l = (hl & 0xFF) as u8;
            }
            0x08 => { // LD [imm16], SP
                let low = self.fetch();
                let high = self.fetch();
                let addr = ((high as u16) << 8) | (low as u16);

                let sp_low = (self.regs.sp & 0xFF) as u8;
                let sp_high = (self.regs.sp >> 8) as u8;

                self.mmu.write_byte(addr, sp_low);
                self.mmu.write_byte(addr+1, sp_high);
            }
            0x03 => { // INC BC
                let value = self.get_inc_r16_value(self.regs.b, self.regs.c);
                self.regs.b = (value >> 8) as u8;
                self.regs.c = (value & 0xFF) as u8;
            }
            0x13 => { // INC DE
                let value = self.get_inc_r16_value(self.regs.d, self.regs.e);
                self.regs.d = (value >> 8) as u8;
                self.regs.e = (value & 0xFF) as u8;
            }
            0x23 => { // INC HL
                let value = self.get_inc_r16_value(self.regs.h, self.regs.l);
                self.regs.h = (value >> 8) as u8;
                self.regs.l = (value & 0xFF) as u8;
            }
            0x33 => { // INC SP
                self.regs.sp = self.regs.sp.wrapping_add(1);
            }
            0x0B => { // DEC BC
                let value = self.get_dec_r16_value(self.regs.b, self.regs.c);
                self.regs.b = (value >> 8) as u8;
                self.regs.c = (value & 0xFF) as u8;
            }
            0x1B => { // DEC DE
                let value = self.get_dec_r16_value(self.regs.d, self.regs.e);
                self.regs.d = (value >> 8) as u8;
                self.regs.e = (value & 0xFF) as u8;
            }
            0x2B => { // DEC HL
                let value = self.get_dec_r16_value(self.regs.h, self.regs.l);
                self.regs.h = (value >> 8) as u8;
                self.regs.l = (value & 0xFF) as u8;
            }
            0x3B => self.regs.sp = self.regs.sp.wrapping_sub(1), // DEC SP
            0x09 => self.add_hl_r16(self.regs.b, self.regs.c, RegisterType::BC), // ADD HL, BC
            0x19 => self.add_hl_r16(self.regs.d, self.regs.e, RegisterType::DE), // ADD HL, DE
            0x29 => self.add_hl_r16(self.regs.h, self.regs.l, RegisterType::HL), // ADD HL, HL
            0x39 => self.add_hl_r16(0, 0, RegisterType::SP),                     // ADD HL, SP
            0x3C => self.inc_r8(RegisterType::A),  // INC A
            0x04 => self.inc_r8(RegisterType::B),  // INC B
            0x0C => self.inc_r8(RegisterType::C),  // INC C
            0x14 => self.inc_r8(RegisterType::D),  // INC D
            0x1C => self.inc_r8(RegisterType::E),  // INC E
            0x24 => self.inc_r8(RegisterType::H),  // INC H
            0x2C => self.inc_r8(RegisterType::L),  // INC L
            0x34 => self.inc_r8(RegisterType::HL), // INC [HL]
            0x3D => self.dec_r8(RegisterType::A),  // DEC A
            0x05 => self.dec_r8(RegisterType::B),  // DEC B
            0x0D => self.dec_r8(RegisterType::C),  // DEC C
            0x15 => self.dec_r8(RegisterType::D),  // DEC D
            0x1D => self.dec_r8(RegisterType::E),  // DEC E
            0x25 => self.dec_r8(RegisterType::H),  // DEC H
            0x2D => self.dec_r8(RegisterType::L),  // DEC L
            0x35 => self.dec_r8(RegisterType::HL), // DEC [HL]
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
            0x36 => { // LD [HL], n
                let addr = ((self.regs.h as u16) << 8) | (self.regs.l as u16);
                let value = self.fetch();
                self.mmu.write_byte(addr, value);
            }
            0x07 => self.rlca(), // RLCA
            0x0F => self.rrca(), // RRCA
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

