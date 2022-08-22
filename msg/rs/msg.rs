#[allow(non_camel_case_types)]

#[allow(dead_code)]

pub mod msg {

pub const ID_OFFSET: usize = 0;
pub const TYPE_OFFSET: usize = 2;
pub const CMD_OFFSET: usize = 4;
pub const KEY_OFFSET: usize = 6;
pub const PAYLOAD_OFFSET: usize = 8;
pub const MSG_KEY_DIV: usize = 1000;
pub const DEF_PL_SIZE: usize = 512;

#[derive(Primitive)]
pub enum MessageType { 
    UNDEFINED = 0,
    CONFIGURATION = 1,
    DATA = 2,
    NETWORK= 3
}
#[derive(Primitive)]
pub enum MessageCommand { 
    RESERVED = 0,
    READ = 1,
    WRITE = 2,
    PRINT = 3,
    ACK = 4,
    INPUT = 5,
    REPLICATE= 6
}
#[derive(Primitive)]
pub enum MessageKey { 
    UNDEFINED = 0,
    C_ACTBTS = 1,
    C_RAWDAT = 2,
    C_SDRLEN = 3,
    D_INPUT = 1001,
    D_SPOOL= 1002
}
#[derive(Primitive)]
pub enum NodeType { 
    UNDEFINED = 0,
    SCALAR_ENCODER = 1,
    SPATIAL_POOLER= 2
}
}
