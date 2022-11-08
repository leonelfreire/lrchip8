use std::{fs, thread, time::Duration};

use lrchip8::{
    chip8::{f, Chip8},
    video::Video,
};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

pub fn main() {
    let mut chip8 = f();
    let rom = fs::read("rom/chiptest-mini.ch8").unwrap();
    let mut video = Video::init();

    chip8.load(&rom);
    loop {
        chip8.tick();
        video.draw(&chip8.gfx);

        thread::sleep(Duration::from_secs_f64(1. / 60.));
    }
    // thread::sleep(Duration::from_secs(10));
    // let mut event_pump = sdl_context.event_pump().unwrap();
    // let mut i = 0;
    // 'running: loop {
    //     i = (i + 1) % 255;
    //     canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
    //     canvas.clear();
    //     for event in event_pump.poll_iter() {
    //         match event {
    //             Event::Quit { .. }
    //             | Event::KeyDown {
    //                 keycode: Some(Keycode::Escape),
    //                 ..
    //             } => break 'running,
    //             _ => {}
    //         }
    //     }
    //     // The rest of the game loop goes here...

    //     canvas.present();
    //     ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    // }
}
