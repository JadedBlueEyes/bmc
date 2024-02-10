use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

const MEMORY_SIZE: usize = 256;
const REGISTER_COUNT: usize = 16;

pub type MachineMemory = [u8; MEMORY_SIZE];
type MachineRegisters = [u8; REGISTER_COUNT];
type PC = u8;

type Register = u8;
type DirectAddress = u8;
type ImmediateValue = u8;

pub type Res = Result<(), Err>;

#[derive(Debug, PartialEq)]
#[cfg_attr(kani, derive(kani::Arbitrary))]
pub enum Err {
    FloatingPointSaturated,
    HaltExecution,
}
use crate::machine_code::Err::HaltExecution;

#[derive(Debug)]
pub struct Ctx {
    pub pc: PC,
    pub memory: MachineMemory,
    pub registers: MachineRegisters,
}

#[cfg(kani)]
impl kani::arbitrary::Arbitrary for Ctx {
    fn any() -> Self {
        let pc = u8::any();
        kani::assume(pc < (MEMORY_SIZE - 1) as u8);
        Self {
            memory: kani::any(),
            pc,
            registers: kani::any(),
        }
    }
}

/// No operation. Carry on to the next instruction. The data fields must all be F.
#[cfg_attr(kani, kani::requires(true))]
#[cfg_attr(kani, kani::ensures(result.is_ok()))]
pub fn no_op(_ctx: &mut Ctx) -> Res {
    Res::Ok(())
}

#[cfg(kani)]
#[kani::proof_for_contract(no_op)]
fn no_op_harness() {
    let mut ctx: Ctx = kani::any();
    let res = no_op(&mut ctx);
    assert!(res.is_ok())
}

/// Load from memory (direct addressing). Copy the data at memory address xy into register r.
#[cfg_attr(kani, kani::requires(r_register < REGISTER_COUNT as u8))]
#[cfg_attr(kani, kani::requires(xy_address < MEMORY_SIZE as u8))]
#[cfg_attr(kani, kani::ensures(ctx.registers[r_register as usize] == ctx.memory[xy_address as usize]))]
#[cfg_attr(kani, kani::ensures(result.is_ok()))]
pub fn load_memory(ctx: &mut Ctx, r_register: Register, xy_address: DirectAddress) -> Res {
    ctx.registers[r_register as usize] = ctx.memory[xy_address as usize];
    Res::Ok(())
}
// let mut ctx: Ctx = kani::any();

#[cfg(kani)]
#[kani::proof_for_contract(load_memory)]
fn load_memory_harness() {
    let mut ctx: Ctx = kani::any();
    let reg: Register = kani::any();
    let addr: DirectAddress = kani::any();
    kani::assume(reg < REGISTER_COUNT as u8);
    kani::assume(addr < MEMORY_SIZE as u8);
    load_memory(&mut ctx, reg, addr);
}

/// Load value (immediate addressing). Copy xy into register r.
// #[cfg_attr(kani, kani::requires(r_register < REGISTER_COUNT as u8))]
// #[cfg_attr(kani, kani::requires(ctx.registers.len() == REGISTER_COUNT))]
// #[cfg_attr(kani, kani::ensures(ctx.registers[r_register as usize] == xy_value))]
// #[cfg_attr(kani, kani::ensures(result.is_ok()))]
pub fn load_value(ctx: &mut Ctx, r_register: Register, xy_value: ImmediateValue) -> Res {
    ctx.registers[r_register as usize] = xy_value;
    Res::Ok(())
}

#[cfg(kani)]
// #[kani::proof_for_contract(load_value)]
// For some reason verifying the kani contract fails with
// SUMMARY:
//  ** 1 of 1503 failed
// Failed Checks: Check that ctx->registers[var_4] is assignable
//  File: "src/machine_code.rs", line 90, in machine_code::load_value_wrapper_18dbe5
#[kani::proof]
fn load_value_harness() {
    let mut ctx: Ctx = kani::any();
    let reg: Register = kani::any();
    let val: u8 = kani::any();
    kani::assume(reg < REGISTER_COUNT as u8);
    // ctx.registers[reg as usize] = val;
    load_value(&mut ctx, reg, val);
    assert_eq!(ctx.registers[reg as usize], val)
}

