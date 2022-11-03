use crate::{dec_errror, dec_mem_addr, dec_reg1, dec_reg2, dec_value};

const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const MEM_SIZE: usize = 4096;
const MEM_PROG_START: usize = 0x200;
const MEM_PROG_END: usize = 0xE8F;
const GFX_SIZE: usize = 64 * 32;

pub struct Chip8 {
    regs_v: [u8; NUM_REGS],
    reg_pc: u16,
    reg_sp: u16,
    stack: [u16; STACK_SIZE],
    memory: [u8; MEM_SIZE],
    gfx: [u8; GFX_SIZE],
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            regs_v: [0u8; NUM_REGS],
            reg_pc: 0,
            reg_sp: 0,
            stack: [0u16; STACK_SIZE],
            memory: [0u8; MEM_SIZE],
            gfx: [0u8; GFX_SIZE],
        }
    }

    pub fn load(&mut self, program: &[u8]) {
        println!("Loading {} bytes...", program.len());

        let program_area = &mut self.memory[MEM_PROG_START..=MEM_PROG_END];

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

    fn execute(&mut self, inst: u16) {
        match inst & 0xF000 {
            0x0000 => match inst {
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
                self.reg_pc = dec_mem_addr!(inst);
            }

            // 2NNN
            // Execute subroutine starting at address NNN.
            0x2000 => {
                self.stack[self.reg_sp as usize] = self.reg_pc;
                self.reg_sp += 1;
                self.reg_pc = dec_mem_addr!(inst);
            }

            // 3XNN
            // Skip the following instruction if the value of register VX equals NN.
            0x3000 => {
                if self.regs_v[dec_reg1!(inst) as usize] == dec_value!(inst) {
                    self.reg_pc += 2;
                }
            }

            // 4XNN
            // Skip the following instruction if the value of register VX is not equal to NN.
            0x4000 => {
                if self.regs_v[dec_reg1!(inst) as usize] != dec_value!(inst) {
                    self.reg_pc += 2;
                }
            }

            // 5XY0
            // Skip the following instruction if the value of register VX is equal to the value of register VY.
            0x5000 => {
                if self.regs_v[dec_reg1!(inst) as usize] == self.regs_v[dec_reg2!(inst) as usize] {
                    self.reg_pc += 2;
                }
            }

            // 6XNN
            // Store number NN in register VX.
            0x6000 => {
                self.regs_v[dec_reg1!(inst) as usize] = dec_value!(inst);
            }

            // 7XNN
            // Add the value NN to register VX.
            0x7000 => {
                let x = dec_reg1!(inst) as usize;
                self.regs_v[x] = self.regs_v[x].wrapping_add(dec_value!(inst));
            }

            0x8000 => match inst & 0x000F {
                // 8XY0
                // Store the value of register VY in register VX.
                0x0000 => {
                    self.regs_v[dec_reg1!(inst) as usize] = self.regs_v[dec_reg2!(inst) as usize];
                }

                // 8XY1
                // Set VX to VX OR VY.
                0x0001 => {
                    self.regs_v[dec_reg1!(inst) as usize] |= self.regs_v[dec_reg2!(inst) as usize];
                }

                // 8XY2
                // Set VX to VX AND VY.
                0x0002 => {
                    self.regs_v[dec_reg1!(inst) as usize] &= self.regs_v[dec_reg2!(inst) as usize];
                }

                // 8XY3
                // Set VX to VX XOR VY.
                0x0003 => {
                    self.regs_v[dec_reg1!(inst) as usize] ^= self.regs_v[dec_reg2!(inst) as usize];
                }

                // 8XY4
                // Add the value of register VY to register VX.
                // Set VF to 01 if a carry occurs.
                // Set VF to 00 if a carry does not occur.
                0x0004 => {
                    let x = dec_reg1!(inst) as usize;
                    let y = dec_reg2!(inst) as usize;

                    if let Some(result) = self.regs_v[x].checked_add(self.regs_v[y]) {
                        self.regs_v[x] = result;
                        self.regs_v[0xF] = 0;
                    } else {
                        self.regs_v[x] = self.regs_v[x].wrapping_add(self.regs_v[y]);
                        self.regs_v[0xF] = 1;
                    }
                }

                // 8XY5
                // Subtract the value of register VY from register VX.
                // Set VF to 00 if a borrow occurs.
                // Set VF to 01 if a borrow does not occur
                0x0005 => {}

                // 8XY6
                // Store the value of register VY shifted right one bit in register VX.
                // Set register VF to the least significant bit prior to the shift.
                // VY is unchanged
                0x0006 => {}

                // 8XY7
                // Set register VX to the value of VY minus VX.
                // Set VF to 00 if a borrow occurs.
                // Set VF to 01 if a borrow does not occur.
                0x0007 => {}

                // 8XYE
                // Store the value of register VY shifted left one bit in register VX.
                // Set register VF to the most significant bit prior to the shift.
                // VY is unchanged.
                0x000E => {}

                _ => dec_errror!(inst),
            },

            // 9XY0
            // Skip the following instruction if the value of register VX is not equal to the value of register VY.
            0x9000 => {}

            // ANNN
            // Store memory address NNN in register I.
            0xA000 => {}

            // BNNN
            // Jump to address NNN + V0.
            0xB000 => {}

            // CXNN
            // Set VX to a random number with a mask of NN.
            0xC000 => {}

            // DXYN
            // Draw a sprite at position VX, VY with N bytes of sprite data starting at the address stored in I.
            // Set VF to 01 if any set pixels are changed to unset, and 00 otherwise.
            0xD000 => {}

            0xE000 => match inst & 0x00FF {
                // EX9E
                // Skip the following instruction if the key corresponding to the hex value currently stored in register VX is pressed.
                0x009E => {}

                // EXA1
                // Skip the following instruction if the key corresponding to the hex value currently stored in register VX is not pressed.
                0x00A1 => {}

                _ => dec_errror!(inst),
            },

            0xF000 => match inst & 0x00FF {
                // FX07
                // Store the current value of the delay timer in register VX.
                0x0007 => {}

                // FX0A
                // Wait for a keypress and store the result in register VX.
                0x000A => {}

                // FX15
                // Set the delay timer to the value of register VX.
                0x0015 => {}

                // FX18
                // Set the sound timer to the value of register VX.
                0x0018 => {}

                // FX1E
                // Add the value stored in register VX to register I.
                0x001E => {}

                // FX29
                // Set I to the memory address of the sprite data corresponding to the hexadecimal digit stored in register VX.
                0x0029 => {}

                // FX33
                // Store the binary-coded decimal equivalent of the value stored in register VX at addresses I, I + 1, and I + 2.
                0x0033 => {}

                // FX55
                // Store the values of registers V0 to VX inclusive in memory starting at address I.
                // I is set to I + X + 1 after operation.
                0x0055 => {}

                // FX65
                // Fill registers V0 to VX inclusive with the values stored in memory starting at address I.
                0x0065 => {}

                _ => dec_errror!(inst),
            },

            _ => dec_errror!(inst),
        }
    }
}

#[cfg(test)]
mod tests {}
