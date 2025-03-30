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
            a: 0x01,
            f: 0xB0, //初期値（ゲームボーイの仕様）
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
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

#[derive(Debug, PartialEq)]
enum ConditionType {
    NZ,
    Z,
    NC,
    C,
}

// LR35902 CPU 定義
pub struct CPU {
    pub regs: Registers, // レジスタ
    pub mmu: MMU,        //メモリ管理ユニット
    pub stopped: bool,
    pub halted: bool,
    pub ime: bool,
}

impl CPU {
    pub fn new(mmu: MMU) -> Self {
        Self {
            regs: Registers::new(),
            mmu,
            stopped: false,
            halted: false,
            ime: true,
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
            self.regs.f |= 0x80; // Z
        }
        self.regs.f |= 0x40; // N
        if (value & 0x10) == 0x10 {
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
            self.regs.f |= 0x10; // C
        }
    }

    fn rrca(&mut self) {
        let a = self.regs.a;
        let carry = a & 0x01;

        self.regs.a = (a >> 1) | (carry << 7);

        self.regs.f = 0x00;
        if carry == 1 {
            self.regs.f |= 0x10; // C
        }
    }

    fn rla(&mut self) {
        let a = self.regs.a;
        let carry = (self.regs.f & 0x10) >> 4;

        let a_msb = (self.regs.a & 0x80) >> 7;

        self.regs.a = (a << 1) | carry;

        self.regs.f = 0x00;
        if a_msb == 1 {
            self.regs.f |= 0x10; // C
        }
    }

    fn rra(&mut self) {
        let a = self.regs.a;
        let carry = (self.regs.f & 0x10) >> 4;

        let a_lsb = self.regs.a & 0x01;

        self.regs.a = (a >> 1) | (carry << 7);

        self.regs.f = 0x00;
        if a_lsb == 1 {
            self.regs.f |= 0x10; // C
        }
    }

    fn daa(&mut self) {
        let mut correction = 0;
        let mut carry = false;

        if (self.regs.f & 0x20 != 0) || (self.regs.a & 0x0F) > 9 {
            correction |= 0x06;
        }

        if (self.regs.f & 0x10 != 0) || (self.regs.a > 0x99) {
            correction |= 0x60;
            carry = true;
        }

        if (self.regs.f & 0x40) == 0 {
            self.regs.a = self.regs.a.wrapping_add(correction);
        } else {
            self.regs.a = self.regs.a.wrapping_sub(correction);
        }

        self.regs.f &= 0x10 | 0x40; // Keep C, N
        if self.regs.a == 0 {
            self.regs.f |= 0x80; // Z
        }
        if carry {
            self.regs.f |= 0x10; // C
        }
        self.regs.f &= !0x20; // Clear H
    }

    fn cpl(&mut self) {
        self.regs.a = !self.regs.a;
        self.regs.f |= 0x60; // Set N, H
    }

    fn scf(&mut self) {
        self.regs.f |= 0x10; // Set C
        self.regs.f &= !0x60; // Reset N, H
    }

    fn ccf(&mut self) {
        // Invert C
        if self.regs.f & 0x10 != 0 {
            self.regs.f &= !0x10; // Reset C
        } else {
            self.regs.f |= 0x10; // Set C
        }

        self.regs.f &= !0x60; // Reset N, H
    }

    fn jr_e8(&mut self) {
        let offset = self.fetch() as i8;
        self.regs.pc = self.regs.pc.wrapping_add(offset as i16 as u16);
    }

    fn jr_cond_e8(&mut self, condition: ConditionType) {
        let offset = self.fetch() as i8;
        let mut add_flg = false;
        match condition {
            ConditionType::NZ => {
                if self.regs.f & 0x80 == 0 {
                    add_flg = true;
                }
            }
            ConditionType::Z => {
                if self.regs.f & 0x80 != 0 {
                    add_flg = true;
                }
            }
            ConditionType::NC => {
                if self.regs.f & 0x10 == 0 {
                    add_flg = true;
                }
            }
            ConditionType::C => {
                if self.regs.f & 0x10 != 0 {
                    add_flg = true;
                }
            }
        }

        if add_flg == true {
            self.regs.pc = self.regs.pc.wrapping_add(offset as u16);
        }
    }

    fn stop(&mut self) {
        println!("CPU STOP");
        self.stopped = true;
    }

    fn handle_interrupts(&mut self) {
        if self.stopped {
            println!("CPU START");
            self.stopped = false;
        }
    }

