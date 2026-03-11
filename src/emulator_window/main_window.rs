use std::sync::Arc;

use pixels::{Pixels, SurfaceTexture};
use purple8::{Chip8Emulator, SCREEN_HEIGHT, SCREEN_WIDTH, chip8::chip8_emulator::Chip8Clock};
use winit::{application::ApplicationHandler, dpi::{PhysicalSize, Size}, event::WindowEvent, event_loop::ActiveEventLoop, window::Window};

pub struct EmulatorWindow<'win> {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'win>>,
    chip8: Chip8Emulator,
    clock: Chip8Clock,
}

impl EmulatorWindow<'_> {
    pub fn new() -> Self {
        Self {
            window: None,
            pixels: None,
            chip8: Chip8Emulator::init(),
            clock: Chip8Clock::new(),
        }
    }
    fn init_window(&mut self, event_loop: &ActiveEventLoop){
        let scale = 15;
        let window_width = purple8::SCREEN_WIDTH as u32 * scale;
        let window_height = purple8::SCREEN_HEIGHT as u32 * scale;
        let window = Window::default_attributes()
            .with_title("Purple8")
            .with_inner_size(Size::Physical(PhysicalSize::new(
                window_width,
                window_height,
            ))).with_visible(false);
        
        let window = Arc::new(event_loop.create_window(window).unwrap());
        

        self.window = Some(window.clone());
        let pixels = Pixels::new(
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
            SurfaceTexture::new(window_width, window_height, window),
        )
        .unwrap();
        self.pixels = Some(pixels);
    }
    fn pixels_render(&mut self){
        if let Some(pixels) = &mut self.pixels {
            if self.chip8.get_draw_flag() {
                self.chip8.done_drawing();
                for (i, pixel) in self.chip8.get_display().iter().enumerate() {
                    let color = if *pixel == 1 {
                        [0xff, 0xff, 0xff, 0xff]
                    } else {
                        [0x00, 0x00, 0x00, 0xff]
                    };

                    let slice_start = i * 4;
                    pixels.frame_mut()[slice_start..slice_start + 4]
                        .copy_from_slice(&color);
                }
            }

            if let Err(e) =  pixels.render(){
                panic!("{e}")
            }
        }
    }

}
impl ApplicationHandler for EmulatorWindow<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.init_window(event_loop);
        }
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.pixels_render();

            }
            WindowEvent::KeyboardInput { event, .. } => {
                for (i, key) in self.chip8.get_key_map().iter().enumerate() {
                    if event.physical_key == *key {
                        self.chip8.set_key(i, event.state.is_pressed());
                    }
                }
            }
            WindowEvent::DroppedFile(path)=>self.chip8.load_game(&path).unwrap(),
            WindowEvent::Resized(size)=>self.pixels.as_mut().unwrap().resize_surface(size.width, size.height).unwrap(),
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.set_visible(true);
            for _ in 0..320 {
                self.chip8.cycle();
            }
            if self.clock.tick_chip8_timers() {
                self.chip8.tick_timers();
            }

            if self.chip8.get_draw_flag() {
                window.request_redraw();
            }
        }
    }
    

}

