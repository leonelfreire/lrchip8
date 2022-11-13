use sdl2::{event::Event, keyboard::Keycode, EventPump};

pub const KEY_QUIT: usize = 16;

const KEYS_SIZE: usize = 17;

pub struct Input {
    event_pump: EventPump,
    keys: [bool; KEYS_SIZE],
}

impl Input {
    pub fn init(event_pump: EventPump) -> Self {
        Self {
            event_pump,
            keys: [false; KEYS_SIZE],
        }
    }

    pub fn read(&mut self) -> &[bool] {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => self.keys[KEY_QUIT] = true,
                Event::KeyDown {
                    keycode: Some(keycode),
                    repeat: false,
                    ..
                } => {
                    if let Some(key) = Input::get_key(keycode) {
                        self.keys[key] = true;
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    repeat: false,
                    ..
                } => {
                    if let Some(key) = Input::get_key(keycode) {
                        self.keys[key] = false;
                    }
                }
                _ => {}
            }
        }

        &self.keys
    }

    fn get_key(keycode: Keycode) -> Option<usize> {
        Some(match keycode {
            Keycode::Num1 => 1,
            Keycode::Num2 => 2,
            Keycode::Num3 => 3,
            Keycode::Q => 4,
            Keycode::W => 5,
            Keycode::E => 6,
            Keycode::A => 7,
            Keycode::S => 8,
            Keycode::D => 9,
            Keycode::Z => 0xA,
            Keycode::X => 0,
            Keycode::C => 0xB,
            Keycode::V => 0xF,
            Keycode::F => 0xE,
            Keycode::R => 0xD,
            Keycode::Num4 => 0xC,
            _ => return None,
        })
    }
}
