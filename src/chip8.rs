use crate::{dec_error, dec_mem_addr, dec_reg_x, dec_reg_y, dec_val_byte, dec_val_nibble, i};

const NUM_REGS: usize = 16;

const STACK_SIZE: usize = 16;

const MEM_SIZE: usize = 4096;

const PROG_START_ADDR: usize = 0x200;

const VIDEO_COLS: usize = 64;
const VIDEO_ROWS: usize = 32;
const VIDEO_SIZE: usize = VIDEO_COLS * VIDEO_ROWS;

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
    inc_pc: bool,
    stack: [u16; STACK_SIZE],
    mem: [u8; MEM_SIZE],
    video: [u8; VIDEO_SIZE],
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
            inc_pc: false,
            stack: [0u16; STACK_SIZE],
            mem,
            video: [0u8; VIDEO_SIZE],
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

    pub fn video_buffer(&self) -> &[u8] {
        &self.video
    }

    pub fn load(&mut self, program: &[u8]) {
        println!("Loading program ({} bytes)...", program.len());

        if let Some(program_area) = self
            .mem
            .get_mut(PROG_START_ADDR..(PROG_START_ADDR + program.len()))
        {
            program_area.copy_from_slice(program);
            println!("{} bytes loaded.", program.len());
        } else {
            panic!("The program is too big to fit in memory.");
        }

        self.reg_pc = PROG_START_ADDR as u16;
    }

    pub fn tick(&mut self) {
        let opcode = self.fetch();
        println!("Executing opcode [{opcode:X}]");

        self.inc_pc = true;

        self.execute(opcode);

        if self.inc_pc {
            self.reg_pc += 2;
        }
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
    }

    // 00EE
    // Return from a subroutine.
    fn op_00ee(&mut self) {
        self.reg_pc = self.stack[i!(self.reg_sp)];
        self.reg_sp -= 1;
        self.inc_pc = false;
    }

    // 0NNN
    // Jump to a machine code routine at NNN.
    // Ignored by modern interpreters.
    fn op_0nnn(&self) {}

    // 1NNN
    // Jump to address NNN.
    fn op_1nnn(&mut self, opcode: u16) {
        self.reg_pc = dec_mem_addr!(opcode);
        self.inc_pc = false;
    }

    // 2NNN
    // Execute subroutine starting at address NNN.
    fn op_2nnn(&mut self, opcode: u16) {
        self.reg_sp += 1;
        self.stack[i!(self.reg_sp)] = self.reg_pc;
        self.reg_pc = dec_mem_addr!(opcode);
        self.inc_pc = false;
    }

    // 3XNN
    // Skip the following instruction if the value of register VX equals NN.
    fn op_3xnn(&mut self, opcode: u16) {
        if self.regs_v[dec_reg_x!(opcode)] == dec_val_byte!(opcode) {
            self.reg_pc += 2;
            self.inc_pc = false;
        }
    }

    // 4XNN
    // Skip the following instruction if the value of register VX is not equal to NN.
    fn op_4xnn(&mut self, opcode: u16) {
        if self.regs_v[dec_reg_x!(opcode)] != dec_val_byte!(opcode) {
            self.reg_pc += 2;
            self.inc_pc = false;
        }
    }

    // 5XY0
    // Skip the following instruction if the value of register VX is equal to the value of register VY.
    fn op_5xy0(&mut self, opcode: u16) {
        if self.regs_v[dec_reg_x!(opcode)] == self.regs_v[dec_reg_y!(opcode)] {
            self.reg_pc += 2;
            self.inc_pc = false;
        }
    }

    // 6XNN
    // Store number NN in register VX.
    fn op_6xnn(&mut self, opcode: u16) {
        self.regs_v[dec_reg_x!(opcode)] = dec_val_byte!(opcode);
    }

    // 7XNN
    // Add the value NN to register VX.
    fn op_7xnn(&mut self, opcode: u16) {
        let x = dec_reg_x!(opcode);

        self.regs_v[x] = self.regs_v[x].wrapping_add(dec_val_byte!(opcode));
    }

    // 8XY0
    // Store the value of register VY in register VX.
    fn op_8xy0(&mut self, opcode: u16) {
        self.regs_v[dec_reg_x!(opcode)] = self.regs_v[dec_reg_y!(opcode)];
    }

    // 8XY1
    // Set VX to VX OR VY.
    fn op_8xy1(&mut self, opcode: u16) {
        self.regs_v[dec_reg_x!(opcode)] |= self.regs_v[dec_reg_y!(opcode)];
    }

    // 8XY2
    // Set VX to VX AND VY.
    fn op_8xy2(&mut self, opcode: u16) {
        self.regs_v[dec_reg_x!(opcode)] &= self.regs_v[dec_reg_y!(opcode)];
    }

    // 8XY3
    // Set VX to VX XOR VY.
    fn op_8xy3(&mut self, opcode: u16) {
        self.regs_v[dec_reg_x!(opcode)] ^= self.regs_v[dec_reg_y!(opcode)];
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
    }

    // 9XY0
    // Skip the following instruction if the value of register VX is not equal to the value of register VY.
    fn op_9xy0(&mut self, opcode: u16) {
        if self.regs_v[dec_reg_x!(opcode)] != self.regs_v[dec_reg_y!(opcode)] {
            self.reg_pc += 2;
            self.inc_pc = false;
        }
    }

    // ANNN
    // Store memory address NNN in register I.
    fn op_annn(&mut self, opcode: u16) {
        self.reg_i = dec_mem_addr!(opcode);
    }

    // BNNN
    // Jump to address NNN + V0.
    fn op_bnnn(&mut self, opcode: u16) {
        self.reg_pc = dec_mem_addr!(opcode) + self.regs_v[0x0] as u16;
        self.inc_pc = false;
    }

    // CXNN
    // Set VX to a random number with a mask of NN.
    fn op_cxnn(&mut self, opcode: u16) {
        self.regs_v[dec_reg_x!(opcode)] = fastrand::u8(0..=255) & dec_val_byte!(opcode);
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
    }

    // EX9E
    // Skip the following instruction if the key corresponding to the hex value currently stored in register VX is pressed.
    fn op_ex9e(&mut self, opcode: u16) {}

    // EXA1
    // Skip the following instruction if the key corresponding to the hex value currently stored in register VX is not pressed.
    fn op_exa1(&mut self, opcode: u16) {}

    // FX07
    // Store the current value of the delay timer in register VX.
    fn op_fx07(&mut self, opcode: u16) {
        self.regs_v[dec_reg_x!(opcode)] = self.delay_t;
    }

    // FX0A
    // Wait for a keypress and store the result in register VX.
    fn op_fx0a(&mut self, opcode: u16) {}

    // FX15
    // Set the delay timer to the value of register VX.
    fn op_fx15(&mut self, opcode: u16) {
        self.delay_t = self.regs_v[dec_reg_x!(opcode)];
    }

    // FX18
    // Set the sound timer to the value of register VX.
    fn op_fx18(&mut self, opcode: u16) {
        self.buzzer_t = self.regs_v[dec_reg_x!(opcode)];
    }

    // FX1E
    // Add the value stored in register VX to register I.
    fn op_fx1e(&mut self, opcode: u16) {
        let reg_vx = self.regs_v[dec_reg_x!(opcode)];

        self.reg_i = self.reg_i.wrapping_add(reg_vx as u16);
    }

    // FX29
    // Set I to the memory address of the sprite data corresponding to the hexadecimal digit stored in register VX.
    fn op_fx29(&mut self, opcode: u16) {}

    // FX33
    // Store the binary-coded decimal equivalent of the value stored in register VX at addresses I, I + 1, and I + 2.
    fn op_fx33(&mut self, opcode: u16) {}

    // FX55
    // Store the values of registers V0 to VX inclusive in memory starting at address I.
    // I is set to I + X + 1 after operation.
    fn op_fx55(&mut self, opcode: u16) {
        let x = dec_reg_x!(opcode);

        for i in 0..=x {
            self.mem[i!(self.reg_i) + i] = self.regs_v[i];
        }

        self.reg_i += (x + 1) as u16;
    }

    // FX65
    // Fill registers V0 to VX inclusive with the values stored in memory starting at address I.
    // I is set to I + X + 1 after operation.
    fn op_fx65(&mut self, opcode: u16) {
        let x = dec_reg_x!(opcode);

        for i in 0..=x {
            self.regs_v[x] = self.mem[i!(self.reg_i) + i];
        }

        self.reg_i += (x + 1) as u16;
    }
}
