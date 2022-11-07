#[macro_export]
macro_rules! dec_reg_x {
    ($opcode:expr) => {
        ((($opcode & 0x0F00) >> 8) as usize)
    };
}

#[macro_export]
macro_rules! dec_reg_y {
    ($opcode:expr) => {
        ((($opcode & 0x00F0) >> 4) as usize)
    };
}

#[macro_export]
macro_rules! dec_val_nibble {
    ($opcode:expr) => {
        (($opcode & 0x000F) as u8)
    };
}

#[macro_export]
macro_rules! dec_val_byte {
    ($opcode:expr) => {
        (($opcode & 0x00FF) as u8)
    };
}

#[macro_export]
macro_rules! dec_mem_addr {
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

#[macro_export]
macro_rules! i {
    ($val:expr) => {
        ($val as usize)
    };
}
