// macro_rules! instructions {
//     ($instr: tt, $ctx: ident, $(($t: ident, $f: ident, $i: ident, $($y: expr),*)),*) => (
//         match $instr {
//         $(
//             $i if machine_code::$t.0 == machine_code::$t.1 & $i => machine_code::$f($ctx, $($y, )*),
//         )*
//             0x0000 => panic!("Reached null insutruction"),
//             _ => panic!("Failed to decode instruction {:#04x}", $instr),
//         }
//     )
// }

use std::fmt::Debug;
use std::num::NonZeroU16;

use crate::Ctx;

type Register = u8;
type DirectAddress = u8;
type ImmediateValue = u8;

pub enum DecodeError {
    NullInstruction,
    InvalidInstruction(NonZeroU16),
}

impl Debug for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::NullInstruction => write!(f, "Reached null insutruction"),
            DecodeError::InvalidInstruction(i) => {
                write!(f, "Failed to decode instruction {:#04x}", i)
            }
        }
    }
}

macro_rules! instructions {
    ($instructions_name:ident, $(($variant:ident ($($param:ident $type:ident: $shift:literal & $mask:literal),*), $code:ident, $bitpattern:literal, $bitmask:literal)),* $(,)*) => {

#[derive(Debug, Clone, Copy)]
pub enum $instructions_name {
        $(
            $variant ($( $type, )*),
        )*
}

impl $instructions_name {
    pub fn decode(instr: u16) -> Result<Self, DecodeError> {
        match instr {
            $(
            _i if $bitpattern == (instr & $bitmask) => Ok($instructions_name::$variant ($( ( instr >> $shift) as u8 & $mask , )*)),
            )*
            0x0000 => Err(DecodeError::NullInstruction),
            _ => Err(DecodeError::InvalidInstruction (instr.try_into().unwrap())),
        }
    }


    pub fn encode(self) -> u16 {
        match self {
            $(
                $instructions_name::$variant ($( $param, )*) => { return $bitpattern $( | ($param as u16) << $shift )*;},
            )*
        }
    }

    pub fn execute(self, ctx: &mut Ctx) {
        match self {
            $(
                $instructions_name::$variant($($param, )*) => crate::machine_code::$code(ctx, $($param, )*),
            )*
        }
    }
}
    }
}

instructions!(
    Instr,
    (NoOp (), no_op, 0x0FFF, 0xFFFF),
    (LoadMemory (r Register: 8 & 0xf, xy DirectAddress: 0 & 0xff), load_memory, 0x1000, 0xF000),
    (LoadValue (r Register: 8 & 0xf, xy ImmediateValue: 0 & 0xff), load_value, 0x2000, 0xF000),
    (LoadIndirect (r Register: 4 & 0xf, s Register: 0 & 0xf), load_indirect, 0xD000, 0xFF00),
    (StoreMemory (r Register: 8 & 0xf, xy DirectAddress: 0 & 0xff), store_memory, 0x3000, 0xF000),
    (StoreIndirect (r Register: 4 & 0xf, s Register: 0 & 0xf), store_indirect, 0xE000, 0xFF00),
    (MoveRegister (r Register: 4 & 0xf, s Register: 0 & 0xf), move_register, 0x4000, 0xFF00),
    (AddInteger (r Register: 8 & 0xf, s Register: 4 & 0xf, t Register: 0 & 0xf), add_integer, 0x5000, 0xF000),
    (AddFloat (r Register: 8 & 0xf, s Register: 4 & 0xf, t Register: 0 & 0xf), add_float, 0x6000, 0xF000),
    (BitwiseOr (r Register: 8 & 0xf, s Register: 4 & 0xf, t Register: 0 & 0xf), bitwise_or, 0x7000, 0xF000),
    (BitwiseAnd (r Register: 8 & 0xf, s Register: 4 & 0xf, t Register: 0 & 0xf), bitwise_and, 0x8000, 0xF000),
    (BitwiseXor (r Register: 8 & 0xf, s Register: 4 & 0xf, t Register: 0 & 0xf), bitwise_xor, 0x9000, 0xF000),
    (BitwiseRotate (r Register: 8 & 0xf, x ImmediateValue: 0 & 0xf), bitwise_rotate, 0xA000, 0xF0F0), // ImmediateValue is 2 nibbles, but we're masking out one
    (Jump (xy DirectAddress: 0 & 0xff), jump, 0xB000, 0xFF00),
    (JumpIndirect (t Register: 0 & 0xf), jump_indirect, 0xF000, 0xFFF0),
    (JumpIfEq (r Register: 8 & 0xf, xy DirectAddress: 0 & 0xff), jump_if_eq, 0xB000, 0xF000),
    (JumpWithTest (r Register: 8 & 0xf, x u8: 4 & 0xf, t Register: 0 & 0xf), jump_with_test, 0xF000, 0xF000),
    (Halt (), halt, 0xC000, 0xFFFF),
);
