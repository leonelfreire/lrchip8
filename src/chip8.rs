// Reference: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM

use oorandom::Rand32;

use crate::{dec_addr, dec_byte, dec_error, dec_nibble, dec_x, dec_y};

const NUM_REGS: usize = 16;

const STACK_SIZE: usize = 16;

const MEM_SIZE: usize = 4096;

const ROM_START_ADDR: usize = 0x200;

const VIDEO_COLS: usize = 64;
const VIDEO_ROWS: usize = 32;
const VIDEO_SIZE: usize = VIDEO_COLS * VIDEO_ROWS;

const KEYS_SIZE: usize = 16;

const FONT_BYTES_PER_CHAR: u16 = 5;
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
    audio_t: u8,
    wait_for_key: Option<u8>,
    vblank: bool,
    rng: Rand32,
}

impl Chip8 {
    pub fn init(rng_seed: u64) -> Self {
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
            audio_t: 0,
            wait_for_key: None,
            vblank: false,
            rng: Rand32::new(rng_seed),
        }
    }

    pub fn video_cols(&self) -> usize {
        VIDEO_COLS
    }

    pub fn video_rows(&self) -> usize {
        VIDEO_ROWS
    }

    pub fn video(&self) -> &[u8] {
        &self.video
    }

    pub fn audio(&self) -> bool {
        self.audio_t > 0
    }

    pub fn write_keys(&mut self, keys: &[bool]) {
        self.keys.copy_from_slice(&keys[..KEYS_SIZE]);
    }

    pub fn update_timers(&mut self) {
        self.delay_t = self.delay_t.saturating_sub(1);
        self.audio_t = self.audio_t.saturating_sub(1);
    }

    pub fn load(&mut self, rom: &[u8]) {
        println!("Loading rom ({} bytes)...", rom.len());

        if let Some(rom_area) = self
            .mem
            .get_mut(ROM_START_ADDR..(ROM_START_ADDR + rom.len()))
        {
            rom_area.copy_from_slice(rom);
            println!("{} bytes loaded.", rom.len());
        } else {
            panic!("The rom is too big to fit in memory.");
        }

        self.pc = ROM_START_ADDR as u16;
    }

    pub fn load16(&mut self, rom16: &[u16]) {
        let rom8 = rom16
            .iter()
            .flat_map(|&w| [(w >> 8) as u8, (w & 0x00FF) as u8].into_iter())
            .collect::<Vec<u8>>();

        self.load(&rom8);
    }

    pub fn set_vblank(&mut self, vblank: bool) {
        self.vblank = vblank;
    }

    pub fn tick(&mut self) {
        let opcode = self.fetch();

        #[cfg(debug_assertions)]
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
                _ => self.op_0nnn(opcode),
            },
            0x1000 => self.op_1nnn(dec_addr!(opcode)),
            0x2000 => self.op_2nnn(dec_addr!(opcode)),
            0x3000 => self.op_3xkk(dec_x!(opcode), dec_byte!(opcode)),
            0x4000 => self.op_4xkk(dec_x!(opcode), dec_byte!(opcode)),
            0x5000 => self.op_5xy0(dec_x!(opcode), dec_y!(opcode)),
            0x6000 => self.op_6xkk(dec_x!(opcode), dec_byte!(opcode)),
            0x7000 => self.op_7xkk(dec_x!(opcode), dec_byte!(opcode)),
            0x8000 => match opcode & 0x000F {
                0x0000 => self.op_8xy0(dec_x!(opcode), dec_y!(opcode)),
                0x0001 => self.op_8xy1(dec_x!(opcode), dec_y!(opcode)),
                0x0002 => self.op_8xy2(dec_x!(opcode), dec_y!(opcode)),
                0x0003 => self.op_8xy3(dec_x!(opcode), dec_y!(opcode)),
                0x0004 => self.op_8xy4(dec_x!(opcode), dec_y!(opcode)),
                0x0005 => self.op_8xy5(dec_x!(opcode), dec_y!(opcode)),
                0x0006 => self.op_8xy6(dec_x!(opcode)),
                0x0007 => self.op_8xy7(dec_x!(opcode), dec_y!(opcode)),
                0x000E => self.op_8xye(dec_x!(opcode)),
                _ => dec_error!(opcode),
            },
            0x9000 => self.op_9xy0(dec_x!(opcode), dec_y!(opcode)),
            0xA000 => self.op_annn(dec_addr!(opcode)),
            0xB000 => self.op_bnnn(dec_addr!(opcode)),
            0xC000 => self.op_cxkk(dec_x!(opcode), dec_byte!(opcode)),
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

    // 00E0 - CLS
    // Clear the display.
    fn op_00e0(&mut self) {
        self.video.fill(0);
    }

    // 00EE - RET
    // Return from a subroutine.
    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    // 0nnn - SYS addr
    // Jump to a machine code routine at nnn.
    // This instruction is only used on the old computers on which Chip-8 was originally implemented.
    // It is ignored by modern interpreters.
    fn op_0nnn(&self, opcode: u16) {
        println!(
            "Machine code routine: [OP=0x{:0>4X}] [I=0x{:0>4X}, PC=0x{:0>4X}, SP=0x{:0>4X}, V={:?}]",
            opcode, self.i, self.pc, self.sp, self.v
        );
    }

    // 1nnn - JP addr
    // Jump to location nnn.
    fn op_1nnn(&mut self, addr: u16) {
        self.pc = addr;
    }

    // 2nnn - CALL addr
    // Call subroutine at nnn.
    fn op_2nnn(&mut self, addr: u16) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = addr;
    }

    // 3xkk - SE Vx, byte
    // Skip next instruction if Vx = kk.
    fn op_3xkk(&mut self, x: usize, kk: u8) {
        if self.v[x] == kk {
            self.pc += 2;
        }
    }

    // 4xkk - SNE Vx, byte
    // Skip next instruction if Vx != kk.
    fn op_4xkk(&mut self, x: usize, kk: u8) {
        if self.v[x] != kk {
            self.pc += 2;
        }
    }

    // 5xy0 - SE Vx, Vy
    // Skip next instruction if Vx = Vy.
    fn op_5xy0(&mut self, x: usize, y: usize) {
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }

    // 6xkk - LD Vx, byte
    // Set Vx = kk.
    fn op_6xkk(&mut self, x: usize, kk: u8) {
        self.v[x] = kk;
    }

    // 7xkk - ADD Vx, byte
    // Set Vx = Vx + kk.
    fn op_7xkk(&mut self, x: usize, kk: u8) {
        self.v[x] = self.v[x].wrapping_add(kk);
    }

    // 8xy0 - LD Vx, Vy
    // Set Vx = Vy.
    fn op_8xy0(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
    }

    // 8xy1 - OR Vx, Vy
    // Set Vx = Vx OR Vy.
    fn op_8xy1(&mut self, x: usize, y: usize) {
        self.v[x] |= self.v[y];
    }

    // 8xy2 - AND Vx, Vy
    // Set Vx = Vx AND Vy.
    fn op_8xy2(&mut self, x: usize, y: usize) {
        self.v[x] &= self.v[y];
    }

    // 8xy3 - XOR Vx, Vy
    // Set Vx = Vx XOR Vy.
    fn op_8xy3(&mut self, x: usize, y: usize) {
        self.v[x] ^= self.v[y];
    }

    // 8xy4 - ADD Vx, Vy
    // Set Vx = Vx + Vy, set VF = carry.
    fn op_8xy4(&mut self, x: usize, y: usize) {
        let (sum, carry) = self.v[x].overflowing_add(self.v[y]);

        self.v[x] = sum;
        self.v[0xF] = carry.into();
    }

    // 8xy5 - SUB Vx, Vy
    // Set Vx = Vx - Vy, set VF = NOT borrow.
    fn op_8xy5(&mut self, x: usize, y: usize) {
        let (sub, borrow) = self.v[x].overflowing_sub(self.v[y]);

        self.v[x] = sub;
        self.v[0xF] = (!borrow).into();
    }

    // 8xy6 - SHR Vx {, Vy}
    // Set Vx = Vx SHR 1.
    // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
    fn op_8xy6(&mut self, x: usize) {
        let vf = self.v[x] & 1;

        self.v[x] >>= 1;
        self.v[0xF] = vf;
    }

    // 8xy7 - SUBN Vx, Vy
    // Set Vx = Vy - Vx, set VF = NOT borrow.
    fn op_8xy7(&mut self, x: usize, y: usize) {
        let (sub, borrow) = self.v[y].overflowing_sub(self.v[x]);

        self.v[x] = sub;
        self.v[0xF] = (!borrow).into();
    }

    // 8xyE - SHL Vx {, Vy}
    // Set Vx = Vx SHL 1.
    // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
    fn op_8xye(&mut self, x: usize) {
        let vf = (self.v[x] >> 7) & 1;

        self.v[x] <<= 1;
        self.v[0xF] = vf;
    }

    // 9xy0 - SNE Vx, Vy
    // Skip next instruction if Vx != Vy.
    fn op_9xy0(&mut self, x: usize, y: usize) {
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
    }

    // Annn - LD I, addr
    // Set I = nnn.
    fn op_annn(&mut self, addr: u16) {
        self.i = addr;
    }

    // Bnnn - JP V0, addr
    // Jump to location nnn + V0.
    fn op_bnnn(&mut self, nnn: u16) {
        self.pc = nnn + self.v[0] as u16;
    }

    // Cxkk - RND Vx, byte
    // Set Vx = random byte AND kk.
    fn op_cxkk(&mut self, x: usize, kk: u8) {
        self.v[x] = self.rng.rand_range(0..256) as u8 & kk;
    }

    // Dxyn - DRW Vx, Vy, nibble
    // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    fn op_dxyn(&mut self, x: usize, y: usize, n: u8) {
        // Wait for vblank.
        if !self.vblank {
            self.pc -= 2;
            return;
        }

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

    // Ex9E - SKP Vx
    // Skip next instruction if key with the value of Vx is pressed.
    fn op_ex9e(&mut self, x: usize) {
        let vx = self.v[x] as usize;

        if self.keys[vx] {
            self.pc += 2;
        }
    }

    // ExA1 - SKNP Vx
    // Skip next instruction if key with the value of Vx is not pressed.
    fn op_exa1(&mut self, x: usize) {
        let vx = self.v[x] as usize;

        if !self.keys[vx] {
            self.pc += 2;
        }
    }

    // Fx07 - LD Vx, DT
    // Set Vx = delay timer value.
    fn op_fx07(&mut self, x: usize) {
        self.v[x] = self.delay_t;
    }

    // Fx0A - LD Vx, K
    // Wait for a key press, store the value of the key in Vx.
    fn op_fx0a(&mut self, x: usize) {
        #[cfg(debug_assertions)]
        println!("Waiting for key...");

        if let Some(key) = self.wait_for_key {
            if !self.keys[key as usize] {
                #[cfg(debug_assertions)]
                println!("Got key 0x{:X}.", key);
                self.v[x] = key;
                self.wait_for_key = None;
                return;
            }
        } else if let Some(key) = self.keys.iter().position(|&k| k) {
            self.wait_for_key = Some(key as u8);
        }

        // Try again.
        self.pc -= 2;
    }

    // Fx15 - LD DT, Vx
    // Set delay timer = Vx.
    fn op_fx15(&mut self, x: usize) {
        self.delay_t = self.v[x];
    }

    // Fx18 - LD ST, Vx
    // Set sound timer = Vx.
    fn op_fx18(&mut self, x: usize) {
        self.audio_t = self.v[x];
    }

    // Fx1E - ADD I, Vx
    // Set I = I + Vx.
    fn op_fx1e(&mut self, x: usize) {
        let vx = self.v[x] as u16;

        self.i = self.i.wrapping_add(vx);
        self.v[0xF] = (self.i > 0xFFF).into();
    }

    // Fx29 - LD F, Vx
    // Set I = location of sprite for digit Vx.
    fn op_fx29(&mut self, x: usize) {
        let vx = self.v[x] as u16;

        self.i = vx * FONT_BYTES_PER_CHAR;
    }

    // Fx33 - LD B, Vx
    // Store BCD representation of Vx in memory locations I, I+1, and I+2.
    fn op_fx33(&mut self, x: usize) {
        let n = self.v[x];
        let i = self.i as usize;

        self.mem[i] = n / 100;
        self.mem[i + 1] = n % 100 / 10;
        self.mem[i + 2] = n % 10;
    }

    // Fx55 - LD [I], Vx
    // Store registers V0 through Vx in memory starting at location I.
    fn op_fx55(&mut self, x: usize) {
        for i in 0..=x {
            self.mem[self.i as usize + i] = self.v[i];
        }
    }

    // Fx65 - LD Vx, [I]
    // Read registers V0 through Vx from memory starting at location I.
    fn op_fx65(&mut self, x: usize) {
        for i in 0..=x {
            self.v[i] = self.mem[self.i as usize + i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Chip8;

    fn load_chip8(rom16: &[u16]) -> Chip8 {
        let mut chip8 = Chip8::init(0);

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
