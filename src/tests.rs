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
    todo!()
}