/// Load from memory (register indirect addressing). Copy the data from the memory location whose address is in register s. Place it in register r.
/// For example, if register s contains the value 86, and memory location 86 contains the value 7B, register r will be given the value 7B.
#[cfg_attr(kani, kani::requires(r_register < REGISTER_COUNT as u8))]
#[cfg_attr(kani, kani::requires(s_register < REGISTER_COUNT as u8))]
#[cfg_attr(kani, kani::ensures(ctx.registers[r_register as usize] == ctx.memory[ctx.registers[s_register as usize] as usize]))]
#[cfg_attr(kani, kani::ensures(result.is_ok()))]
pub fn load_indirect(ctx: &mut Ctx, r_register: Register, s_register: Register) -> Res {
    ctx.registers[r_register as usize] = ctx.memory[ctx.registers[s_register as usize] as usize];
    Res::Ok(())
}

#[cfg(kani)]
#[kani::proof_for_contract(load_indirect)]
fn load_indirect_harness() {
    let reg_1: Register = kani::any(); // Copy value to here
    let reg_2: Register = kani::any(); // Copy value from the memory location stored here
    kani::assume(reg_1 < REGISTER_COUNT as u8);
    kani::assume(reg_2 < REGISTER_COUNT as u8);
    let mem_loc: DirectAddress = kani::any(); // The memory location the value is stored in.
    kani::assume(mem_loc < MEMORY_SIZE as u8);

    let val: u8 = kani::any(); // The value

    let pc: u8 = kani::any();
    kani::assume(pc < (MEMORY_SIZE - 1) as u8);

    // State with everything in the correct place
    let mut ctx: Ctx = Ctx {
        memory: kani::any_where(|mem: &MachineMemory| mem[mem_loc as usize] == val),
        pc,
        registers: kani::any_where(|reg: &MachineRegisters| reg[reg_2 as usize] == mem_loc),
    };

    load_indirect(&mut ctx, reg_1, reg_2);
    assert_eq!(ctx.registers[reg_1 as usize], val)
}

#[cfg(test)]
#[test]
fn load_indirect_works() {
    let mut ctx = Ctx {
        pc: 0,
        memory: [0; MEMORY_SIZE],
        registers: [0; REGISTER_COUNT],
    };

    let reg_1 = 4;
    let reg_2 = 8;
    let val = 16;
    let mem_loc = 4;
    ctx.registers[reg_2 as usize] = mem_loc;
    ctx.memory[mem_loc as usize] = val;

    load_indirect(&mut ctx, reg_1, reg_2).unwrap();
    assert_eq!(ctx.memory[reg_1 as usize], val);
}

/// Store (direct addressing). Copy the contents of register r into memory at address xy.
#[cfg_attr(kani, kani::requires(r_register < REGISTER_COUNT as u8))]
#[cfg_attr(kani, kani::requires(xy_address < MEMORY_SIZE as u8))]
#[cfg_attr(kani, kani::ensures(ctx.memory[xy_address as usize] == ctx.registers[r_register as usize]))]
#[cfg_attr(kani, kani::ensures(result.is_ok()))]
pub fn store_memory(ctx: &mut Ctx, r_register: Register, xy_address: DirectAddress) -> Res {
    ctx.memory[xy_address as usize] = ctx.registers[r_register as usize];
    Res::Ok(())
}

#[cfg(kani)]
#[kani::proof_for_contract(store_memory)]
fn store_memory_harness() {
    let reg: Register = kani::any();
    let addr: DirectAddress = kani::any();
    kani::assume(reg < REGISTER_COUNT as u8);
    kani::assume(addr < MEMORY_SIZE as u8);

    let val: u8 = kani::any(); // The value

    let mut ctx: Ctx = kani::any();
    ctx.registers[reg as usize] = val;
    store_memory(&mut ctx, reg, addr);
    assert_eq!(ctx.memory[addr as usize], val)
}

