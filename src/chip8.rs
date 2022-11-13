use std::fmt::format;

use crate::{dec_addr, dec_byte, dec_error, dec_nibble, dec_x, dec_y};

const NUM_REGS: usize = 16;

const STACK_SIZE: usize = 16;

const MEM_SIZE: usize = 4096;

const PROG_START_ADDR: usize = 0x200;

const VIDEO_COLS: usize = 64;
const VIDEO_ROWS: usize = 32;
const VIDEO_SIZE: usize = VIDEO_COLS * VIDEO_ROWS;

const KEYS_SIZE: usize = 16;

const FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8 {
    v: [u8; NUM_REGS],
    i: u16,
    pc: u16,
    sp: u8,
    stack: [u16; STACK_SIZE],
    mem: [u8; MEM_SIZE],
    video: [u8; VIDEO_SIZE],
    keys: [bool; KEYS_SIZE],
    delay_t: u8,
    buzzer_t: u8,
    wait_for_key: Option<u8>,
    schip_shift: bool,
    schip_load: bool,
}

impl Chip8 {
    pub fn init() -> Self {
        let mut mem = [0u8; MEM_SIZE];

        let font_area = &mut mem[..FONT_SET.len()];
        font_area.copy_from_slice(&FONT_SET);

        Self {
            v: [0u8; NUM_REGS],
            i: 0,
            pc: 0,
            sp: 0,
            stack: [0u16; STACK_SIZE],
            mem,
            video: [0u8; VIDEO_SIZE],
            keys: [false; KEYS_SIZE],
            delay_t: 0,
            buzzer_t: 0,
            wait_for_key: None,
            schip_shift: true,
            schip_load: true,
        }
    }

    pub fn video_cols(&self) -> usize {
        VIDEO_COLS
    }

    pub fn video_rows(&self) -> usize {
        VIDEO_ROWS
    }

    pub fn read_video(&self) -> &[u8] {
        &self.video
    }

    pub fn write_keys(&mut self, keys: &[bool]) {
        self.keys.copy_from_slice(&keys[..KEYS_SIZE]);
    }

    pub fn update_timers(&mut self) {
        self.delay_t = self.delay_t.saturating_sub(1);
        self.buzzer_t = self.buzzer_t.saturating_sub(1);
    }

    pub fn load(&mut self, rom: &[u8]) {
        println!("Loading program ({} bytes)...", rom.len());

        if let Some(program_area) = self
            .mem
            .get_mut(PROG_START_ADDR..(PROG_START_ADDR + rom.len()))
        {
            program_area.copy_from_slice(rom);
            println!("{} bytes loaded.", rom.len());
        } else {
            panic!("The program is too big to fit in memory.");
        }

        self.pc = PROG_START_ADDR as u16;
    }

    pub fn load16(&mut self, rom16: &[u16]) {
        let rom8 = rom16
            .iter()
            .flat_map(|&w| [(w >> 8) as u8, (w & 0x00FF) as u8].into_iter())
            .collect::<Vec<u8>>();

        self.load(&rom8);
    }

    pub fn tick(&mut self) {
        let opcode = self.fetch();

        println!(
            "[OP=0x{:0>4X}] [I=0x{:0>4X}, PC=0x{:0>4X}, SP=0x{:0>4X}, V={:?}]",
            opcode, self.i, self.pc, self.sp, self.v
        );

        self.execute(opcode);
    }

    fn fetch(&mut self) -> u16 {
        let pc = self.pc as usize;
        let opcode = (self.mem[pc] as u16) << 8 | self.mem[pc + 1] as u16;

        self.pc += 2;

        opcode
    }

