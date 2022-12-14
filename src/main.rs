use std::{
    env, fs, thread,
    time::{self, Duration, Instant, SystemTime},
};

use lrchip8::{
    audio::Audio,
    chip8::Chip8,
    input::{self, Input},
    video::Video,
};
use sdl2::pixels::Color;

const IPS: u32 = 800;
const FPS: f64 = 60.0;

const SECS_PER_FRAME: f64 = 1.0 / FPS;
const ITERS_PER_FRAME: u32 = IPS / FPS as u32;

const VIDEO_SCALE_FACTOR: usize = 12;

fn main() {
    let args = env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        println!("Please inform a rom path.");
        return;
    }

    let rom_path = &args[1];

    let rng_seed = SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH.")
        .as_secs();

    println!("RNG seed: {}", rng_seed);

    let mut chip8 = Chip8::init(rng_seed);
    let rom = fs::read(rom_path).unwrap();

    chip8.load(&rom);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();

    let mut video = Video::init(
        video_subsystem,
        chip8.video_cols(),
        chip8.video_rows(),
        VIDEO_SCALE_FACTOR,
        Color::BLACK,
        Color::RGB(225, 225, 225),
    );

    let audio = Audio::init(audio_subsystem);

    let mut input = Input::init(event_pump);

    let secs_per_frame = Duration::from_secs_f64(SECS_PER_FRAME);

    println!("IPS: {}", ITERS_PER_FRAME * FPS as u32);

    'mainloop: loop {
        let start_time = Instant::now();

        chip8.update_timers();

        for i in 0..ITERS_PER_FRAME {
            let keys = input.read();

            if keys[input::KEY_QUIT] {
                break 'mainloop;
            }

            chip8.write_keys(keys);
            chip8.set_vblank(i == 0);
            chip8.tick();
        }

        audio.play(chip8.audio());
        video.draw(chip8.video());

        thread::sleep(
            secs_per_frame.saturating_sub(Instant::now().saturating_duration_since(start_time)),
        );
    }
}
