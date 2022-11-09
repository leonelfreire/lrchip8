use std::{fs, thread, time::Duration};

use lrchip8::{chip8::Chip8, video::Video};
use sdl2::pixels::Color;

const SCALE_FACTOR: usize = 24;

fn main() {
    let mut chip8 = Chip8::init();

    let mut video = Video::init(
        chip8.video_cols(),
        chip8.video_rows(),
        SCALE_FACTOR,
        Color::BLACK,
        Color::GRAY,
    );

    let rom = fs::read("rom/chiptest-mini.ch8").unwrap();

    chip8.load(&rom);

    loop {
        chip8.tick();
        video.draw(chip8.video_buffer());
        thread::sleep(Duration::from_secs_f64(10. / 60.));
    }
}
