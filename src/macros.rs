#[macro_export]
macro_rules! dec_x {
    ($opcode:expr) => {
        ((($opcode & 0x0F00) >> 8) as usize)
    };
}

#[macro_export]
macro_rules! dec_y {
    ($opcode:expr) => {
        ((($opcode & 0x00F0) >> 4) as usize)
    };
}

#[macro_export]
macro_rules! dec_nibble {
    ($opcode:expr) => {
        (($opcode & 0x000F) as u8)
    };
}

#[macro_export]
macro_rules! dec_byte {
    ($opcode:expr) => {
        (($opcode & 0x00FF) as u8)
    };
}

#[macro_export]
macro_rules! dec_addr {
    ($opcode:expr) => {
        (($opcode & 0x0FFF) as u16)
    };
}

#[macro_export]
macro_rules! dec_error {
    ($opcode:expr) => {
        panic!("Unknown instruction: {:X}", $opcode)
    };
}
