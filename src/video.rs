use sdl2::{pixels::Color, rect::Rect, render::WindowCanvas, VideoSubsystem};

const WINDOW_TITLE: &str = "lrchip8";

pub struct Video {
    canvas: WindowCanvas,
    cols: usize,
    scale_factor: usize,
    bg_color: Color,
    pxl_color: Color,
}

impl Video {
    pub fn init(
        video_subsystem: VideoSubsystem,
        cols: usize,
        rows: usize,
        scale_factor: usize,
        bg_color: Color,
        pxl_color: Color,
    ) -> Self {
        let width = (cols * scale_factor) as u32;
        let height = (rows * scale_factor) as u32;

        let window = video_subsystem
            .window(WINDOW_TITLE, width, height)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();

        Self {
            canvas,
            cols,
            scale_factor,
            bg_color,
            pxl_color,
        }
    }

    pub fn draw(&mut self, chip8_video: &[u8]) {
        let rects = chip8_video
            .iter()
            .enumerate()
            .filter_map(|(i, &pixel)| {
                if pixel == 1 {
                    Some(Rect::new(
                        ((i % self.cols) * self.scale_factor) as i32,
                        ((i / self.cols) * self.scale_factor) as i32,
                        self.scale_factor as u32,
                        self.scale_factor as u32,
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<Rect>>();

        self.canvas.set_draw_color(self.bg_color);
        self.canvas.clear();

        self.canvas.set_draw_color(self.pxl_color);
        self.canvas.fill_rects(&rects).unwrap();

        self.canvas.present();
    }
}
