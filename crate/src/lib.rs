use machine_code::{Ctx, Res};

pub mod highlight;
pub mod instructions;
pub mod lexer;
pub mod machine_code;
pub mod memory;
// mod interpreter;

pub fn execute(ctx: &mut Ctx, mut fuel: usize) -> (usize, Res) {
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
        let res = instr_dec.execute(ctx);
        if res.is_err() {
            return (fuel, res);
        };
    }
    (fuel, Res::Ok(()))
}
