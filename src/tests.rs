use crate::*;

#[test]
fn its_alive() {
    // Codegen a single identity function on i32.
    let mut module = Module::new(Arch::Xr17032, System::Freestanding);
    let func_symbol = module.create_symbol("id", SymbolBinding::Global);
    let func_sig = FuncSig::new(
        CallConv::Jackal,
        [FuncParam { ty: Ty::I32 }],
        [FuncParam { ty: Ty::I32 }],
    );
    let _meower = module.create_symbol("", SymbolBinding::Global);
    let func = module.create_func(func_symbol, func_sig);
    let param = func.get_param(0);
    let entry = func.entry_block();
    entry.push_return([param]);
    let code = module.codegen();
    assert_eq!(code, "id:\n    mov  t0, a0\n    mov  a3, t0\n    ret");
}
