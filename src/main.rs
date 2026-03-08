use std::path::Path;

use chip8_emu::{Chip8Emulator, SCREEN_HEIGHT, SCREEN_WIDTH};
use minifb::{Key, Window, WindowOptions};

fn main() {
    let mut chip8 = Chip8Emulator::init();
    chip8.load_game(Path::new("3-corax+.ch8")).unwrap();
    let scale: usize = 15;
    let fb_width: usize = chip8_emu::SCREEN_WIDTH as usize * scale;
    let fb_height: usize = chip8_emu::SCREEN_HEIGHT as usize * scale;

    let mut window = Window::new(
        "Chip8 Emulator",
        fb_width as usize,
        fb_height as usize,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    window.set_target_fps(60);

    let mut buffer = vec![0; fb_width * fb_height];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for _ in 0..10 {
            chip8.cycle();
        }

        if chip8.get_draw_flag() {
            chip8.draw_flag_off();
            for y in 0..SCREEN_HEIGHT as usize {
                let width = SCREEN_WIDTH as usize;
                let y_offset = y * width;
                for x in 0..width {
                    let pixel = chip8.get_display()[y_offset + x];
                    let color = if pixel == 1 { 0xFFFFFF } else { 0x000000 };

                    for row in 0..scale {
                        let render_y = (y * scale) + row;
                        let start_idx = (render_y * 960) + (x * scale);
                        buffer[start_idx..start_idx + scale].fill(color);
                    }
                }
            }

            window
                .update_with_buffer(&buffer, fb_width as usize, fb_height as usize)
                .unwrap();
        } else {
            window.update();
        }
    }
}
