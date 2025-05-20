#![allow(unused_mut, unused_variables)]
use iron_rs::*;

fn main() {
    let code = Module::new(Arch::Xr17032, System::Freestanding, |module| {
        let func_symbol = module.create_symbol("binop_const_test", SymbolBinding::Global);
        let func_sig = FuncSig::new(CallConv::Jackal, [], [FuncParam { ty: Ty::I32 }]);
        module.create_func(func_symbol, func_sig, |func| {
            let entry = func.entry_block();
            let const42 = entry.push_const(Const::U32(42));
            let const1337 = entry.push_const(Const::U32(1337));
            let const1_000_000 = entry.push_const(Const::U32(1_000_000));
            let const_neg_1 = entry.push_const(Const::U32(-1 as _));
            let add1 = entry.push_binop(BinOp::IAdd, const42, const1337);
            let add2 = entry.push_binop(BinOp::IAdd, const1_000_000, const_neg_1);
            let sub = entry.push_binop(BinOp::ISub, add1, add2);
            entry.push_return([sub]);
        });
        module.codegen()
    });
    println!("{code}");
    assert_eq!(
        code,
        ".section text\n\nbinop_const_test:\n.global binop_const_test\n.b0:\n    addi t0, zero, 1337\n    lui  t1, zero, 15\n    addi t2, t1, 16960\n    subi t1, zero, 1\n    addi t0, t0, 42\n    add  t1, t1, t2\n    sub  t0, t0, t1\n    mov  a3, t0\n    ret"
    );
}
