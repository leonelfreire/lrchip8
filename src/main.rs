use std::{fs, thread, time::Duration};

use lrchip8::{
    chip8::Chip8,
    input::{self, Input},
    video::Video,
};
use sdl2::pixels::Color;

const SCALE_FACTOR: usize = 12;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();

    let mut chip8 = Chip8::init();

    let mut video = Video::init(
        video_subsystem,
        chip8.video_cols(),
        chip8.video_rows(),
        SCALE_FACTOR,
        Color::BLACK,
        Color::WHITE,
    );

    let mut input = Input::init(event_pump);

    let rom = fs::read("rom/test_opcode.ch8").unwrap();

    chip8.load(&rom);

    'main_loop: loop {
        let keys = input.read();

        if keys[input::KEY_QUIT] {
            break 'main_loop;
        }

        chip8.write_keys(&keys);
        chip8.tick();

        video.draw(chip8.read_video());

        thread::sleep(Duration::from_secs_f64(1.0 / 60.));
    }
}
