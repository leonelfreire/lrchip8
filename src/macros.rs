#[macro_export]
macro_rules! dec_reg_x {
    ($inst:expr) => {
        ((($inst & 0x0F00) >> 8) as usize)
    };
}

#[macro_export]
macro_rules! dec_reg_y {
    ($inst:expr) => {
        ((($inst & 0x00F0) >> 4) as usize)
    };
}

#[macro_export]
macro_rules! dec_value8 {
    ($inst:expr) => {
        (($inst & 0x00FF) as u8)
    };
}

#[macro_export]
macro_rules! dec_mem_addr {
    ($inst:expr) => {
        (($inst & 0x0FFF) as u16)
    };
}

#[macro_export]
macro_rules! dec_error {
    ($inst:expr) => {
        panic!("Unknown instruction: {:X}", $inst)
    };
}
