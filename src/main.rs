use std::{fs, thread, time::Duration};

use lrchip8::chip8::{f, Chip8};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

pub fn main() {
    let mut c = f();
    let f = fs::read("rom/chiptest-mini.ch8").unwrap();
    c.load(&f);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", 1024, 512)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    loop {
        c.tick();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(200, 200, 200));

        let rects = c
            .gfx
            .iter()
            .enumerate()
            .filter_map(|(i, &pixel)| {
                if pixel == 1 {
                    Some(Rect::new(i as i32 % 64 * 16, i as i32 / 64 * 16, 16, 16))
                } else {
                    None
                }
            })
            .collect::<Vec<Rect>>();

        canvas.fill_rects(&rects).unwrap();
        canvas.present();

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
