use crate::mmu::MMU;

// CPUのレジスタ構造
#[derive(Debug)]
pub struct Registers {
    pub a: u8, // アキュムレータ
    pub f: u8, // フラグレジスタ(Z, N, H, C)
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
            0x80 => { // ADD A, n (Aレジスタにnを加算)
                let value = self.fetch();
                self.regs.a += value;
            }
            0x90 => { // SUB A, n (Aレジスタからnを減算)
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

