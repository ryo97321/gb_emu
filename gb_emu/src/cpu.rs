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

    fn execute(&mut self, opcode: u8) {
        match opcode {
            0x00 => { /* Nothing */ }
            0x80 => { // ADD A, B
                self.regs.a += self.regs.b;
            }
            0x81 => { // ADD A, C
                self.regs.a += self.regs.c;
            }
            0x82 => { // ADD A, D
                self.regs.a += self.regs.d;
            }
            0x83 => { // ADD A, E
                self.regs.a += self.regs.e;
            }
            0x84 => { // ADD A, H
                self.regs.a += self.regs.h;
            }
            0x85 => { // ADD A, L
                self.regs.a += self.regs.l;
            }
            0x86 => { // ADD A, (HL)
                let address = ((self.regs.h as u16) << 8) | (self.regs.l as u16);
                let value = self.mmu.read_byte(address);
                self.regs.a += value;
            }
            0x87 => { // ADD A, A
                self.regs.a += self.regs.a;
            }
            0x88 => { // ADC A, B
                let a = self.regs.a;
                let b = self.regs.b;
                let carry = if self.regs.f & 0x10 != 0 { 1 } else { 0 };

                let result = a.wrapping_add(b).wrapping_add(carry);

                self.regs.f = 0x00;
                if result == 0 {
                    self.regs.f |= 0x80; // Z
                }
                if ((a & 0x0F) + (b & 0x0F) + carry) > 0x0F {
                    self.regs.f |= 0x20; // H
                }
                if ((a as u16) + (b as u16) + (carry as u16)) > 0xFF {
                    self.regs.f |= 0x10; // C
                }

                self.regs.a = result;
            }
            0x89 => { // ADC A, C
                let a = self.regs.a;
                let c = self.regs.c;
                let carry = if self.regs.f & 0x10 != 0 { 1 } else { 0 };

                let result = a.wrapping_add(c).wrapping_add(carry);

                self.regs.f = 0x00;
                if result == 0 {
                    self.regs.f |= 0x80; // Z
                }
                if ((a & 0x0F) + (c & 0x0F) + carry) > 0x0F {
                    self.regs.f |= 0x20; // H
                }
                if ((a as u16) + (c as u16) + (carry as u16)) > 0xFF {
                    self.regs.f |= 0x10; // C
                }

                self.regs.a = result;
            }
            0x8A => { // ADC A, D
                let a = self.regs.a;
                let d = self.regs.d;
                let carry = if self.regs.f & 0x10 != 0 { 1 } else { 0 };

                let result = a.wrapping_add(d).wrapping_add(carry);

                self.regs.f = 0x00;

                if result == 0 {
                    self.regs.f |= 0x80; // Z
                }
                if ((a & 0x0F) + (d & 0x0F) + carry) > 0x0F {
                    self.regs.f |= 0x20; // H
                }
                if ((a as u16) + (d as u16) + (carry as u16)) > 0xFF {
                    self.regs.f |= 0x10; // C
                }

                self.regs.a = result;
            }
            0x8B => { // ADC A, E
                let a = self.regs.a;
                let e = self.regs.e;
                let carry = if self.regs.f & 0x10 != 0 { 1 } else { 0 };

                let result = a.wrapping_add(e).wrapping_add(carry);

                self.regs.f = 0x00;

                if result == 0 {
                    self.regs.f |= 0x80; // Z
                }
                if ((a & 0x0F) + (e & 0x0F) + carry) > 0x0F {
                    self.regs.f |= 0x20; // H
                }
                if ((a as u16) + (e as u16) + (carry as u16)) > 0xFF {
                    self.regs.f |= 0x10; // C
                }

                self.regs.a = result;
            }
            0x8C => { // ADC A, H
                let a = self.regs.a;
                let h = self.regs.h;
                let carry = if self.regs.f & 0x10 != 0 { 1 } else { 0 };

                let result = a.wrapping_add(h).wrapping_add(carry);

                if result == 0 {
                    self.regs.f |= 0x80; // Z
                }
                if ((a & 0x0F) + (h & 0x0F) + carry) > 0x0F {
                    self.regs.f |= 0x20; // H
                }
                if ((a as u16) + (h as u16) + (carry as u16)) > 0xFF {
                    self.regs.f |= 0x10; // C
                }

                self.regs.a = result;
            }
            0x8D => { // ADC A, L
                let a = self.regs.a;
                let l = self.regs.l;
                let carry = if self.regs.f & 0x10 != 0 { 1 } else { 0 };

                let result = a.wrapping_add(l).wrapping_add(carry);

                if result == 0 {
                    self.regs.f |= 0x80; // Z
                }
                if ((a & 0x0F) + (l & 0x0F) + carry) > 0x0F {
                    self.regs.f |= 0x20; // H
                }
                if ((a as u16) + (l as u16) + (carry as u16)) > 0xFF {
                    self.regs.f |= 0x10; // C
                }

                self.regs.a = result;
            }
            0x3E => { // LD A, n (Aレジスタにnをロード)
                let value = self.fetch();
                self.regs.a = value;
            }
            0x06 => { // LD B, n (Bレジスタにnをロード)
                let value = self.fetch();
                self.regs.b = value;
            }
            0x0E => { // LD C, n (Cレジスタにnをロード)
                let value = self.fetch();
                self.regs.c = value;
            }
            0x16 => { // LD D, n (Dレジスタにnをロード)
                let value = self.fetch();
                self.regs.d = value;
            }
            0x1E => { // LD E, n (Eレジスタにnをロード)
                let value = self.fetch();
                self.regs.e = value;
            }
            0x26 => { // LD H, n (Hレジスタにnをロード)
                let value = self.fetch();
                self.regs.h = value;
            }
            0x2E => { // LD L, n (Lレジスタにnをロード)
                let value = self.fetch();
                self.regs.l = value;
            }
            0xC3 => { // JP nn (絶対ジャンプ)
                let low = self.fetch();
                let high = self.fetch();
                self.regs.pc = ((high as u16) << 8) | (low as u16);
            }
            0xC6 => { // ADD A, n (Aレジスタにnを加算)
                let value = self.fetch();
                self.regs.a += value;
            }
            0xD6 => { // SUB A, n (Aレジスタからnを減算)
                let value = self.fetch();
                self.regs.a -= value;
            }
            0x01 => { // LD BC, nn (BCレジスタにnnをロード)
                let low = self.fetch();
                let high = self.fetch();
                self.regs.b = high;
                self.regs.c = low;
            }
            0x11 => { // LD DE, nn (DEレジスタにnnをロード)
                let low = self.fetch();
                let high = self.fetch();
                self.regs.d = high;
                self.regs.e = low;
            }
            0x21 => { // LD HL, nn (HLレジスタにnnをロード)
                let low = self.fetch();
                let high = self.fetch();
                self.regs.h = high;
                self.regs.l = low;
            }
            0x31 => { // LD SP, nn (SPレジスタにnnをロード)
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