    fn ld_r8_r8(&mut self, dst_reg: RegisterType, src_reg: RegisterType) {
        let value = match src_reg {
            RegisterType::A => self.regs.a,
            RegisterType::B => self.regs.b,
            RegisterType::C => self.regs.c,
            RegisterType::D => self.regs.d,
            RegisterType::E => self.regs.e,
            RegisterType::H => self.regs.h,
            RegisterType::L => self.regs.l,
            RegisterType::HL => {
                let addr = ((self.regs.h as u16) << 8) | (self.regs.l as u16);
                self.mmu.read_byte(addr)
            }
            _ => 0,
        };
        match dst_reg {
            RegisterType::A => self.regs.a = value,
            RegisterType::B => self.regs.b = value,
            RegisterType::C => self.regs.c = value,
            RegisterType::D => self.regs.d = value,
            RegisterType::E => self.regs.e = value,
            RegisterType::HL => {
                let addr = ((self.regs.h as u16) << 8) | (self.regs.l as u16);
                self.mmu.write_byte(addr, value);
            }
            _ => {}
        }
    }

    fn halt(&mut self) {
        if self.ime == true {
            self.halted = true;
        } else {
            let interrupted_enable = self.mmu.read_byte(0xFFFF);
            let interrupted_flag = self.mmu.read_byte(0xFF0F);
            if interrupted_enable & interrupted_flag != 0 {
                self.halted = false;
            }
        }
    }

