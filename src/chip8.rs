use crate::{dec_error, dec_mem_addr, dec_reg_x, dec_reg_y, dec_val_byte, dec_val_nibble, i};

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
    regs_v: [u8; NUM_REGS],
    reg_i: u16,
    reg_pc: u16,
    reg_sp: u8,
    stack: [u16; STACK_SIZE],
    mem: [u8; MEM_SIZE],
    video: [u8; VIDEO_SIZE],
    keys: [bool; KEYS_SIZE],
    wait_for_key: Option<u8>,
    delay_t: u8,
    buzzer_t: u8,
}

impl Chip8 {
    pub fn init() -> Self {
        let mut mem = [0u8; MEM_SIZE];

        let font_area = &mut mem[..FONT_SET.len()];
        font_area.copy_from_slice(&FONT_SET);

        Self {
            regs_v: [0u8; NUM_REGS],
            reg_i: 0,
            reg_pc: 0,
            reg_sp: 0,
            stack: [0u16; STACK_SIZE],
            mem,
            video: [0u8; VIDEO_SIZE],
            keys: [false; KEYS_SIZE],
            wait_for_key: None,
            delay_t: 0,
            buzzer_t: 0,
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

        self.reg_pc = PROG_START_ADDR as u16;
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
            "Executing opcode [0x{opcode:0>4X}] [I=0x{:0>4X}, PC=0x{:0>4X}, SP=0x{:0>4X}]",
            self.reg_i, self.reg_pc, self.reg_sp
        );

        self.execute(opcode);
    }

    fn fetch(&self) -> u16 {
        (self.mem[i!(self.reg_pc)] as u16) << 8 | self.mem[i!(self.reg_pc + 1)] as u16
    }