    fn execute(&mut self, opcode: u16) {
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => self.op_00e0(),
                0x00EE => self.op_00ee(),
                _ => self.op_0nnn(),
            },
            0x1000 => self.op_1nnn(dec_addr!(opcode)),
            0x2000 => self.op_2nnn(dec_addr!(opcode)),
            0x3000 => self.op_3xnn(dec_x!(opcode), dec_byte!(opcode)),
            0x4000 => self.op_4xnn(dec_x!(opcode), dec_byte!(opcode)),
            0x5000 => self.op_5xy0(dec_x!(opcode), dec_y!(opcode)),
            0x6000 => self.op_6xnn(dec_x!(opcode), dec_byte!(opcode)),
            0x7000 => self.op_7xnn(dec_x!(opcode), dec_byte!(opcode)),
            0x8000 => match opcode & 0x000F {
                0x0000 => self.op_8xy0(dec_x!(opcode), dec_y!(opcode)),
                0x0001 => self.op_8xy1(dec_x!(opcode), dec_y!(opcode)),
                0x0002 => self.op_8xy2(dec_x!(opcode), dec_y!(opcode)),
                0x0003 => self.op_8xy3(dec_x!(opcode), dec_y!(opcode)),
                0x0004 => self.op_8xy4(dec_x!(opcode), dec_y!(opcode)),
                0x0005 => self.op_8xy5(dec_x!(opcode), dec_y!(opcode)),
                0x0006 => self.op_8xy6(dec_x!(opcode), dec_y!(opcode)),
                0x0007 => self.op_8xy7(dec_x!(opcode), dec_y!(opcode)),
                0x000E => self.op_8xye(dec_x!(opcode), dec_y!(opcode)),
                _ => dec_error!(opcode),
            },
            0x9000 => self.op_9xy0(dec_x!(opcode), dec_y!(opcode)),
            0xA000 => self.op_annn(dec_addr!(opcode)),
            0xB000 => self.op_bnnn(dec_addr!(opcode)),
            0xC000 => self.op_cxnn(dec_x!(opcode), dec_byte!(opcode)),
            0xD000 => self.op_dxyn(dec_x!(opcode), dec_y!(opcode), dec_nibble!(opcode)),
            0xE000 => match opcode & 0x00FF {
                0x009E => self.op_ex9e(dec_x!(opcode)),
                0x00A1 => self.op_exa1(dec_x!(opcode)),
                _ => dec_error!(opcode),
            },
            0xF000 => match opcode & 0x00FF {
                0x0007 => self.op_fx07(dec_x!(opcode)),
                0x000A => self.op_fx0a(dec_x!(opcode)),
                0x0015 => self.op_fx15(dec_x!(opcode)),
                0x0018 => self.op_fx18(dec_x!(opcode)),
                0x001E => self.op_fx1e(dec_x!(opcode)),
                0x0029 => self.op_fx29(dec_x!(opcode)),
                0x0033 => self.op_fx33(dec_x!(opcode)),
                0x0055 => self.op_fx55(dec_x!(opcode)),
                0x0065 => self.op_fx65(dec_x!(opcode)),
                _ => dec_error!(opcode),
            },
            _ => dec_error!(opcode),
        }
    }

    // 00E0
    // Clear the screen.
    fn op_00e0(&mut self) {
        self.video.fill(0);
    }

    // 00EE
    // Return from a subroutine.
    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    // 0NNN
    // Jump to a machine code routine at NNN.
    // Ignored by modern interpreters.
    fn op_0nnn(&self) {}

    // 1NNN
    // Jump to address NNN.
    fn op_1nnn(&mut self, addr: u16) {
        self.pc = addr;
    }

    // 2NNN
    // Execute subroutine starting at address NNN.
    fn op_2nnn(&mut self, addr: u16) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = addr;
    }

    // 3XNN
    // Skip the following instruction if the value of register VX equals NN.
    fn op_3xnn(&mut self, x: usize, nn: u8) {
        if self.v[x] == nn {
            self.pc += 2;
        }
    }

    // 4XNN
    // Skip the following instruction if the value of register VX is not equal to NN.
    fn op_4xnn(&mut self, x: usize, nn: u8) {
        if self.v[x] != nn {
            self.pc += 2;
        }
    }

    // 5XY0
    // Skip the following instruction if the value of register VX is equal to the value of register VY.
    fn op_5xy0(&mut self, x: usize, y: usize) {
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }

    // 6XNN
    // Store number NN in register VX.
    fn op_6xnn(&mut self, x: usize, nn: u8) {
        self.v[x] = nn;
    }

    // 7XNN
    // Add the value NN to register VX.
    fn op_7xnn(&mut self, x: usize, nn: u8) {
        self.v[x] = self.v[x].wrapping_add(nn);
    }

    // 8XY0
    // Store the value of register VY in register VX.
    fn op_8xy0(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
    }

    // 8XY1
    // Set VX to VX OR VY.
    fn op_8xy1(&mut self, x: usize, y: usize) {
        self.v[x] |= self.v[y];
    }

    // 8XY2
    // Set VX to VX AND VY.
    fn op_8xy2(&mut self, x: usize, y: usize) {
        self.v[x] &= self.v[y];
    }

    // 8XY3
    // Set VX to VX XOR VY.
    fn op_8xy3(&mut self, x: usize, y: usize) {
        self.v[x] ^= self.v[y];
    }

    // 8XY4
    // Add the value of register VY to register VX.
    // Set VF to 01 if a carry occurs.
    // Set VF to 00 if a carry does not occur.
    fn op_8xy4(&mut self, x: usize, y: usize) {
        let (sum, carry) = self.v[x].overflowing_add(self.v[y]);

        self.v[x] = sum;
        self.v[0xF] = carry.into();
    }

    // 8XY5
    // Subtract the value of register VY from register VX.
    // Set VF to 00 if a borrow occurs.
    // Set VF to 01 if a borrow does not occur
    fn op_8xy5(&mut self, x: usize, y: usize) {
        let (sub, borrow) = self.v[x].overflowing_sub(self.v[y]);

        self.v[x] = sub;
        self.v[0xF] = (!borrow).into();
    }

    // 8XY6
    //
    // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0.
    // Then Vx is divided by 2.
    //
    // or
    //
    // Store the value of register VY shifted right one bit in register VX.
    // Set register VF to the least significant bit prior to the shift.
    // VY is unchanged
    fn op_8xy6(&mut self, x: usize, y: usize) {
        if self.schip_shift {
            self.v[0xF] = self.v[x] & 1;
            self.v[x] >>= 1;
        } else {
            self.v[0xF] = self.v[y] & 1;
            self.v[x] = self.v[y] >> 1;
        }
    }

    // 8XY7
    // Set register VX to the value of VY minus VX.
    // Set VF to 00 if a borrow occurs.
    // Set VF to 01 if a borrow does not occur.
    fn op_8xy7(&mut self, x: usize, y: usize) {
        let (sub, borrow) = self.v[y].overflowing_sub(self.v[x]);

        self.v[x] = sub;
        self.v[0xF] = (!borrow).into();
    }

    // 8XYE
    //
    // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0.
    // Then Vx is multiplied by 2.
    //
    // or
    //
    // Store the value of register VY shifted left one bit in register VX.
    // Set register VF to the most significant bit prior to the shift.
    // VY is unchanged.
    fn op_8xye(&mut self, x: usize, y: usize) {
        if self.schip_shift {
            self.v[0xF] = (self.v[x] >> 7) & 1;
            self.v[x] <<= 1;
        } else {
            self.v[0xF] = (self.v[y] >> 7) & 1;
            self.v[x] = self.v[y] << 1;
        }
    }

    // 9XY0
    // Skip the following instruction if the value of register VX is not equal to the value of register VY.
    fn op_9xy0(&mut self, x: usize, y: usize) {
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
    }

    // ANNN
    // Store memory address NNN in register I.
    fn op_annn(&mut self, addr: u16) {
        self.i = addr;
    }

    // BNNN
    // Jump to address NNN + V0.
    fn op_bnnn(&mut self, nnn: u16) {
        self.pc = nnn + self.v[0] as u16;
    }

    // CXNN
    // Set VX to a random number with a mask of NN.
    fn op_cxnn(&mut self, x: usize, nn: u8) {
        self.v[x] = fastrand::u8(0..=255) & nn;
    }

    // DXYN
    // Draw a sprite at position VX, VY with N bytes of sprite data starting at the address stored in I.
    // Set VF to 01 if any set pixels are changed to unset, and 00 otherwise.
    fn op_dxyn(&mut self, x: usize, y: usize, n: u8) {
        // Wrap x and y.
        let x = self.v[x] as usize % VIDEO_COLS;
        let y = self.v[y] as usize % VIDEO_ROWS;

        // Memory area.
        let addr_start = self.i as usize;
        let n = n as usize;

        // Clip rows.
        let addr_end = if (y + n) < VIDEO_ROWS {
            addr_start + n
        } else {
            addr_start + (VIDEO_ROWS - y)
        };

        // Clip cols.
        let bit_max = if (x + 8) < VIDEO_COLS {
            8
        } else {
            VIDEO_COLS - x
        };

        self.v[0xF] = 0;

        for (y_ofst, addr) in (addr_start..addr_end).enumerate() {
            for i in 0..bit_max {
                if (self.mem[addr] & (0x80 >> i)) != 0 {
                    let pixel_pos = ((y + y_ofst) * VIDEO_COLS) + x + i;

                    if self.video[pixel_pos] == 1 {
                        self.v[0xF] = 1;
                    }

                    self.video[pixel_pos] ^= 1;
                }
            }
        }
    }

    // EX9E
    // Skip the following instruction if the key corresponding to the hex value currently stored in register VX is pressed.
    fn op_ex9e(&mut self, x: usize) {
        let vx = self.v[x] as usize;

        if self.keys[vx] == true {
            self.pc += 2;
        }
    }

    // EXA1
    // Skip the following instruction if the key corresponding to the hex value currently stored in register VX is not pressed.
    fn op_exa1(&mut self, x: usize) {
        let vx = self.v[x] as usize;

        if self.keys[vx] == false {
            self.pc += 2;
        }
    }

    // FX07
    // Store the current value of the delay timer in register VX.
    fn op_fx07(&mut self, x: usize) {
        self.v[x] = self.delay_t;
    }

    // FX0A
    // Wait for a keypress and store the result in register VX.
    fn op_fx0a(&mut self, x: usize) {
        println!("Waiting for key...");

        if let Some(key) = self.wait_for_key {
            if !self.keys[key as usize] {
                println!("Got key {:?}", self.wait_for_key);
                self.v[x] = key;
                self.wait_for_key = None;
                return;
            }
        } else if let Some(key) = self.keys.iter().position(|&k| k) {
            self.keys[key] = true;
            self.wait_for_key = Some(key as u8);
        }

        // Try again.
        self.pc -= 2;
    }

    // FX15
    // Set the delay timer to the value of register VX.
    fn op_fx15(&mut self, x: usize) {
        self.delay_t = self.v[x];
    }

    // FX18
    // Set the sound timer to the value of register VX.
    fn op_fx18(&mut self, x: usize) {
        self.buzzer_t = self.v[x];
    }

    // FX1E
    // Add the value stored in register VX to register I.
    fn op_fx1e(&mut self, x: usize) {
        let vx = self.v[x] as u16;

        self.i = self.i.wrapping_add(vx);
        self.v[0xF] = if self.i > 0xFFF { 1 } else { 0 };
    }

    // FX29
    // Set I to the memory address of the sprite data corresponding to the hexadecimal digit stored in register VX.
    fn op_fx29(&mut self, x: usize) {
        let vx = self.v[x] as u16;

        self.i = vx * 5;
    }

    // FX33
    // Store the binary-coded decimal equivalent of the value stored in register VX at addresses I, I + 1, and I + 2.
    fn op_fx33(&mut self, x: usize) {
        let n = self.v[x];
        let i = self.i as usize;

        self.mem[i] = n / 100;
        self.mem[i + 1] = n % 100 / 10;
        self.mem[i + 2] = n % 10;
    }

    // FX55
    // Store the values of registers V0 to VX inclusive in memory starting at address I.
    // I is set to I + X + 1 after operation.
    fn op_fx55(&mut self, x: usize) {
        for i in 0..=x {
            self.mem[self.i as usize + i] = self.v[i];
        }

        if !self.schip_load {
            self.i += (x + 1) as u16;
        }
    }

    // FX65
    // Fill registers V0 to VX inclusive with the values stored in memory starting at address I.
    // I is set to I + X + 1 after operation.
    fn op_fx65(&mut self, x: usize) {
        for i in 0..=x {
            self.v[i] = self.mem[self.i as usize + i];
        }

        if !self.schip_load {
            self.i += (x + 1) as u16;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Chip8;

    fn load_chip8(rom16: &[u16]) -> Chip8 {
        let mut chip8 = Chip8::init();

        chip8.load16(rom16);

        chip8
    }

    #[test]
    fn test_op_00e0() {
        let mut chip8 = load_chip8(&[0x00E0]);
        chip8.video[0] = 1;
        chip8.video[100] = 1;
        chip8.video[1000] = 1;

        chip8.tick();

        assert!(chip8.video.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_op_00ee() {
        let mut chip8 = load_chip8(&[0x6012, 0x2206, 0x00E0, 0x6110, 0x6111, 0x6112, 0x00EE]);
        chip8.video[0] = 1;
        chip8.video[100] = 1;
        chip8.video[1000] = 1;

        chip8.tick();
        assert_eq!(chip8.v[0], 0x12);

        chip8.tick();
        assert_eq!(chip8.stack[0], 0x204);
        assert_eq!(chip8.sp, 1);

        chip8.tick();
        assert_eq!(chip8.v[1], 0x10);

        chip8.tick();
        assert_eq!(chip8.v[1], 0x11);

        chip8.tick();
        assert_eq!(chip8.v[1], 0x12);

        chip8.tick();
        assert_eq!(chip8.pc, 0x204);
        assert_eq!(chip8.sp, 0);

        chip8.tick();
        assert!(chip8.video.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_op_fx33_1() {
        let mut chip8 = load_chip8(&[0xF033]);
        chip8.v[0] = 123;
        chip8.i = 0x300;

        chip8.tick();

        assert_eq!(chip8.mem[0x300], 1);
        assert_eq!(chip8.mem[0x301], 2);
        assert_eq!(chip8.mem[0x302], 3);
    }

    #[test]
    fn test_op_fx33_2() {
        let mut chip8 = load_chip8(&[0xF033]);
        chip8.v[0] = 000;
        chip8.i = 0x300;

        chip8.tick();

        assert_eq!(chip8.mem[0x300], 0);
        assert_eq!(chip8.mem[0x301], 0);
        assert_eq!(chip8.mem[0x302], 0);
    }

    #[test]
    fn test_op_fx33_3() {
        let mut chip8 = load_chip8(&[0xF033]);

        chip8.v[0] = 255;
        chip8.i = 0x300;

        chip8.tick();

        assert_eq!(chip8.mem[0x300], 2);
        assert_eq!(chip8.mem[0x301], 5);
        assert_eq!(chip8.mem[0x302], 5);
    }

    #[test]
    fn test_op_fx29_1() {
        let mut chip8 = load_chip8(&[0x6000, 0xF029]);

        chip8.tick();
        chip8.tick();
        assert_eq!(chip8.i, 0x0000);
    }

    #[test]
    fn test_op_fx29_2() {
        let mut chip8 = load_chip8(&[0x6001, 0xF029]);

        chip8.tick();
        chip8.tick();
        assert_eq!(chip8.i, 0x0000 + 5);
    }

    #[test]
    fn test_op_fx55() {
        let rom16 = [
            0xA3E8, 0x6000, 0x6101, 0x6202, 0x6303, 0x6404, 0x6505, 0x6606, 0x6707, 0x6808, 0x6909,
            0x6A0A, 0x6B0B, 0x6C0C, 0x6D0D, 0x6E0E, 0x6F0F, 0xFF55, 0xA3E8,
        ];
        let mut chip8 = load_chip8(&rom16);

        for _ in 0..rom16.len() {
            chip8.tick();
        }

        assert_eq!(
            chip8.v,
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF]
        );

        let i = chip8.i as usize;
        let mem = &chip8.mem[i..(i + 16)];

        assert_eq!(chip8.v, mem);
    }

    #[test]
    fn test_op_fx65() {
        let rom16 = [
            0xA3E8, 0x6000, 0x6101, 0x6202, 0x6303, 0x6404, 0x6505, 0x6606, 0x6707, 0x6808, 0x6909,
            0x6A0A, 0x6B0B, 0x6C0C, 0x6D0D, 0x6E0E, 0x6F0F, 0xFF55, 0x6000, 0x6100, 0x6200, 0x6300,
            0x6400, 0x6500, 0x6600, 0x6700, 0x6800, 0x6900, 0x6A00, 0x6B00, 0x6C00, 0x6D00, 0x6E00,
            0x6F00, 0xA3E8, 0xFF65,
        ];
        let mut chip8 = load_chip8(&rom16);

        for _ in 0..rom16.len() {
            chip8.tick();
        }

        assert_eq!(
            chip8.v,
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF]
        );
    }
}