    fn execute(&mut self, opcode: u8) {
        match opcode {
            0x00 => { /* Nothing */ }
            0x02 => self.ld_r16mem(self.regs.b, self.regs.c), // LD [BC], A
            0x12 => self.ld_r16mem(self.regs.d, self.regs.e), // LD [DE], A
            0x22 => {
                // LD [HL+], A
                let addr = ((self.regs.h as u16) << 8) | (self.regs.l as u16);
                self.mmu.write_byte(addr, self.regs.a);
                let hl = addr.wrapping_add(1);
                self.regs.h = (hl >> 8) as u8;
                self.regs.l = (hl & 0xFF) as u8;
            }
            0x32 => {
                // LD [HL-], A
                let addr = ((self.regs.h as u16) << 8) | (self.regs.l as u16);
                self.mmu.write_byte(addr, self.regs.a);
                let hl = addr.wrapping_sub(1);
                self.regs.h = (hl >> 8) as u8;
                self.regs.l = (hl & 0xFF) as u8;
            }
            0x0A => self.ld_a_r16mem(self.regs.b, self.regs.c), // LD A, [BC]
            0x1A => self.ld_a_r16mem(self.regs.d, self.regs.e), // LD A, [DE]
            0x2A => {
                // LD A, [HL+]
                self.ld_a_r16mem(self.regs.h, self.regs.l);
                let addr = ((self.regs.h as u16) << 8) | (self.regs.l as u16);
                let hl = addr.wrapping_add(1);
                self.regs.h = (hl >> 8) as u8;
                self.regs.l = (hl & 0xFF) as u8;
            }
            0x3A => {
                // LD A, [HL-]
                self.ld_a_r16mem(self.regs.h, self.regs.l);
                let addr = ((self.regs.h as u16) << 8) | (self.regs.l as u16);
                let hl = addr.wrapping_sub(1);
                self.regs.h = (hl >> 8) as u8;
                self.regs.l = (hl & 0xFF) as u8;
            }
            0x08 => {
                // LD [imm16], SP
                let low = self.fetch();
                let high = self.fetch();
                let addr = ((high as u16) << 8) | (low as u16);

                let sp_low = (self.regs.sp & 0xFF) as u8;
                let sp_high = (self.regs.sp >> 8) as u8;

                self.mmu.write_byte(addr, sp_low);
                self.mmu.write_byte(addr + 1, sp_high);
            }
            0x03 => {
                // INC BC
                let value = self.get_inc_r16_value(self.regs.b, self.regs.c);
                self.regs.b = (value >> 8) as u8;
                self.regs.c = (value & 0xFF) as u8;
            }
            0x13 => {
                // INC DE
                let value = self.get_inc_r16_value(self.regs.d, self.regs.e);
                self.regs.d = (value >> 8) as u8;
                self.regs.e = (value & 0xFF) as u8;
            }
            0x23 => {
                // INC HL
                let value = self.get_inc_r16_value(self.regs.h, self.regs.l);
                self.regs.h = (value >> 8) as u8;
                self.regs.l = (value & 0xFF) as u8;
            }
            0x33 => {
                // INC SP
                self.regs.sp = self.regs.sp.wrapping_add(1);
            }
            0x0B => {
                // DEC BC
                let value = self.get_dec_r16_value(self.regs.b, self.regs.c);
                self.regs.b = (value >> 8) as u8;
                self.regs.c = (value & 0xFF) as u8;
            }
            0x1B => {
                // DEC DE
                let value = self.get_dec_r16_value(self.regs.d, self.regs.e);
                self.regs.d = (value >> 8) as u8;
                self.regs.e = (value & 0xFF) as u8;
            }
            0x2B => {
                // DEC HL
                let value = self.get_dec_r16_value(self.regs.h, self.regs.l);
                self.regs.h = (value >> 8) as u8;
                self.regs.l = (value & 0xFF) as u8;
            }
            0x3B => self.regs.sp = self.regs.sp.wrapping_sub(1), // DEC SP
            0x09 => self.add_hl_r16(self.regs.b, self.regs.c, RegisterType::BC), // ADD HL, BC
            0x19 => self.add_hl_r16(self.regs.d, self.regs.e, RegisterType::DE), // ADD HL, DE
            0x29 => self.add_hl_r16(self.regs.h, self.regs.l, RegisterType::HL), // ADD HL, HL
            0x39 => self.add_hl_r16(0, 0, RegisterType::SP),     // ADD HL, SP
            0x3C => self.inc_r8(RegisterType::A),                // INC A
            0x04 => self.inc_r8(RegisterType::B),                // INC B
            0x0C => self.inc_r8(RegisterType::C),                // INC C
            0x14 => self.inc_r8(RegisterType::D),                // INC D
            0x1C => self.inc_r8(RegisterType::E),                // INC E
            0x24 => self.inc_r8(RegisterType::H),                // INC H
            0x2C => self.inc_r8(RegisterType::L),                // INC L
            0x34 => self.inc_r8(RegisterType::HL),               // INC [HL]
            0x3D => self.dec_r8(RegisterType::A),                // DEC A
            0x05 => self.dec_r8(RegisterType::B),                // DEC B
            0x0D => self.dec_r8(RegisterType::C),                // DEC C
            0x15 => self.dec_r8(RegisterType::D),                // DEC D
            0x1D => self.dec_r8(RegisterType::E),                // DEC E
            0x25 => self.dec_r8(RegisterType::H),                // DEC H
            0x2D => self.dec_r8(RegisterType::L),                // DEC L
            0x35 => self.dec_r8(RegisterType::HL),               // DEC [HL]
            0x80 => self.add_a(self.regs.b),                     // ADD A, B
            0x81 => self.add_a(self.regs.c),                     // ADD A, C
            0x82 => self.add_a(self.regs.d),                     // ADD A, D
            0x83 => self.add_a(self.regs.e),                     // ADD A, E
            0x84 => self.add_a(self.regs.h),                     // ADD A, H
            0x85 => self.add_a(self.regs.l),                     // ADD A, L
            0x86 => {
                // ADD A, (HL)
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
            0x3E => {
                let value = self.fetch();
                self.regs.a = value;
            } // LD A, n
            0x06 => {
                let value = self.fetch();
                self.regs.b = value;
            } // LD B, n
            0x0E => {
                let value = self.fetch();
                self.regs.c = value;
            } // LD C, n
            0x16 => {
                let value = self.fetch();
                self.regs.d = value;
            } // LD D, n
            0x1E => {
                let value = self.fetch();
                self.regs.e = value;
            } // LD E, n
            0x26 => {
                let value = self.fetch();
                self.regs.h = value;
            } // LD H, n
            0x2E => {
                let value = self.fetch();
                self.regs.l = value;
            } // LD L, n
            0x36 => {
                // LD [HL], n
                let addr = ((self.regs.h as u16) << 8) | (self.regs.l as u16);
                let value = self.fetch();
                self.mmu.write_byte(addr, value);
            }
            0x07 => self.rlca(),                                      // RLCA
            0x0F => self.rrca(),                                      // RRCA
            0x17 => self.rla(),                                       // RLA
            0x1F => self.rra(),                                       // RRA
            0x27 => self.daa(),                                       // DAA
            0x2F => self.cpl(),                                       // CPL
            0x37 => self.scf(),                                       // SCF
            0x3F => self.ccf(),                                       // CCF
            0x18 => self.jr_e8(),                                     // JR e8
            0x20 => self.jr_cond_e8(ConditionType::NZ),               // JR NZ, e8
            0x28 => self.jr_cond_e8(ConditionType::Z),                // JR Z, e8
            0x30 => self.jr_cond_e8(ConditionType::NC),               // JR NC, e8
            0x38 => self.jr_cond_e8(ConditionType::C),                // JR C, e8
            0x10 => self.stop(),                                      // STOP
            0x40 => self.ld_r8_r8(RegisterType::B, RegisterType::B),  // LD B, B
            0x41 => self.ld_r8_r8(RegisterType::B, RegisterType::C),  // LD B, C
            0x42 => self.ld_r8_r8(RegisterType::B, RegisterType::D),  // LD B, D
            0x43 => self.ld_r8_r8(RegisterType::B, RegisterType::E),  // LD B, E
            0x44 => self.ld_r8_r8(RegisterType::B, RegisterType::H),  // LD B, H
            0x45 => self.ld_r8_r8(RegisterType::B, RegisterType::L),  // LD B, L
            0x46 => self.ld_r8_r8(RegisterType::B, RegisterType::HL), // LD B, [HL]
            0x47 => self.ld_r8_r8(RegisterType::B, RegisterType::A),  // LD B, A
            0x48 => self.ld_r8_r8(RegisterType::C, RegisterType::B),  // LD C, B
            0x49 => self.ld_r8_r8(RegisterType::C, RegisterType::C),  // LD C, C
            0x4A => self.ld_r8_r8(RegisterType::C, RegisterType::D),  // LD C, D
            0x4B => self.ld_r8_r8(RegisterType::C, RegisterType::E),  // LD C, E
            0x4C => self.ld_r8_r8(RegisterType::C, RegisterType::H),  // LD C, H
            0x4D => self.ld_r8_r8(RegisterType::C, RegisterType::L),  // LD C, L
            0x4E => self.ld_r8_r8(RegisterType::C, RegisterType::HL), // LD C, [HL]
            0x4F => self.ld_r8_r8(RegisterType::C, RegisterType::A),  // LD C, A
            0x50 => self.ld_r8_r8(RegisterType::D, RegisterType::B),  // LD D, B
            0x51 => self.ld_r8_r8(RegisterType::D, RegisterType::C),  // LD D, C
            0x52 => self.ld_r8_r8(RegisterType::D, RegisterType::D),  // LD D, D
            0x53 => self.ld_r8_r8(RegisterType::D, RegisterType::E),  // LD D, E
            0x54 => self.ld_r8_r8(RegisterType::D, RegisterType::H),  // LD D, H
            0x55 => self.ld_r8_r8(RegisterType::D, RegisterType::L),  // LD D, L
            0x56 => self.ld_r8_r8(RegisterType::D, RegisterType::HL), // LD D, [HL]
            0x57 => self.ld_r8_r8(RegisterType::D, RegisterType::A),  // LD D, A
            0x58 => self.ld_r8_r8(RegisterType::E, RegisterType::B),  // LD E, B
            0x59 => self.ld_r8_r8(RegisterType::E, RegisterType::C),  // LD E, C
            0x5A => self.ld_r8_r8(RegisterType::E, RegisterType::D),  // LD E, D
            0x5B => self.ld_r8_r8(RegisterType::E, RegisterType::E),  // LD E, E
            0x5C => self.ld_r8_r8(RegisterType::E, RegisterType::H),  // LD E, H
            0x5D => self.ld_r8_r8(RegisterType::E, RegisterType::L),  // LD E, L
            0x5E => self.ld_r8_r8(RegisterType::E, RegisterType::HL), // LD E, [HL]
            0x5F => self.ld_r8_r8(RegisterType::E, RegisterType::A),  // LD E, A
            0x60 => self.ld_r8_r8(RegisterType::H, RegisterType::B),  // LD H, B
            0x61 => self.ld_r8_r8(RegisterType::H, RegisterType::C),  // LD H, C
            0x62 => self.ld_r8_r8(RegisterType::H, RegisterType::D),  // LD H, D
            0x63 => self.ld_r8_r8(RegisterType::H, RegisterType::E),  // LD H, E
            0x64 => self.ld_r8_r8(RegisterType::H, RegisterType::H),  // LD H, H
            0x65 => self.ld_r8_r8(RegisterType::H, RegisterType::L),  // LD H, L
            0x66 => self.ld_r8_r8(RegisterType::H, RegisterType::HL), // LD H, [HL]
            0x67 => self.ld_r8_r8(RegisterType::H, RegisterType::A),  // LD H, A
            0x68 => self.ld_r8_r8(RegisterType::L, RegisterType::B),  // LD L, B
            0x69 => self.ld_r8_r8(RegisterType::L, RegisterType::C),  // LD L, C
            0x6A => self.ld_r8_r8(RegisterType::L, RegisterType::D),  // LD L, D
            0x6B => self.ld_r8_r8(RegisterType::L, RegisterType::E),  // LD L, E
            0x6C => self.ld_r8_r8(RegisterType::L, RegisterType::H),  // LD L, H
            0x6D => self.ld_r8_r8(RegisterType::L, RegisterType::L),  // LD L, L
            0x6E => self.ld_r8_r8(RegisterType::L, RegisterType::HL), // LD L, [HL]
            0x6F => self.ld_r8_r8(RegisterType::L, RegisterType::A),  // LD L, A
            0x70 => self.ld_r8_r8(RegisterType::HL, RegisterType::B), // LD [HL], B
            0x71 => self.ld_r8_r8(RegisterType::HL, RegisterType::C), // LD [HL], C
            0x72 => self.ld_r8_r8(RegisterType::HL, RegisterType::D), // LD [HL], D
            0x73 => self.ld_r8_r8(RegisterType::HL, RegisterType::E), // LD [HL], E
            0x74 => self.ld_r8_r8(RegisterType::HL, RegisterType::H), // LD [HL], H
            0x75 => self.ld_r8_r8(RegisterType::HL, RegisterType::L), // LD [HL], L
            0x76 => self.halt(),                                      // HALT
            0x77 => self.ld_r8_r8(RegisterType::HL, RegisterType::A), // LD [HL], A
            0x78 => self.ld_r8_r8(RegisterType::A, RegisterType::B),  // LD A, B
            0x79 => self.ld_r8_r8(RegisterType::A, RegisterType::C),  // LD A, C
            0x7A => self.ld_r8_r8(RegisterType::A, RegisterType::D),  // LD A, D
            0x7B => self.ld_r8_r8(RegisterType::A, RegisterType::E),  // LD A, E
            0x7C => self.ld_r8_r8(RegisterType::A, RegisterType::H),  // LD A, H
            0x7D => self.ld_r8_r8(RegisterType::A, RegisterType::L),  // LD A, L
            0x7E => self.ld_r8_r8(RegisterType::A, RegisterType::HL), // LD A, [HL]
            0x7F => self.ld_r8_r8(RegisterType::A, RegisterType::A),  // LD A, A
            0xC3 => {
                // JP nn (絶対ジャンプ)
                let low = self.fetch();
                let high = self.fetch();
                self.regs.pc = ((high as u16) << 8) | (low as u16);
            }
            0xC6 => {
                // ADD A, n
                let value = self.fetch();
                self.regs.a += value;
            }
            0xD6 => {
                // SUB A, n
                let value = self.fetch();
                self.regs.a -= value;
            }
            0x01 => {
                // LD BC, nn
                let low = self.fetch();
                let high = self.fetch();
                self.regs.b = high;
                self.regs.c = low;
            }
            0x11 => {
                // LD DE, nn
                let low = self.fetch();
                let high = self.fetch();
                self.regs.d = high;
                self.regs.e = low;
            }
            0x21 => {
                // LD HL, nn
                let low = self.fetch();
                let high = self.fetch();
                self.regs.h = high;
                self.regs.l = low;
            }
            0x31 => {
                // LD SP, nn
                let low = self.fetch();
                let high = self.fetch();
                self.regs.sp = u16::from_le_bytes([low, high]);
            }
            0xE2 => {
                // LDH (C), A
                let addr = 0xFF00 | (self.regs.c as u16);
                self.mmu.write_byte(addr, self.regs.a);
            }
            0xE0 => {
                // LDH (n), A
                let offset = self.fetch();
                let addr = 0xFF00 | (offset as u16); // 0xFF00 + n
                self.mmu.write_byte(addr, self.regs.a);
            }
            0xEA => {
                // LDH (nn), A
                let low = self.fetch();
                let high = self.fetch();
                let addr = u16::from_le_bytes([low, high]);
                self.mmu.write_byte(addr, self.regs.a);
            }
            0xF2 => {
                // LDH A, (C)
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
