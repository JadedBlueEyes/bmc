use machine_code::Ctx;
use num_traits::FromPrimitive;

pub mod machine_code;
pub mod memory;
// mod interpreter;

macro_rules! decode_and_execute {
    ($instr: tt, $ctx: ident, $(($t: ident, $f: ident, $i: ident, $($y: expr),*)),*) => (
        match $instr {
        $(
            $i if machine_code::$t.0 == machine_code::$t.1 & $i => machine_code::$f($ctx, $($y, )*),
        )*
            0x0000 => panic!("Reached null insutruction"),
            _ => panic!("Failed to decode instruction {:#04x}", $instr),
        }
    )
}

pub fn execute(ctx: &mut Ctx, mut fuel: usize) -> usize {
    while let Some(remaining) = fuel.checked_sub(1) {
        fuel = remaining;
        let instr = u16::from_be_bytes(
            <[u8; 2]>::try_from(&ctx.memory[ctx.pc as usize..=ctx.pc as usize + 1]).unwrap(),
        );
        // println!("{:#04x} {:#04x}", ctx.pc, &instr);
        ctx.pc += 2;
        decode_and_execute!(
            instr,
            ctx,
            (NO_OP, no_op, i,),
            (
                LOAD_MEMORY,
                load_memory,
                i,
                (i >> 8) as u8 & 0xf,
                i as u8
            ),
            (
                LOAD_VALUE,
                load_value,
                i,
                (i >> 8) as u8 & 0xf,
                i as u8
            ),
            (
                LOAD_INDIRECT,
                load_indirect,
                i,
                (i >> 4) as u8 & 0xf,
                i as u8 & 0xf
            ),
            (
                STORE_MEMORY,
                store_memory,
                i,
                (i >> 8) as u8 & 0xf,
                i as u8
            ),
            (
                STORE_INDIRECT,
                store_indirect,
                i,
                (i >> 4) as u8 & 0xf,
                i as u8 & 0xf
            ),
            (
                MOVE_REGISTER,
                move_register,
                i,
                (i >> 4) as u8 & 0xf,
                i as u8 & 0xf
            ),
            (
                ADD_INTEGER,
                add_integer,
                i,
                (i >> 8) as u8 & 0xf,
                (i >> 4) as u8 & 0xf,
                i as u8 & 0xf
            ),
            (
                ADD_FLOAT,
                add_float,
                i,
                (i >> 8) as u8 & 0xf,
                (i >> 4) as u8 & 0xf,
                i as u8 & 0xf
            ),
            (
                BITWISE_OR,
                bitwise_or,
                i,
                (i >> 8) as u8 & 0xf,
                (i >> 4) as u8 & 0xf,
                i as u8 & 0xf
            ),
            (
                BITWISE_AND,
                bitwise_and,
                i,
                (i >> 8) as u8 & 0xf,
                (i >> 4) as u8 & 0xf,
                i as u8 & 0xf
            ),
            (
                BITWISE_XOR,
                bitwise_xor,
                i,
                (i >> 8) as u8 & 0xf,
                (i >> 4) as u8 & 0xf,
                i as u8 & 0xf
            ),
            (
                BITWISE_ROTATE,
                bitwise_rotate,
                i,
                (i >> 8) as u8 & 0xf,
                i as u8 & 0xf
            ),
            (JUMP, jump, i, i as u8),
            (JUMP_INDIRECT, jump_indirect, i, i as u8 & 0xf),
            (
                JUMP_IF_EQ,
                jump_if_eq,
                i,
                (i >> 8) as u8 & 0xf,
                i as u8
            ),
            (
                JUMP_WITH_TEST,
                jump_with_test,
                i,
                (i >> 8) as u8 & 0xf,
                FromPrimitive::from_u8((i >> 4) as u8 & 0xf).unwrap_or_default(),
                i as u8 & 0xf
            ),
            (HALT, halt, i,)
        );
        if !ctx.executing {
            break;
        };
    }
    fuel
}