    fn execute(&mut self, opcode: u16) {
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => self.op_00e0(),
                0x00EE => self.op_00ee(),
                _ => self.op_0nnn(),
            },
            0x1000 => self.op_1nnn(opcode),
            0x2000 => self.op_2nnn(opcode),
            0x3000 => self.op_3xnn(opcode),
            0x4000 => self.op_4xnn(opcode),
            0x5000 => self.op_5xy0(opcode),
            0x6000 => self.op_6xnn(opcode),
            0x7000 => self.op_7xnn(opcode),
            0x8000 => match opcode & 0x000F {
                0x0000 => self.op_8xy0(opcode),
                0x0001 => self.op_8xy1(opcode),
                0x0002 => self.op_8xy2(opcode),
                0x0003 => self.op_8xy3(opcode),
                0x0004 => self.op_8xy4(opcode),
                0x0005 => self.op_8xy5(opcode),
                0x0006 => self.op_8xy6(opcode),
                0x0007 => self.op_8xy7(opcode),
                0x000E => self.op_8xye(opcode),
                _ => dec_error!(opcode),
            },
            0x9000 => self.op_9xy0(opcode),
            0xA000 => self.op_annn(opcode),
            0xB000 => self.op_bnnn(opcode),
            0xC000 => self.op_cxnn(opcode),
            0xD000 => self.op_dxyn(opcode),
            0xE000 => match opcode & 0x00FF {
                0x009E => self.op_ex9e(opcode),
                0x00A1 => self.op_exa1(opcode),
                _ => dec_error!(opcode),
            },
            0xF000 => match opcode & 0x00FF {
                0x0007 => self.op_fx07(opcode),
                0x000A => self.op_fx0a(opcode),
                0x0015 => self.op_fx15(opcode),
                0x0018 => self.op_fx18(opcode),
                0x001E => self.op_fx1e(opcode),
                0x0029 => self.op_fx29(opcode),
                0x0033 => self.op_fx33(opcode),
                0x0055 => self.op_fx55(opcode),
                0x0065 => self.op_fx65(opcode),
                _ => dec_error!(opcode),
            },
            _ => dec_error!(opcode),
        }
    }

    // 00E0
    // Clear the screen.
    fn op_00e0(&mut self) {
        self.video.fill(0);

        self.reg_pc += 2;
    }

    // 00EE
    // Return from a subroutine.
    fn op_00ee(&mut self) {
        self.reg_sp -= 1;
        self.reg_pc = self.stack[i!(self.reg_sp)];

        self.reg_pc += 2;
    }

    // 0NNN
    // Jump to a machine code routine at NNN.
    // Ignored by modern interpreters.
    fn op_0nnn(&self) {}

    // 1NNN
    // Jump to address NNN.
    fn op_1nnn(&mut self, opcode: u16) {
        self.reg_pc = dec_mem_addr!(opcode);
    }

    // 2NNN
    // Execute subroutine starting at address NNN.
    fn op_2nnn(&mut self, opcode: u16) {
        self.stack[i!(self.reg_sp)] = self.reg_pc;
        self.reg_sp += 1;

        self.reg_pc = dec_mem_addr!(opcode);
    }

    // 3XNN
    // Skip the following instruction if the value of register VX equals NN.
    fn op_3xnn(&mut self, opcode: u16) {
        self.reg_pc += if self.regs_v[dec_reg_x!(opcode)] == dec_val_byte!(opcode) {
            4
        } else {
            2
        };
    }

    // 4XNN
    // Skip the following instruction if the value of register VX is not equal to NN.
    fn op_4xnn(&mut self, opcode: u16) {
        self.reg_pc += if self.regs_v[dec_reg_x!(opcode)] != dec_val_byte!(opcode) {
            4
        } else {
            2
        };
    }

    // 5XY0
    // Skip the following instruction if the value of register VX is equal to the value of register VY.
    fn op_5xy0(&mut self, opcode: u16) {
        self.reg_pc += if self.regs_v[dec_reg_x!(opcode)] == self.regs_v[dec_reg_y!(opcode)] {
            4
        } else {
            2
        };
    }

    // 6XNN
    // Store number NN in register VX.
    fn op_6xnn(&mut self, opcode: u16) {
        self.regs_v[dec_reg_x!(opcode)] = dec_val_byte!(opcode);

        self.reg_pc += 2;
    }

    // 7XNN
    // Add the value NN to register VX.
    fn op_7xnn(&mut self, opcode: u16) {
        let x = dec_reg_x!(opcode);

        self.regs_v[x] = self.regs_v[x].wrapping_add(dec_val_byte!(opcode));

        self.reg_pc += 2;
    }

    // 8XY0
    // Store the value of register VY in register VX.
    fn op_8xy0(&mut self, opcode: u16) {
        self.regs_v[dec_reg_x!(opcode)] = self.regs_v[dec_reg_y!(opcode)];

        self.reg_pc += 2;
    }

    // 8XY1
    // Set VX to VX OR VY.
    fn op_8xy1(&mut self, opcode: u16) {
        self.regs_v[dec_reg_x!(opcode)] |= self.regs_v[dec_reg_y!(opcode)];

        self.reg_pc += 2;
    }

    // 8XY2
    // Set VX to VX AND VY.
    fn op_8xy2(&mut self, opcode: u16) {
        self.regs_v[dec_reg_x!(opcode)] &= self.regs_v[dec_reg_y!(opcode)];

        self.reg_pc += 2;
    }

    // 8XY3
    // Set VX to VX XOR VY.
    fn op_8xy3(&mut self, opcode: u16) {
        self.regs_v[dec_reg_x!(opcode)] ^= self.regs_v[dec_reg_y!(opcode)];

        self.reg_pc += 2;
    }

    // 8XY4
    // Add the value of register VY to register VX.
    // Set VF to 01 if a carry occurs.
    // Set VF to 00 if a carry does not occur.
    fn op_8xy4(&mut self, opcode: u16) {
        let x = dec_reg_x!(opcode);
        let y = dec_reg_y!(opcode);

        let (sum, carry) = self.regs_v[x].overflowing_add(self.regs_v[y]);

        self.regs_v[x] = sum;
        self.regs_v[0xF] = carry.into();

        self.reg_pc += 2;
    }

    // 8XY5
    // Subtract the value of register VY from register VX.
    // Set VF to 00 if a borrow occurs.
    // Set VF to 01 if a borrow does not occur
    fn op_8xy5(&mut self, opcode: u16) {
        let x = dec_reg_x!(opcode);
        let y = dec_reg_y!(opcode);

        let (sub, borrow) = self.regs_v[x].overflowing_sub(self.regs_v[y]);

        self.regs_v[x] = sub;
        self.regs_v[0xF] = (!borrow).into();

        self.reg_pc += 2;
    }

    // 8XY6
    // Store the value of register VY shifted right one bit in register VX.
    // Set register VF to the least significant bit prior to the shift.
    // VY is unchanged
    fn op_8xy6(&mut self, opcode: u16) {
        let x = dec_reg_x!(opcode);
        let y = dec_reg_y!(opcode);

        self.regs_v[0xF] = self.regs_v[y] & 0x0001;
        self.regs_v[x] = self.regs_v[y] >> 1;

        self.reg_pc += 2;
    }

    // 8XY7
    // Set register VX to the value of VY minus VX.
    // Set VF to 00 if a borrow occurs.
    // Set VF to 01 if a borrow does not occur.
    fn op_8xy7(&mut self, opcode: u16) {
        let x = dec_reg_x!(opcode);
        let y = dec_reg_y!(opcode);

        let (sub, borrow) = self.regs_v[y].overflowing_sub(self.regs_v[x]);

        self.regs_v[x] = sub;
        self.regs_v[0xF] = (!borrow).into();

        self.reg_pc += 2;
    }

    // 8XYE
    // Store the value of register VY shifted left one bit in register VX.
    // Set register VF to the most significant bit prior to the shift.
    // VY is unchanged.
    fn op_8xye(&mut self, opcode: u16) {
        let x = dec_reg_x!(opcode);
        let y = dec_reg_y!(opcode);

        self.regs_v[0xF] = (self.regs_v[y] >> 7) & 1;
        self.regs_v[x] = self.regs_v[y] << 1;

        self.reg_pc += 2;
    }

    // 9XY0
    // Skip the following instruction if the value of register VX is not equal to the value of register VY.
    fn op_9xy0(&mut self, opcode: u16) {
        self.reg_pc += if self.regs_v[dec_reg_x!(opcode)] != self.regs_v[dec_reg_y!(opcode)] {
            4
        } else {
            2
        };
    }

    // ANNN
    // Store memory address NNN in register I.
    fn op_annn(&mut self, opcode: u16) {
        self.reg_i = dec_mem_addr!(opcode);

        self.reg_pc += 2;
    }

    // BNNN
    // Jump to address NNN + V0.
    fn op_bnnn(&mut self, opcode: u16) {
        self.reg_pc = dec_mem_addr!(opcode) + self.regs_v[0x0] as u16;
    }

    // CXNN
    // Set VX to a random number with a mask of NN.
    fn op_cxnn(&mut self, opcode: u16) {
        self.regs_v[dec_reg_x!(opcode)] = fastrand::u8(0..=255) & dec_val_byte!(opcode);

        self.reg_pc += 2;
    }

    // DXYN
    // Draw a sprite at position VX, VY with N bytes of sprite data starting at the address stored in I.
    // Set VF to 01 if any set pixels are changed to unset, and 00 otherwise.
    fn op_dxyn(&mut self, opcode: u16) {
        let x = self.regs_v[dec_reg_x!(opcode)] as usize % VIDEO_COLS;
        let y = self.regs_v[dec_reg_y!(opcode)] as usize % VIDEO_ROWS;

        let addr_start = self.reg_i as usize;
        let n = dec_val_nibble!(opcode) as usize;

        // Clip rows.
        let addr_end = if (y + n) < VIDEO_ROWS {
            addr_start + n
        } else {
            addr_start + (VIDEO_ROWS - y)
        };

        // Clip cols.
        let bit_end = if (x + 8) < VIDEO_COLS {
            8
        } else {
            VIDEO_COLS - x
        };

        self.regs_v[0xF] = 0;

        for (y_ofst, addr) in (addr_start..addr_end).enumerate() {
            for i in 0..bit_end {
                if (self.mem[addr] & (0x80 >> i)) != 0 {
                    let pixel_pos = ((y + y_ofst) * VIDEO_COLS) + x + i;

                    if self.video[pixel_pos] == 1 {
                        self.regs_v[0xF] = 1;
                    }

                    self.video[pixel_pos] ^= 1;
                }
            }
        }

        self.reg_pc += 2;
    }

    // EX9E
    // Skip the following instruction if the key corresponding to the hex value currently stored in register VX is pressed.
    fn op_ex9e(&mut self, opcode: u16) {
        let reg_vx = self.regs_v[dec_reg_x!(opcode)] as usize;

        self.reg_pc = if self.keys[reg_vx] == true { 4 } else { 2 };
    }

    // EXA1
    // Skip the following instruction if the key corresponding to the hex value currently stored in register VX is not pressed.
    fn op_exa1(&mut self, opcode: u16) {
        let reg_vx = self.regs_v[dec_reg_x!(opcode)] as usize;

        self.reg_pc = if self.keys[reg_vx] == false { 4 } else { 2 };
    }

    // FX07
    // Store the current value of the delay timer in register VX.
    fn op_fx07(&mut self, opcode: u16) {
        self.regs_v[dec_reg_x!(opcode)] = self.delay_t;

        self.reg_pc += 2;
    }

    // FX0A
    // Wait for a keypress and store the result in register VX.
    fn op_fx0a(&mut self, opcode: u16) {
        println!("Waiting for key...");

        if let Some(key) = self.wait_for_key {
            if !self.keys[i!(key)] {
                println!("Got key {:?}", self.wait_for_key);
                self.regs_v[dec_reg_x!(opcode)] = key;
                self.wait_for_key = None;
                self.reg_pc += 2;
            }
        } else {
            if let Some(key) = self.keys.iter().position(|&k| k) {
                self.keys[key] = true;
                self.wait_for_key = Some(key as u8);
            }
        }
    }

    // FX15
    // Set the delay timer to the value of register VX.
    fn op_fx15(&mut self, opcode: u16) {
        self.delay_t = self.regs_v[dec_reg_x!(opcode)];

        self.reg_pc += 2;
    }

    // FX18
    // Set the sound timer to the value of register VX.
    fn op_fx18(&mut self, opcode: u16) {
        self.buzzer_t = self.regs_v[dec_reg_x!(opcode)];

        self.reg_pc += 2;
    }

    // FX1E
    // Add the value stored in register VX to register I.
    fn op_fx1e(&mut self, opcode: u16) {
        let reg_vx = self.regs_v[dec_reg_x!(opcode)];

        self.reg_i = self.reg_i.wrapping_add(reg_vx as u16);

        // Spacefight 2091!
        if self.reg_i > 0xFFF {
            self.regs_v[0xF] = 1;
        }

        self.reg_pc += 2;
    }

    // FX29
    // Set I to the memory address of the sprite data corresponding to the hexadecimal digit stored in register VX.
    fn op_fx29(&mut self, opcode: u16) {
        let regv_x = self.regs_v[dec_reg_x!(opcode)];

        self.reg_i = (regv_x as u16 * 5) as u16;

        self.reg_pc += 2;
    }

    // FX33
    // Store the binary-coded decimal equivalent of the value stored in register VX at addresses I, I + 1, and I + 2.
    fn op_fx33(&mut self, opcode: u16) {
        let n = self.regs_v[i!(dec_reg_x!(opcode))];

        self.mem[i!(self.reg_i)] = n / 100;
        self.mem[i!(self.reg_i + 1)] = n % 100 / 10;
        self.mem[i!(self.reg_i + 2)] = n % 10;

        self.reg_pc += 2;
    }

    // FX55
    // Store the values of registers V0 to VX inclusive in memory starting at address I.
    // I is set to I + X + 1 after operation.
    fn op_fx55(&mut self, opcode: u16) {
        let x = dec_reg_x!(opcode);

        for i in 0..=x {
            self.mem[i!(self.reg_i) + i] = self.regs_v[i];
        }

        self.reg_i += (x + 1) as u16;

        self.reg_pc += 2;
    }

    // FX65
    // Fill registers V0 to VX inclusive with the values stored in memory starting at address I.
    // I is set to I + X + 1 after operation.
    fn op_fx65(&mut self, opcode: u16) {
        let x = dec_reg_x!(opcode);

        for i in 0..=x {
            self.regs_v[i] = self.mem[i!(self.reg_i) + i];
        }

        self.reg_i += (x + 1) as u16;

        self.reg_pc += 2;
    }
}

