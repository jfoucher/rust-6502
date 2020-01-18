use crate::computer::ADRESSING_MODE;
use crate::computer::decode;

pub fn get_adressing_mode(opcode: u8) -> ADRESSING_MODE {
    let bbb = (opcode >> 2) & 7;
    let cc = opcode & 3;
    
    if opcode == 0x6C {
        return ADRESSING_MODE::INDIRECT;
    }
    if opcode == 0x4C {
        return ADRESSING_MODE::ABSOLUTE;
    }

    match cc {
        0 => {
            match bbb {
                0b000	=> return ADRESSING_MODE::IMMEDIATE,
                0b001	=> return ADRESSING_MODE::ZERO_PAGE,
                0b011	=> return ADRESSING_MODE::ABSOLUTE,
                0b101	=> return ADRESSING_MODE::ZERO_PAGE_X,
                0b111	=> return ADRESSING_MODE::ABSOLUTE_X,
                _ => {}
            };
        },
        1 => {
            match bbb {
                0b000	=> return ADRESSING_MODE::INDIRECT_X,
                0b001	=> return ADRESSING_MODE::ZERO_PAGE,
                0b010	=> return ADRESSING_MODE::IMMEDIATE,
                0b011	=> return ADRESSING_MODE::ABSOLUTE,
                0b100	=> return ADRESSING_MODE::INDIRECT_Y,
                0b101	=> return ADRESSING_MODE::ZERO_PAGE_X,
                0b110	=> return ADRESSING_MODE::ABSOLUTE_Y,
                0b111	=> return ADRESSING_MODE::ABSOLUTE_X,
                _ => {}
            };
        },
        2 => {
            match bbb {
                0b000	=> return ADRESSING_MODE::IMMEDIATE,
                0b001	=> return ADRESSING_MODE::ZERO_PAGE,
                0b010	=> return ADRESSING_MODE::ACCUMULATOR,
                0b011	=> return ADRESSING_MODE::ABSOLUTE,
                0b101	=> if decode::get_opcode_name(opcode) == "STX" || decode::get_opcode_name(opcode) == "LDX" { return ADRESSING_MODE::ZERO_PAGE_Y } else { return ADRESSING_MODE::ZERO_PAGE_X },
                0b111	=> if decode::get_opcode_name(opcode) == "LDX" { return ADRESSING_MODE::ABSOLUTE_Y } else { return ADRESSING_MODE::ABSOLUTE_X },
                _ => {}
            }
        },
        _ => {}
    }

    

    ADRESSING_MODE::NONE
}

pub fn get_opcode_name<'a>(opcode: u8) -> &'a str {
    let cc = opcode & 3;
    let aaa = (opcode >> 5) & 7;

    match opcode {
        0x10 => return "BPL",
        0x30 => return "BMI",
        0x50 => return "BVC",
        0x70 => return "BVS",
        0x90 => return "BCC",
        0xB0 => return "BCS",
        0xD0 => return "BNE",
        0xF0 => return "BEQ",

        0 => return "BRK",
        0x20 => return "JSR",
        0x40 => return "RTI",
        0x60 => return "RTS",

        0x08 => return "PHP",
        0x28 => return "PLP",
        0x48 => return "PHA",
        0x68 => return "PLA",
        0x88 => return "DEY",
        0xa8 => return "TAY",
        0xc8 => return "INY",
        0xe8 => return "INX",

        0x18 => return "CLC",
        0x38 => return "SEC",
        0x58 => return "CLI",
        0x78 => return "SEI",
        0x98 => return "TYA",
        0xB8 => return "CLV",
        0xD8 => return "CLD",
        0xF8 => return "SED",

        0x8a => return "TXA",
        0x9a => return "TXS",
        0xaa => return "TAX",
        0xba => return "TSX",
        0xca => return "DEX",
        0xea => return "NOP",

        _ => {}
    }

    match cc {
        0 => {
            match aaa {
                0b001	=> return "BIT",
                0b010	=> return "JMP",
                0b011	=> return "JMP",
                0b100	=> return "STY",
                0b101	=> return "LDY",
                0b110	=> return "CPY",
                0b111	=> return "CPX",
                _ => {}
            };
            
        },
        1 => {
            match aaa {
                0b000	=> return "ORA",
                0b001	=> return "AND",
                0b010	=> return "EOR",
                0b011	=> return "ADC",
                0b100	=> return "STA",
                0b101	=> return "LDA",
                0b110	=> return "CMP",
                0b111	=> return "SBC",
                _ => {}
            };
        },
        2 => {
            match aaa {
                0b000	=> return "ASL",
                0b001	=> return "ROL",
                0b010	=> return "LSR",
                0b011	=> return "ROR",
                0b100	=> return "STX",
                0b101	=> return "LDX",
                0b110	=> return "DEC",
                0b111	=> return "INC",
                _ => {}
            };
        },
        _ => {}
    }

    

    ""
}
