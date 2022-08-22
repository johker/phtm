export const ID_OFFSET:number = 0
export const TYPE_OFFSET:number = 2
export const CMD_OFFSET:number = 4
export const KEY_OFFSET:number = 6
export const PAYLOAD_OFFSET:number = 8
export const MSG_KEY_DIV:number = 1000
export const DEF_PL_SIZE:number = 512

export enum MessageType { 
    UNDEFINED= 0,
    CONFIGURATION= 1,
    DATA= 2,
    NETWORK= 3,
}

export enum MessageCommand { 
    RESERVED= 0,
    READ= 1,
    WRITE= 2,
    PRINT= 3,
    ACK= 4,
    INPUT= 5,
    REPLICATE= 6,
}

export enum MessageKey { 
    UNDEFINED= 0,
    C_ACTBTS= 1,
    C_RAWDAT= 2,
    C_SDRLEN= 3,
    D_INPUT= 1001,
    D_SPOOL= 1002,
}

export enum NodeType { 
    UNDEFINED= 0,
    SCALAR_ENCODER= 1,
    SPATIAL_POOLER= 2,
}