#[cfg(test)]
mod tests {
    use super::i;
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
        assert_eq!(chip8.regs_v[0], 0x12);

        chip8.tick();
        assert_eq!(chip8.stack[0], 0x202);
        assert_eq!(chip8.reg_sp, 1);

        chip8.tick();
        assert_eq!(chip8.regs_v[1], 0x10);

        chip8.tick();
        assert_eq!(chip8.regs_v[1], 0x11);

        chip8.tick();
        assert_eq!(chip8.regs_v[1], 0x12);

        chip8.tick();
        assert_eq!(chip8.reg_pc, 0x204);
        assert_eq!(chip8.reg_sp, 0);

        chip8.tick();
        assert!(chip8.video.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_op_fx33_1() {
        let mut chip8 = load_chip8(&[0xF033]);
        chip8.regs_v[0] = 123;
        chip8.reg_i = 0x300;

        chip8.tick();

        assert_eq!(chip8.mem[0x300], 1);
        assert_eq!(chip8.mem[0x301], 2);
        assert_eq!(chip8.mem[0x302], 3);
    }

    #[test]
    fn test_op_fx33_2() {
        let mut chip8 = load_chip8(&[0xF033]);
        chip8.regs_v[0] = 000;
        chip8.reg_i = 0x300;

        chip8.tick();

        assert_eq!(chip8.mem[0x300], 0);
        assert_eq!(chip8.mem[0x301], 0);
        assert_eq!(chip8.mem[0x302], 0);
    }

    #[test]
    fn test_op_fx33_3() {
        let mut chip8 = load_chip8(&[0xF033]);

        chip8.regs_v[0] = 255;
        chip8.reg_i = 0x300;

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
        assert_eq!(chip8.reg_i, 0x0000);
    }

    #[test]
    fn test_op_fx29_2() {
        let mut chip8 = load_chip8(&[0x6001, 0xF029]);

        chip8.tick();
        chip8.tick();
        assert_eq!(chip8.reg_i, 0x0000 + 5);
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
            chip8.regs_v,
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF]
        );

        let mem = &chip8.mem[i!(chip8.reg_i)..i!(chip8.reg_i + 16)];

        assert_eq!(chip8.regs_v, mem);
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
            chip8.regs_v,
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF]
        );
    }
}
