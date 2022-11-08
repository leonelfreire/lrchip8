use sdl2::{pixels::Color, rect::Rect, render::WindowCanvas};

const BACKGROUND_COLOR: Color = Color::BLACK;
const PIXEL_COLOR: Color = Color::GRAY;
const SCALE_FACTOR: usize = 16;

pub struct Video {
    canvas: WindowCanvas,
    cols: usize,
}

impl Video {
    pub fn init(cols: usize) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("rust-sdl2 demo", 1024, 512)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();

        Self { canvas, cols }
    }

    pub fn draw(&mut self, chip8_buffer: &[u8]) {
        let rects = chip8_buffer
            .into_iter()
            .enumerate()
            .filter_map(|(i, &pixel)| {
                if pixel == 1 {
                    Some(Rect::new(
                        (i % self.cols * SCALE_FACTOR) as i32,
                        (i / self.cols * SCALE_FACTOR) as i32,
                        SCALE_FACTOR as u32,
                        SCALE_FACTOR as u32,
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<Rect>>();

        self.canvas.set_draw_color(BACKGROUND_COLOR);
        self.canvas.clear();

        self.canvas.set_draw_color(PIXEL_COLOR);
        self.canvas.fill_rects(&rects).unwrap();

        self.canvas.present();
    }
}
