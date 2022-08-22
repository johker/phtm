module.exports = Object.freeze({

ID_OFFSET : 0,
TYPE_OFFSET : 2,
CMD_OFFSET : 4,
KEY_OFFSET : 6,
PAYLOAD_OFFSET : 8,
MSG_KEY_DIV : 1000,
DEF_PL_SIZE : 512,

MessageType: { 
    UNDEFINED: 0,
    CONFIGURATION: 1,
    DATA: 2,
    NETWORK: 3,
},

MessageCommand: { 
    RESERVED: 0,
    READ: 1,
    WRITE: 2,
    PRINT: 3,
    ACK: 4,
    INPUT: 5,
    REPLICATE: 6,
},

MessageKey: { 
    UNDEFINED: 0,
    C_ACTBTS: 1,
    C_RAWDAT: 2,
    C_SDRLEN: 3,
    D_INPUT: 1001,
    D_SPOOL: 1002,
},

NodeType: { 
    UNDEFINED: 0,
    SCALAR_ENCODER: 1,
    SPATIAL_POOLER: 2,
},

});
