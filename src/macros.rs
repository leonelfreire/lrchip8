#[macro_export]
macro_rules! dec_reg1 {
    ($inst:expr) => {
        ((($inst & 0x0F00) >> 8) as u8)
    };
}

#[macro_export]
macro_rules! dec_reg2 {
    ($inst:expr) => {
        ((($inst & 0x00F0) >> 4) as u8)
    };
}

#[macro_export]
macro_rules! dec_value {
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
macro_rules! dec_errror {
    ($inst:expr) => {
        panic!("Unknown instruction: {:X}", $inst)
    };
}
