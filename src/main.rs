mod emulator_window;
use winit::event_loop::EventLoop;

use crate::emulator_window::main_window::EmulatorWindow;


fn main() {
    
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(&mut EmulatorWindow::new()).unwrap();

}