/// Store in memory (register indirect addressing). Copy the data from register r. Place it in the memory location whose address is in register s.
/// For example, if register r contains the value EC, and register s contains the value 41, the value EC will be placed in memory at address 41.
#[cfg_attr(kani, kani::requires(r_register < REGISTER_COUNT as u8))]
#[cfg_attr(kani, kani::requires(s_register < REGISTER_COUNT as u8))]
#[cfg_attr(kani, kani::ensures(ctx.memory[ctx.registers[s_register as usize] as usize] == ctx.registers[r_register as usize]))]
#[cfg_attr(kani, kani::ensures(result.is_ok()))]
pub fn store_indirect(ctx: &mut Ctx, r_register: Register, s_register: Register) -> Res {
    ctx.memory[ctx.registers[s_register as usize] as usize] = ctx.registers[r_register as usize];
    Res::Ok(())
}

#[cfg(kani)]
#[kani::proof_for_contract(store_indirect)]
fn store_indirect_harness() {
    let reg_1: Register = kani::any();
    let reg_2: Register = kani::any();
    let addr: DirectAddress = kani::any();
    kani::assume(reg_1 < REGISTER_COUNT as u8);
    kani::assume(reg_2 < REGISTER_COUNT as u8);
    kani::assume(addr < MEMORY_SIZE as u8);

    let val: u8 = kani::any(); // The value

    let mut ctx: Ctx = kani::any();
    ctx.registers[reg_1 as usize] = val;
    ctx.registers[reg_2 as usize] = addr;
    store_indirect(&mut ctx, reg_1, reg_2);
    assert_eq!(ctx.memory[addr as usize], val)
}

