use std::{
    env, fs, thread,
    time::{Duration, Instant},
};

use lrchip8::{
    chip8::Chip8,
    input::{self, Input},
    video::Video,
};
use sdl2::pixels::Color;

const FPS: f64 = 60.0;
const SECS_PER_FRAME: f64 = 1.0 / FPS;
const SCALE_FACTOR: usize = 24;

fn main() {
    let args = env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        println!("Please inform a rom path.");
        return;
    }

    let rom_path = &args[1];

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
        Color::RGB(225, 225, 225),
    );

    let mut input = Input::init(event_pump);

    let rom = fs::read(rom_path).unwrap();

    chip8.load(&rom);

    let secs_per_frame = Duration::from_secs_f64(SECS_PER_FRAME);

    'mainloop: loop {
        let start_time = Instant::now();

        chip8.update_timers();

        for _ in 0..12 { // 720hz
            let keys = input.read();

            if keys[input::KEY_QUIT] {
                break 'mainloop;
            }

            chip8.write_keys(&keys);
            chip8.tick();
        }

        video.draw(chip8.read_video());

        thread::sleep(
            secs_per_frame.saturating_sub(Instant::now().saturating_duration_since(start_time)),
        );
    }
}
