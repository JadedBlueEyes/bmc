use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

pub type MachineMemory = [u8; 256];
type MachineRegisters = [u8; 16];
type PC = u8;

type Register = u8;
type DirectAddress = u8;
type ImmediateValue = u8;

#[derive(Debug)]
pub struct Ctx {
    pub pc: PC,
    pub executing: bool,
    pub memory: MachineMemory,
    pub registers: MachineRegisters,
}


/// No operation. Carry on to the next instruction. The data fields must all be F.
pub fn no_op(_ctx: &mut Ctx) {}

/// Load from memory (direct addressing). Copy the data at memory address xy into register r.
pub fn load_memory(ctx: &mut Ctx, r_register: Register, xy_address: DirectAddress) {
    ctx.registers[r_register as usize] = ctx.memory[xy_address as usize];
}

/// Load value (immediate addressing). Copy xy into register r.
pub fn load_value(ctx: &mut Ctx, r_register: Register, xy_value: ImmediateValue) {
    ctx.registers[r_register as usize] = xy_value;
}

/// Load from memory (register indirect addressing). Copy the data from the memory location whose address is in register s. Place it in register r.
/// For example, if register s contains the value 86, and memory location 86 contains the value 7B, register r will be given the value 7B.
pub fn load_indirect(ctx: &mut Ctx, r_register: Register, s_register: Register) {
    ctx.registers[r_register as usize] = ctx.memory[ctx.registers[s_register as usize] as usize];
}

/// Store (direct addressing). Copy the contents of register r into memory at address xy.
pub fn store_memory(ctx: &mut Ctx, r_register: Register, xy_address: DirectAddress) {
    ctx.memory[xy_address as usize] = ctx.registers[r_register as usize];
}

/// Store in memory (register indirect addressing). Copy the data from register r. Place it in the memory location whose address is in register s.
/// For example, if register r contains the value EC, and register s contains the value 41, the value EC will be placed in memory at address 41.
pub fn store_indirect(ctx: &mut Ctx, r_register: Register, s_register: Register) {
    ctx.memory[ctx.registers[s_register as usize] as usize] = ctx.registers[r_register as usize];
}

/// Move. Copy the contents of register r into register s.
pub fn move_register(ctx: &mut Ctx, r_register: Register, s_register: Register) {
    ctx.registers[s_register as usize] = ctx.registers[r_register as usize];
}

/// Add as integers. Add the contents of register s to the contents of register t as twos complement integers. Put the result into register r.
pub fn add_integer(
    ctx: &mut Ctx,
    r_register: Register,
    s_register: Register,
    t_register: Register,
) {
    ctx.registers[r_register as usize] = ((ctx.registers[t_register as usize] as i8)
        + (ctx.registers[s_register as usize] as i8))
        as u8;
}

/// Add the contents of register s to the contents of register t as floating point values. Put the result into register r. The format is 1 sign bit, 3 exponent bits and 4 mantissa bits, SEEEMMMM, with 1 as negative.
pub fn add_float(ctx: &mut Ctx, r_register: Register, s_register: Register, t_register: Register) {
    let t = ctx.registers[t_register as usize];
    let s = ctx.registers[s_register as usize];
    let (t_sign_bit, t_exponent, t_mantissa) = (((t >> 7) & 0x1), ((t >> 4) & 0x7), (t & 0xf));
    let (s_sign_bit, s_exponent, s_mantissa) = (((s >> 7) & 0x1), ((s >> 4) & 0x7), (s & 0xf));
    let t_fixed = (1_i32 - 2 * (t_sign_bit as i32)) * ((t_mantissa as i32) << t_exponent);
    let s_fixed = (1_i32 - 2 * (s_sign_bit as i32)) * ((s_mantissa as i32) << s_exponent);
    let r = t_fixed + s_fixed;
    let r_sign_bit: u8 = if r.is_negative() { 1 } else { 0 };
    let r_fixed_mantissa: u32 = r.abs().try_into().unwrap();

    if r_fixed_mantissa == 0 {
        ctx.registers[r_register as usize] = 0;
        return;
    }

    // Attempt normalisation
    let mut excess = 8;
    while !((((r_fixed_mantissa >> excess) & 0xf >> 3) & 0x1) == 1 || excess == 0) {
        excess -= 1;
    }

    if excess > 7 {
        // Overflow
        ctx.registers[r_register as usize] = r_sign_bit << 7 | 7 << 4 | 0xf;
    } else {
        ctx.registers[r_register as usize] =
            r_sign_bit << 7 | excess << 4 | ((r_fixed_mantissa >> excess) & 0xf) as u8;
    }
    //

    // let t: f32 = f32::from_bits(
    //     ((t_signbit as u32) << 31) & ((t_exponent as u32) << 23) & (t_mantissa as u32),
    // );
    // let s: f32 = f32::from_bits(
    //     ((s_signbit as u32) << 31) & ((s_exponent as u32) << 23) & (s_mantissa as u32),
    // );
    // let r = (t + s).to_bits();
    // let (r_signbit, r_exponent, r_mantissa) = (((r >> 31) & 0x1), ((r >> 23) & 0x7), (r & 0xf));
}

