use sdl2::{self, render::Canvas, video::Window};

pub struct Renderer {
    canvas: Canvas<Window>,
}

impl Renderer {
    pub fn new() -> Self {
        // Init SDL2
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem.window("NES", 256, 240).build().unwrap();
        let canvas = window.into_canvas().build().unwrap();

        Renderer { canvas }
    }
}
