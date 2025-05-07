use iron_rs::*;

fn main() {
    let mut module = Module::new(Arch::Xr17032, System::Freestanding);
    let func_symbol = module.create_symbol("id", SymbolBinding::Global);
    let func_sig = FuncSig::new(
        CallConv::Jackal,
        [FuncParam { ty: Ty::I32 }],
        [FuncParam { ty: Ty::I32 }],
    );
    let meower = module.create_symbol("", SymbolBinding::Global);
    let func = module.create_func(func_symbol, func_sig);
}