/// OR. Carry out the bitwise OR operation on the contents of register s and the contents of register t. Put the result into register r.
pub fn bitwise_or(ctx: &mut Ctx, r_register: Register, s_register: Register, t_register: Register) {
    ctx.registers[r_register as usize] =
        ctx.registers[t_register as usize] | ctx.registers[s_register as usize];
}

/// AND. Carry out the bitwise AND operation on the contents of register s and the contents of register t. Put the result into register r.
pub fn bitwise_and(
    ctx: &mut Ctx,
    r_register: Register,
    s_register: Register,
    t_register: Register,
) {
    ctx.registers[r_register as usize] =
        ctx.registers[t_register as usize] & ctx.registers[s_register as usize];
}

/// XOR. Carry out the bitwise exclusive or operation on the contents of register s and the contents of register t. Put the result into register r.
pub fn bitwise_xor(
    ctx: &mut Ctx,
    r_register: Register,
    s_register: Register,
    t_register: Register,
) {
    ctx.registers[r_register as usize] =
        ctx.registers[t_register as usize] ^ ctx.registers[s_register as usize];
}

/// Rotate the contents of register r by x bits to the right. Update register r with the result.
pub fn bitwise_rotate(ctx: &mut Ctx, r_register: Register, x_amount: ImmediateValue) {
    ctx.registers[r_register as usize] >>= x_amount;
}

/// Jump to memory location xy. That is, the program counter is set to xy just before the next instruction is executed. Note that this is really a special case of the next instruction.
pub fn jump(ctx: &mut Ctx, xy_loc: DirectAddress) {
    ctx.pc = xy_loc;
}

/// Jump to register address. Jump to the memory address stored in register t. That is, the contents of register t are copied to the program counter. Note that this is really a special case of the next instruction.
pub fn jump_indirect(ctx: &mut Ctx, t_register: Register) {
    ctx.pc = ctx.registers[t_register as usize];
}

/// Jump if equal. If the contents of register r equal the contents of register 0, jump to memory location xy.
pub fn jump_if_eq(ctx: &mut Ctx, r_register: Register, xy_loc: DirectAddress) {
    // dbg!(ctx.registers[r_register as usize], ctx.registers[0], ctx.registers[r_register as usize] == ctx.registers[0]);
    if ctx.registers[r_register as usize] == ctx.registers[0] {
        // dbg!(xy_loc);
        ctx.pc = xy_loc;
    };
}

#[repr(u8)]
#[derive(FromPrimitive, Default)]
pub enum Test {
    Eq = 0,
    Neq = 1,
    Gte = 2,
    Lte = 3,
    Gt = 4,
    Lt = 5,
    #[default]
    Never,
}

/// Jump to register address with test. The contents of register r are compared to the contents of register 0 using a test which depends on x. If the result of the test is true, a jump is made to the memory address stored in register t.
/// The register values are treated as unsigned integers for the comparisons.
pub fn jump_with_test(ctx: &mut Ctx, r_register: Register, x_test: u8, t_register: Register) {
    if match FromPrimitive::from_u8(x_test).unwrap_or_default() {
        Test::Eq => ctx.registers[r_register as usize] == ctx.registers[0],
        Test::Neq => ctx.registers[r_register as usize] != ctx.registers[0],
        Test::Gte => ctx.registers[r_register as usize] >= ctx.registers[0],
        Test::Lte => ctx.registers[r_register as usize] <= ctx.registers[0],
        Test::Gt => ctx.registers[r_register as usize] > ctx.registers[0],
        Test::Lt => ctx.registers[r_register as usize] < ctx.registers[0],
        Test::Never => false,
    } {
        ctx.pc = ctx.registers[t_register as usize];
    };
}

/// Stop execution.
pub fn halt(ctx: &mut Ctx) {
    ctx.executing = false
}
