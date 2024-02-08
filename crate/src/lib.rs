use machine_code::Ctx;

pub mod machine_code;
pub mod memory;
pub mod lexer;
pub mod highlight;
pub mod instructions;
// mod interpreter;


pub fn execute(ctx: &mut Ctx, mut fuel: usize) -> usize {
    while let Some(remaining) = fuel.checked_sub(1) {
        fuel = remaining;
        let instr = u16::from_be_bytes(
            <[u8; 2]>::try_from(&ctx.memory[ctx.pc as usize..=ctx.pc as usize + 1]).unwrap(),
        );
        let instr_dec = instructions::Instr::decode(instr).unwrap();
        // assert_eq!(instr, instr_dec.encode());
        // dbg!(&instr, &ctx.pc);
        // println!("{:#04x} {:#04x}", ctx.pc, &instr);
        ctx.pc += 2;
        instr_dec.execute(ctx);
        if !ctx.executing {
            break;
        };
    }
    fuel
}
