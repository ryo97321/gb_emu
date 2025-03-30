pub struct MMU {
    rom: Vec<u8>,       // ROM Data
    wram: [u8; 0x2000], // Work RAM (8KB)
    hram: [u8; 0x7F],   // High RAM (127B)
    ie: u8,             // Interrupt Register (0xFFFF)
    interrupt_flag: u8, // Interrupt Flag (0xFF0F)
}

impl MMU {
    // init MMU
    pub fn new(rom_data: Vec<u8>) -> Self {
        Self {
            rom: rom_data,
            wram: [0; 0x2000],
            hram: [0; 0x7F],
            ie: 0,
            interrupt_flag: 0,
        }
    }

    // read Memory
    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.rom[addr as usize], // ROM領域
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize], // WRAM
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize], // WRAM mirror
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize], // HRAM
            0xFFFF => self.ie,                          // 割り込みレジスタ
            0xFF0F => self.interrupt_flag,              // 割り込みフラグ
            _ => {
                eprintln!("Wraning: Read from unmapped memory: 0x{:04X}", addr);
                0xFF // 未定義領域は 0xFF を返す
            }
        }
    }

    // Write Memory
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize] = value, // WRAM
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize] = value, // WRAM mirror
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = value, // HRAM
            0xFFFF => self.ie = value,                                      // 割り込みレジスタ
            0xFF0F => self.interrupt_flag = value,                          // 割り込みフラグ
            _ => {
                eprintln!("Wraning: Write to unmapped memory: 0x{:04X}", addr);
            }
        }
    }
}
