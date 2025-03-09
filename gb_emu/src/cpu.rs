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
            _ => {
                eprintln!("未実装の命令: 0x{:02X}", opcode);
            }
        }
    }
}

