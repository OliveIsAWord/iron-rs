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
    assert_eq!(code, "id:\n    mov  t0, a0\n    mov  a3, t0\n    ret");
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
