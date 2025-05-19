use crate::*;

#[test]
fn its_alive() {
    // Codegen a single identity function on i32.
    let code = Module::new(Arch::Xr17032, System::Freestanding, |module| {
        let func_symbol = module.create_symbol("id", SymbolBinding::Global);
        let func_sig = FuncSig::new(
            CallConv::Jackal,
            [FuncParam { ty: Ty::I32 }],
            [FuncParam { ty: Ty::I32 }],
        );
        let _meower = module.create_symbol("", SymbolBinding::Global);
        module.create_func(func_symbol, func_sig, |func| {
            let param = func.get_param(0);
            let entry = func.entry_block();
            entry.push_return([param]);
            println!("{func}");
        });
        module.codegen()
    });
    println!("{code}");
    assert_eq!(
        code,
        ".section text\n\nid:\n.global id\n    mov  t0, a0\n    mov  a3, t0\n    ret"
    );
}

#[test]
fn binop_const_test() {
    let code = Module::new(Arch::Xr17032, System::Freestanding, |module| {
        let func_symbol = module.create_symbol("binop_const_test", SymbolBinding::Global);
        let func_sig = FuncSig::new(CallConv::Jackal, [], [FuncParam { ty: Ty::I32 }]);
        module.create_func(func_symbol, func_sig, |func| {
            let entry = func.entry_block();
            let const42 = entry.push_const(Const::U32(42));
            let const1337 = entry.push_const(Const::U32(1337));
            let const1_000_000 = entry.push_const(Const::U32(1_000_000));
            let const_neg_1 = entry.push_const(Const::U32(-1 as _));
            let add1 = entry.push_binop(BinOp::Add, const42, const1337);
            let add2 = entry.push_binop(BinOp::Add, const1_000_000, const_neg_1);
            let sub = entry.push_binop(BinOp::Sub, add1, add2);
            entry.push_return([sub]);
        });
        module.codegen()
    });
    println!("{code}");
    assert_eq!(
        code,
        ".section text\n\nbinop_const_test:\n.global binop_const_test\n    addi t0, zero, 1337\n    lui  t1, zero, 15\n    addi t2, t1, 16960\n    subi t1, zero, 1\n    addi t0, t0, 42\n    add  t1, t1, t2\n    sub  t0, t0, t1\n    mov  a3, t0\n    ret"
    );
}

#[test]
#[ignore = "a block jumping to itself causes infinite recursion bug in fe_codegen"]
fn infinite_loop() {
    let code = Module::new(Arch::Xr17032, System::Freestanding, |module| {
        let func_symbol = module.create_symbol("infinite_loop", SymbolBinding::Global);
        let func_sig = FuncSig::new(CallConv::Jackal, [], []);
        module.create_func(func_symbol, func_sig, |func| {
            let entry = func.entry_block();
            entry.push_jump(entry);
        });
        module.codegen()
    });
    println!("{code}");
    assert_eq!(code, "");
}

#[test]
#[should_panic(expected = "symbol length (65536) was greater than u16::MAX")]
fn symbol_too_long() {
    Module::new(Arch::Xr17032, System::Freestanding, |module| {
        _ = module.create_symbol("a".repeat(0x1_0000), SymbolBinding::Global);
    });
}

#[test]
#[should_panic(expected = "function symbol cannot have binding SharedImport")]
fn shared_import_function() {
    Module::new(Arch::Xr17032, System::Freestanding, |module| {
        let func_symbol = module.create_symbol("oopsie", SymbolBinding::SharedImport);
        let func_sig = FuncSig::new(CallConv::Jackal, [], []);
        module.create_func(func_symbol, func_sig, |func| println!("{func}"));
    });
}

#[test]
#[should_panic(expected = "parameter index out of bounds: the len is 0 but the index is 0")]
fn out_of_bounds_param() {
    Module::new(Arch::Xr17032, System::Freestanding, |module| {
        let func_symbol = module.create_symbol("out_of_bounds", SymbolBinding::Global);
        let func_sig = FuncSig::new(CallConv::Jackal, [], []);
        module.create_func(func_symbol, func_sig, |func| {
            func.get_param(0);
        });
    });
}

#[test]
#[should_panic(expected = "incorrect number of return values")]
fn incorrect_number_of_return_values() {
    // Codegen a single identity function on i32.
    Module::new(Arch::Xr17032, System::Freestanding, |module| {
        let func_symbol = module.create_symbol("wrong_return_len", SymbolBinding::Global);
        let func_sig = FuncSig::new(
            CallConv::Jackal,
            [FuncParam { ty: Ty::I32 }],
            [FuncParam { ty: Ty::I32 }],
        );
        module.create_func(func_symbol, func_sig, |func| {
            let entry = func.entry_block();
            entry.push_return([]);
        });
    });
}