/// Move. Copy the contents of register r into register s.
// #[cfg_attr(kani, kani::requires(r_register < REGISTER_COUNT as u8))]
// #[cfg_attr(kani, kani::requires(s_register < REGISTER_COUNT as u8))]
// #[cfg_attr(kani, kani::ensures(ctx.registers[r_register as usize] == ctx.registers[s_register as usize]))]
// #[cfg_attr(kani, kani::ensures(result.is_ok()))]
pub fn move_register(ctx: &mut Ctx, r_register: Register, s_register: Register) -> Res {
    ctx.registers[s_register as usize] = ctx.registers[r_register as usize];
    Res::Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn move_register_harness() {
    let reg_1: Register = kani::any();
    let reg_2: Register = kani::any();
    kani::assume(reg_1 < REGISTER_COUNT as u8);
    kani::assume(reg_2 < REGISTER_COUNT as u8);

    let val: u8 = kani::any(); // The value

    let mut ctx: Ctx = Ctx {
        registers: kani::any_where(|reg: &MachineRegisters| reg[reg_1 as usize] == val),
        ..kani::any()
    };
    // ctx.registers[reg_1 as usize] = val;
    move_register(&mut ctx, reg_1, reg_2);
    assert_eq!(ctx.registers[reg_2 as usize], val)
}

#[cfg(test)]
#[test]
fn move_register_works() {
    let mut ctx = Ctx {
        pc: 0,
        memory: [0; MEMORY_SIZE],
        registers: [0; REGISTER_COUNT],
    };

    let reg_1 = 4;
    let reg_2 = 8;
    let val = 16;
    ctx.registers[reg_1 as usize] = val;

    move_register(&mut ctx, reg_1, reg_2).unwrap();
    assert_eq!(ctx.registers[reg_2 as usize], val);
}

/// Add as integers. Add the contents of register s to the contents of register t as twos complement integers. Put the result into register r.
/// The add is wrapping
pub fn add_integer(
    ctx: &mut Ctx,
    r_register: Register,
    s_register: Register,
    t_register: Register,
) -> Res {
    ctx.registers[r_register as usize] = ((ctx.registers[t_register as usize] as i8)
        .wrapping_add(ctx.registers[s_register as usize] as i8))
        as u8;
    Res::Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn add_integer_harness() {
    let reg_1: Register = kani::any();
    let reg_2: Register = kani::any();
    let dest: Register = kani::any();
    kani::assume(reg_1 < REGISTER_COUNT as u8);
    kani::assume(reg_2 < REGISTER_COUNT as u8);
    kani::assume(dest < REGISTER_COUNT as u8);

    let val_1: u8 = kani::any();
    let val_2: u8 = kani::any();

    let mut ctx: Ctx = Ctx {
        registers: kani::any_where(|reg: &MachineRegisters| {
            reg[reg_1 as usize] == val_1 && reg[reg_2 as usize] == val_2
        }),
        ..kani::any()
    };
    // ctx.registers[reg_1 as usize] = val;
    add_integer(&mut ctx, dest, reg_1, reg_2);
    assert_eq!(ctx.registers[dest as usize], val_1.wrapping_add(val_2))
}

/// Add the contents of register s to the contents of register t as floating point values. Put the result into register r. The format is 1 sign bit, 3 exponent bits and 4 mantissa bits, SEEEMMMM, with 1 as negative.
/// The maximum value is 7, the minimum is -8
pub fn add_float(
    ctx: &mut Ctx,
    r_register: Register,
    s_register: Register,
    t_register: Register,
) -> Res {
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
        // It's just 0!
        ctx.registers[r_register as usize] = 0;
        return Res::Ok(());
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

    Res::Ok(())
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

#[cfg(kani)]
#[kani::proof]
fn add_float_harness() {
    let reg_1: Register = kani::any();
    let reg_2: Register = kani::any();
    let dest: Register = kani::any();
    kani::assume(reg_1 < REGISTER_COUNT as u8);
    kani::assume(reg_2 < REGISTER_COUNT as u8);
    kani::assume(dest < REGISTER_COUNT as u8);

    let val_1: u8 = kani::any();
    let val_2: u8 = kani::any();

    let mut ctx: Ctx = Ctx {
        registers: kani::any_where(|reg: &MachineRegisters| {
            reg[reg_1 as usize] == val_1 && reg[reg_2 as usize] == val_2
        }),
        ..kani::any()
    };
    // ctx.registers[reg_1 as usize] = val;
    add_float(&mut ctx, dest, reg_1, reg_2);
    // assert_eq!(ctx.registers[dest as usize], val_1.wrapping_add(val_2))
}

#[cfg(test)]
#[test]
#[should_panic]
fn add_float_works() {
    // TODO!
    let mut ctx = Ctx {
        pc: 0,
        memory: [0; MEMORY_SIZE],
        registers: [0; REGISTER_COUNT],
    };

    let reg_1 = 4;
    let reg_2 = 8;

    let dest = 12;
    ctx.registers[reg_1 as usize] = 0b00010100;
    ctx.registers[reg_2 as usize] = 0b00010100;

    let _ = dbg!(add_float(&mut ctx, dest, reg_1, reg_2));
    println!("{:b}", ctx.registers[dest as usize]);
    assert_eq!(ctx.registers[dest as usize], 0b00100100);
}

/// OR. Carry out the bitwise OR operation on the contents of register s and the contents of register t. Put the result into register r.
pub fn bitwise_or(
    ctx: &mut Ctx,
    r_register: Register,
    s_register: Register,
    t_register: Register,
) -> Res {
    ctx.registers[r_register as usize] =
        ctx.registers[t_register as usize] | ctx.registers[s_register as usize];
    Res::Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn bitwise_or_harness() {
    let reg_1: Register = kani::any();
    let reg_2: Register = kani::any();
    let dest: Register = kani::any();
    kani::assume(reg_1 < REGISTER_COUNT as u8);
    kani::assume(reg_2 < REGISTER_COUNT as u8);
    kani::assume(dest < REGISTER_COUNT as u8);

    let val_1: u8 = kani::any();
    let val_2: u8 = kani::any();

    let mut ctx: Ctx = Ctx {
        registers: kani::any_where(|reg: &MachineRegisters| {
            reg[reg_1 as usize] == val_1 && reg[reg_2 as usize] == val_2
        }),
        ..kani::any()
    };

    bitwise_or(&mut ctx, dest, reg_1, reg_2);
    assert_eq!(ctx.registers[dest as usize], (val_1 | val_2))
}

/// AND. Carry out the bitwise AND operation on the contents of register s and the contents of register t. Put the result into register r.
pub fn bitwise_and(
    ctx: &mut Ctx,
    r_register: Register,
    s_register: Register,
    t_register: Register,
) -> Res {
    ctx.registers[r_register as usize] =
        ctx.registers[t_register as usize] & ctx.registers[s_register as usize];
    Res::Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn bitwise_and_harness() {
    let reg_1: Register = kani::any();
    let reg_2: Register = kani::any();
    let dest: Register = kani::any();
    kani::assume(reg_1 < REGISTER_COUNT as u8);
    kani::assume(reg_2 < REGISTER_COUNT as u8);
    kani::assume(dest < REGISTER_COUNT as u8);

    let val_1: u8 = kani::any();
    let val_2: u8 = kani::any();

    let mut ctx: Ctx = Ctx {
        registers: kani::any_where(|reg: &MachineRegisters| {
            reg[reg_1 as usize] == val_1 && reg[reg_2 as usize] == val_2
        }),
        ..kani::any()
    };

    bitwise_and(&mut ctx, dest, reg_1, reg_2);
    assert_eq!(ctx.registers[dest as usize], (val_1 & val_2))
}

/// XOR. Carry out the bitwise exclusive or operation on the contents of register s and the contents of register t. Put the result into register r.
pub fn bitwise_xor(
    ctx: &mut Ctx,
    r_register: Register,
    s_register: Register,
    t_register: Register,
) -> Res {
    ctx.registers[r_register as usize] =
        ctx.registers[t_register as usize] ^ ctx.registers[s_register as usize];
    Res::Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn bitwise_xor_harness() {
    let reg_1: Register = kani::any();
    let reg_2: Register = kani::any();
    let dest: Register = kani::any();
    kani::assume(reg_1 < REGISTER_COUNT as u8);
    kani::assume(reg_2 < REGISTER_COUNT as u8);
    kani::assume(dest < REGISTER_COUNT as u8);

    let val_1: u8 = kani::any();
    let val_2: u8 = kani::any();

    let mut ctx: Ctx = Ctx {
        registers: kani::any_where(|reg: &MachineRegisters| {
            reg[reg_1 as usize] == val_1 && reg[reg_2 as usize] == val_2
        }),
        ..kani::any()
    };

    bitwise_xor(&mut ctx, dest, reg_1, reg_2);
    assert_eq!(ctx.registers[dest as usize], (val_1 ^ val_2))
}

/// Rotate the contents of register r by x bits to the right. Update register r with the result.
pub fn bitwise_rotate(ctx: &mut Ctx, r_register: Register, x_amount: ImmediateValue) -> Res {
    ctx.registers[r_register as usize] >>= x_amount;
    Res::Ok(())
}

#[cfg(never)] // Is there any way to silence kani about overflows?
#[kani::proof]
fn bitwise_rotate_harness() {
    let reg_1: Register = kani::any();
    let amount: ImmediateValue = kani::any_where(|&v| v <= 0xF);
    kani::assume(reg_1 < REGISTER_COUNT as u8);

    let val_1: u8 = kani::any();

    let mut ctx: Ctx = Ctx {
        registers: kani::any_where(|reg: &MachineRegisters| reg[reg_1 as usize] == val_1),
        ..kani::any()
    };

    bitwise_rotate(&mut ctx, reg_1, amount);
    use std::ops::Shr;
    assert_eq!(ctx.registers[reg_1 as usize], val_1 >> amount)
}

/// Jump to memory location xy. That is, the program counter is set to xy just before the next instruction is executed.
pub fn jump(ctx: &mut Ctx, xy_loc: DirectAddress) -> Res {
    ctx.pc = xy_loc;
    Res::Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn jump_harness() {
    let dest: DirectAddress = kani::any();
    kani::assume(dest < MEMORY_SIZE as u8);

    let mut ctx: Ctx = kani::any();

    jump(&mut ctx, dest);
    assert_eq!(ctx.pc, dest)
}

/// Jump to register address. Jump to the memory address stored in register t. That is, the contents of register t are copied to the program counter.
pub fn jump_indirect(ctx: &mut Ctx, t_register: Register) -> Res {
    ctx.pc = ctx.registers[t_register as usize];
    Res::Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn jump_indirect_harness() {
    let dest: DirectAddress = kani::any();
    let reg_1: Register = kani::any();
    kani::assume(dest < MEMORY_SIZE as u8);
    kani::assume(reg_1 < REGISTER_COUNT as u8);

    let mut ctx: Ctx = Ctx {
        registers: kani::any_where(|reg: &MachineRegisters| reg[reg_1 as usize] == dest),
        ..kani::any()
    };

    jump_indirect(&mut ctx, reg_1);
    assert_eq!(ctx.pc, dest)
}

/// Jump if equal. If the contents of register r equal the contents of register 0, jump to memory location xy.
pub fn jump_if_eq(ctx: &mut Ctx, r_register: Register, xy_loc: DirectAddress) -> Res {
    // dbg!(ctx.registers[r_register as usize], ctx.registers[0], ctx.registers[r_register as usize] == ctx.registers[0]);
    if ctx.registers[r_register as usize] == ctx.registers[0] {
        // dbg!(xy_loc);
        ctx.pc = xy_loc;
    };
    Res::Ok(())
}

#[cfg(kani)]
#[kani::proof]
fn jump_if_eq_harness_jump() {
    let dest: DirectAddress = kani::any();
    let reg_1: Register = kani::any();
    let val: u8 = kani::any();
    kani::assume(dest < MEMORY_SIZE as u8);
    kani::assume(reg_1 < REGISTER_COUNT as u8);

    let mut ctx: Ctx = Ctx {
        registers: kani::any_where(|reg: &MachineRegisters| {
            reg[reg_1 as usize] == val && reg[0] == val
        }),
        ..kani::any()
    };

    jump_if_eq(&mut ctx, reg_1, dest);
    assert_eq!(ctx.pc, dest)
}

#[cfg(kani)]
#[kani::proof]
fn jump_if_eq_harness_nojump() {
    let dest: DirectAddress = kani::any();
    let reg_1: Register = kani::any();
    let val: u8 = kani::any();
    kani::assume(dest < MEMORY_SIZE as u8);
    kani::assume(reg_1 < REGISTER_COUNT as u8);

    let mut ctx: Ctx = Ctx {
        registers: kani::any_where(|reg: &MachineRegisters| {
            reg[reg_1 as usize] == val && reg[0] != val
        }),
        ..kani::any()
    };

    jump_if_eq(&mut ctx, reg_1, dest);
    assert_ne!(ctx.pc, dest)
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
pub fn jump_with_test(
    ctx: &mut Ctx,
    r_register: Register,
    x_test: u8,
    t_register: Register,
) -> Res {
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
    Res::Ok(())
}

/// Stop execution.
pub fn halt(_ctx: &mut Ctx) -> Res {
    Res::Err(HaltExecution)
}

#[cfg(kani)]
#[kani::proof]
fn halt_harness() {
    let mut ctx: Ctx = kani::any();

    assert_eq!(halt(&mut ctx), Res::Err(HaltExecution));
}
