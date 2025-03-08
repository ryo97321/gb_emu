use std::fs::File;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let rom_path = "pokemon_red.gb";

    let mut file = File::open(rom_path)?;
    let mut buffer = vec![0; 0x150];
    file.read_exact(&mut buffer)?;

    let title_bytes = &buffer[0x134..=0x143];

    println!("ROM Data (0x134-0x143):");
    for byte in title_bytes {
        print!("0x{:02X} ", byte);
    }
    println!();

    let title = String::from_utf8_lossy(title_bytes)
        .trim_matches(char::from(0))
        .to_string();
    println!("Game Title: {}", title);

    Ok(())
}

