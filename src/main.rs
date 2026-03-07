use std::path::Path;

use chip8_emu::Chip8Emulator;

fn main() {
    let mut chip8 = Chip8Emulator::init();
    chip8.load_game(Path::new("1-chip8-logo.ch8")).unwrap();
    for _ in 0..40 {
        chip8.cycle();
        if chip8.display_flag() {
            for y in 0..32 {
                for x in 0..64 {
                    let pixel = chip8.graphics[x + (y * 64)];
                    if pixel == 1 {
                        print!("█"); // Or "1 "
                    } else {
                        print!(" "); // Or "0 "
                    }
                }
                println!(); // This NEWLINE is critical
            }
        }
    }
}
