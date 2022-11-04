use crate::{dec_error, dec_reg_x, dec_reg_y, dec_val_byte, dec_mem_addr};

const NUM_REGS: usize = 16;

const STACK_SIZE: usize = 16;

const MEM_SIZE: usize = 4096;
const MEM_PROG_AREA_START: usize = 0x200;
const MEM_PROG_AREA_END: usize = 0xE8F;

const GFX_COLS: usize = 64;
const GFX_ROWS: usize = 32;
const GFX_SIZE: usize = GFX_COLS * GFX_ROWS;

const FONTS_SIZE: usize = 16 * 5;
const FONTS: [u8; FONTS_SIZE] = [
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
    memory: [u8; MEM_SIZE],
    gfx: [u8; GFX_SIZE],
    delay_t: u8,
    buzzer_t: u8,
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            regs_v: [0u8; NUM_REGS],
            reg_i: 0,
            reg_pc: 0,
            reg_sp: 0,
            stack: [0u16; STACK_SIZE],
            memory: [0u8; MEM_SIZE],
            gfx: [0u8; GFX_SIZE],
            delay_t: 0,
            buzzer_t: 0,
        }
    }

    pub fn load(&mut self, program: &[u8]) {
        println!("Loading {} bytes...", program.len());

        let program_area = &mut self.memory[MEM_PROG_AREA_START..=MEM_PROG_AREA_END];

        if program.len() > program_area.len() {
            panic!("The program is too big to fit in memory.");
        }

        program_area.copy_from_slice(program);

        println!("{} bytes loaded.", program.len());
    }

    pub fn run(&self) {}

    fn draw(&mut self, x: u8, y: u8, data: &[u8]) {}

    fn fetch(&self) -> u16 {
        0
    }

    fn execute(&mut self, opcode: u16) {
        match opcode & 0xF000 {
            0x0000 => match opcode {
                // 00E0
                // Clear the screen.
                0x00E0 => {
                    self.gfx.fill(0);
                }

                // 00EE
                // Return from a subroutine.
                0x00EE => {
                    self.reg_pc = self.stack[self.reg_sp as usize];
                    self.reg_sp -= 1;
                }

                // 0NNN
                // Jump to a machine code routine at nnn.
                // Ignored by modern interpreters.
                _ => {}
            },

            // 1NNN
            // Jump to address NNN.
            0x1000 => {
                self.reg_pc = dec_mem_addr!(opcode);
            }

            // 2NNN
            // Execute subroutine starting at address NNN.
            0x2000 => {
                self.stack[self.reg_sp as usize] = self.reg_pc;
                self.reg_sp += 1;
                self.reg_pc = dec_mem_addr!(opcode);
            }

            // 3XNN
            // Skip the following instruction if the value of register VX equals NN.
            0x3000 => {
                if self.regs_v[dec_reg_x!(opcode)] == dec_val_byte!(opcode) {
                    self.reg_pc += 2;
                }
            }

            // 4XNN
            // Skip the following instruction if the value of register VX is not equal to NN.
            0x4000 => {
                if self.regs_v[dec_reg_x!(opcode)] != dec_val_byte!(opcode) {
                    self.reg_pc += 2;
                }
            }

            // 5XY0
            // Skip the following instruction if the value of register VX is equal to the value of register VY.
            0x5000 => {
                if self.regs_v[dec_reg_x!(opcode)] == self.regs_v[dec_reg_y!(opcode)] {
                    self.reg_pc += 2;
                }
            }

            // 6XNN
            // Store number NN in register VX.
            0x6000 => {
                self.regs_v[dec_reg_x!(opcode)] = dec_val_byte!(opcode);
            }

            // 7XNN
            // Add the value NN to register VX.
            0x7000 => {
                let x = dec_reg_x!(opcode);

                self.regs_v[x] = self.regs_v[x].wrapping_add(dec_val_byte!(opcode));
            }

            0x8000 => match opcode & 0x000F {
                // 8XY0
                // Store the value of register VY in register VX.
                0x0000 => {
                    self.regs_v[dec_reg_x!(opcode)] = self.regs_v[dec_reg_y!(opcode)];
                }

                // 8XY1
                // Set VX to VX OR VY.
                0x0001 => {
                    self.regs_v[dec_reg_x!(opcode)] |= self.regs_v[dec_reg_y!(opcode)];
                }

                // 8XY2
                // Set VX to VX AND VY.
                0x0002 => {
                    self.regs_v[dec_reg_x!(opcode)] &= self.regs_v[dec_reg_y!(opcode)];
                }

                // 8XY3
                // Set VX to VX XOR VY.
                0x0003 => {
                    self.regs_v[dec_reg_x!(opcode)] ^= self.regs_v[dec_reg_y!(opcode)];
                }

                // 8XY4
                // Add the value of register VY to register VX.
                // Set VF to 01 if a carry occurs.
                // Set VF to 00 if a carry does not occur.
                0x0004 => {
                    let x = dec_reg_x!(opcode);
                    let y = dec_reg_y!(opcode);

                    let (sum, carry) = self.regs_v[x].overflowing_add(self.regs_v[y]);

                    self.regs_v[x] = sum;
                    self.regs_v[0xF] = if carry { 1 } else { 0 };
                }

                // 8XY5
                // Subtract the value of register VY from register VX.
                // Set VF to 00 if a borrow occurs.
                // Set VF to 01 if a borrow does not occur
                0x0005 => {
                    let x = dec_reg_x!(opcode);
                    let y = dec_reg_y!(opcode);

                    let (sub, borrow) = self.regs_v[x].overflowing_sub(self.regs_v[y]);

                    self.regs_v[x] = sub;
                    self.regs_v[0xF] = if borrow { 0 } else { 1 };
                }

                // 8XY6
                // Store the value of register VY shifted right one bit in register VX.
                // Set register VF to the least significant bit prior to the shift.
                // VY is unchanged
                0x0006 => {
                    let x = dec_reg_x!(opcode);
                    let y = dec_reg_y!(opcode);

                    self.regs_v[0xF] = self.regs_v[y] & 0x01;
                    self.regs_v[x] = self.regs_v[y] >> 1;
                }

                // 8XY7
                // Set register VX to the value of VY minus VX.
                // Set VF to 00 if a borrow occurs.
                // Set VF to 01 if a borrow does not occur.
                0x0007 => {
                    let x = dec_reg_x!(opcode);
                    let y = dec_reg_y!(opcode);

                    let (sub, borrow) = self.regs_v[y].overflowing_sub(self.regs_v[y]);

                    self.regs_v[x] = sub;
                    self.regs_v[0xF] = (!borrow).into();
                }

                // 8XYE
                // Store the value of register VY shifted left one bit in register VX.
                // Set register VF to the most significant bit prior to the shift.
                // VY is unchanged.
                0x000E => {
                    let x = dec_reg_x!(opcode);
                    let y = dec_reg_y!(opcode);

                    self.regs_v[0xF] = (self.regs_v[y] >> 7) & 0x01;
                    self.regs_v[x] = self.regs_v[y] << 1;
                }

                _ => dec_error!(opcode),
            },

            // 9XY0
            // Skip the following instruction if the value of register VX is not equal to the value of register VY.
            0x9000 => {
                if self.regs_v[dec_reg_x!(opcode)] != self.regs_v[dec_reg_y!(opcode)] {
                    self.reg_pc += 2;
                }
            }

            // ANNN
            // Store memory address NNN in register I.
            0xA000 => {
                self.reg_i = dec_mem_addr!(opcode);
            }

            // BNNN
            // Jump to address NNN + V0.
            0xB000 => {
                self.reg_pc = dec_mem_addr!(opcode) + self.regs_v[0x0] as u16;
            }

            // CXNN
            // Set VX to a random number with a mask of NN.
            0xC000 => {
                self.regs_v[dec_reg_x!(opcode)] = fastrand::u8(0..=255) & dec_val_byte!(opcode);
            }

            // DXYN
            // Draw a sprite at position VX, VY with N bytes of sprite data starting at the address stored in I.
            // Set VF to 01 if any set pixels are changed to unset, and 00 otherwise.
            0xD000 => {}

            0xE000 => match opcode & 0x00FF {
                // EX9E
                // Skip the following instruction if the key corresponding to the hex value currently stored in register VX is pressed.
                0x009E => {}

                // EXA1
                // Skip the following instruction if the key corresponding to the hex value currently stored in register VX is not pressed.
                0x00A1 => {}

                _ => dec_error!(opcode),
            },

            0xF000 => match opcode & 0x00FF {
                // FX07
                // Store the current value of the delay timer in register VX.
                0x0007 => {
                    self.regs_v[dec_reg_x!(opcode)] = self.delay_t;
                }

                // FX0A
                // Wait for a keypress and store the result in register VX.
                0x000A => {}

                // FX15
                // Set the delay timer to the value of register VX.
                0x0015 => {
                    self.delay_t = self.regs_v[dec_reg_x!(opcode)];
                }

                // FX18
                // Set the sound timer to the value of register VX.
                0x0018 => {
                    self.buzzer_t = self.regs_v[dec_reg_x!(opcode)];
                }

                // FX1E
                // Add the value stored in register VX to register I.
                0x001E => {
                    let reg_vx = self.regs_v[dec_reg_x!(opcode)] as u16;

                    self.reg_i = self.reg_i.wrapping_add(reg_vx);
                }

                // FX29
                // Set I to the memory address of the sprite data corresponding to the hexadecimal digit stored in register VX.
                0x0029 => {}

                // FX33
                // Store the binary-coded decimal equivalent of the value stored in register VX at addresses I, I + 1, and I + 2.
                0x0033 => {}

                // FX55
                // Store the values of registers V0 to VX inclusive in memory starting at address I.
                // I is set to I + X + 1 after operation.
                0x0055 => {
                    let x = dec_reg_x!(opcode);

                    for i in 0..=x {
                        self.memory[self.reg_i as usize + i] = self.regs_v[i];
                    }

                    // TODO: configurable?
                    self.reg_i += x as u16 + 1;
                }

                // FX65
                // Fill registers V0 to VX inclusive with the values stored in memory starting at address I.
                0x0065 => {
                    let x = dec_reg_x!(opcode);

                    for i in 0..=x {
                        self.regs_v[x] = self.memory[self.reg_i as usize + i];
                    }

                    // TODO: configurable?
                    self.reg_i += x as u16 + 1;
                }

                _ => dec_error!(opcode),
            },

            _ => dec_error!(opcode),
        }
    }
}

#[cfg(test)]
mod tests {}
