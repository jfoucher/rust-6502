use crate::computer::ADRESSING_MODE;
use crate::computer::decode;

pub fn get_adressing_mode(opcode: u8) -> ADRESSING_MODE {
    let cc = opcode & 3;
    let aaa = (opcode >> 5) & 7;
    let bbb = (opcode >> 2) & 7;

    let mut addressing_mode = ADRESSING_MODE::NONE;

    match cc {
        0 => {
            match bbb {
                0b000	=> addressing_mode = ADRESSING_MODE::IMMEDIATE,
                0b001	=> addressing_mode = ADRESSING_MODE::ZERO_PAGE,
                0b011	=> addressing_mode = ADRESSING_MODE::ABSOLUTE,
                0b101	=> addressing_mode = ADRESSING_MODE::ZERO_PAGE_X,
                0b111	=> addressing_mode = ADRESSING_MODE::ABSOLUTE_X,
                _ => {}
            };
        },
        1 => {
            match bbb {
                0b000	=> addressing_mode = ADRESSING_MODE::INDIRECT_X,
                0b001	=> addressing_mode = ADRESSING_MODE::ZERO_PAGE,
                0b010	=> addressing_mode = ADRESSING_MODE::IMMEDIATE,
                0b011	=> addressing_mode = ADRESSING_MODE::ABSOLUTE,
                0b100	=> addressing_mode = ADRESSING_MODE::INDIRECT_Y,
                0b101	=> addressing_mode = ADRESSING_MODE::ZERO_PAGE_X,
                0b110	=> addressing_mode = ADRESSING_MODE::ABSOLUTE_Y,
                0b111	=> addressing_mode = ADRESSING_MODE::ABSOLUTE_X,
                _ => {}
            };
        },
        2 => {
            match bbb {
                0b000	=> addressing_mode = ADRESSING_MODE::IMMEDIATE,
                0b001	=> addressing_mode = ADRESSING_MODE::ZERO_PAGE,
                0b010	=> addressing_mode = ADRESSING_MODE::ACCUMULATOR,
                0b011	=> addressing_mode = ADRESSING_MODE::ABSOLUTE,
                0b101	=> addressing_mode = if decode::get_opcode_name(opcode) == "STX" || decode::get_opcode_name(opcode) == "LDX" { ADRESSING_MODE::ZERO_PAGE_Y } else { ADRESSING_MODE::ZERO_PAGE_X },
                0b111	=> addressing_mode = if decode::get_opcode_name(opcode) == "LDX" { ADRESSING_MODE::ABSOLUTE_Y } else { ADRESSING_MODE::ABSOLUTE_X },
                _ => {}
            }
        },
        _ => {}
    }

    if opcode == 0x6C {
        addressing_mode = ADRESSING_MODE::INDIRECT;
    }
    if opcode == 0x4C {
        addressing_mode = ADRESSING_MODE::ABSOLUTE;
    }

    addressing_mode
}

pub fn get_opcode_name(opcode: u8) -> String {
    let cc = opcode & 3;
    let aaa = (opcode >> 5) & 7;
    let bbb = (opcode >> 2) & 7;

    let mut name: &str = "";

    match cc {
        0 => {
            match aaa {
                0b001	=> name = "BIT",
                0b010	=> name = "JMP",
                0b011	=> name = "JMP",
                0b100	=> name = "STY",
                0b101	=> name = "LDY",
                0b110	=> name = "CPY",
                0b111	=> name = "CPX",
                _ => {}
            };
            
        },
        1 => {
            match aaa {
                0b000	=> name = "ORA",
                0b001	=> name = "AND",
                0b010	=> name = "EOR",
                0b011	=> name = "ADC",
                0b100	=> name = "STA",
                0b101	=> name = "LDA",
                0b110	=> name = "CMP",
                0b111	=> name = "SBC",
                _ => {}
            };
        },
        2 => {
            match aaa {
                0b000	=> name = "ASL",
                0b001	=> name = "ROL",
                0b010	=> name = "LSR",
                0b011	=> name = "ROR",
                0b100	=> name = "STX",
                0b101	=> name = "LDX",
                0b110	=> name = "DEC",
                0b111	=> name = "INC",
                _ => {}
            };
        },
        _ => {}
    }

    match opcode {
        0x10 => name = "BPL",
        0x30 => name = "BMI",
        0x50 => name = "BVC",
        0x70 => name = "BVS",
        0x90 => name = "BCC",
        0xB0 => name = "BCS",
        0xD0 => name = "BNE",
        0xF0 => name = "BEQ",

        0 => name = "BRK",
        0x20 => name = "JSR",
        0x40 => name = "RTI",
        0x60 => name = "RTS",

        0x08 => name = "PHP",
        0x28 => name = "PLP",
        0x48 => name = "PHA",
        0x68 => name = "PLA",
        0x88 => name = "DEY",
        0xa8 => name = "TAY",
        0xc8 => name = "INY",
        0xe8 => name = "INX",

        0x18 => name = "CLC",
        0x38 => name = "SEC",
        0x58 => name = "CLI",
        0x78 => name = "SEI",
        0x98 => name = "TYA",
        0xB8 => name = "CLV",
        0xD8 => name = "CLD",
        0xF8 => name = "SED",

        0x8a => name = "TXA",
        0x9a => name = "TXS",
        0xaa => name = "TAX",
        0xba => name = "TSX",
        0xca => name = "DEX",
        0xea => name = "NOP",


        _ => {}
    }

    name.to_string()
}
