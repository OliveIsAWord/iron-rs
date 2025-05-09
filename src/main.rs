use iron_rs::*;

fn main() {
    // Codegen a single identity function on i32.
    let mut module = Module::new(Arch::Xr17032, System::Freestanding);
    let func_symbol1 = module.create_symbol("f1", SymbolBinding::Global);
    let func_sig1 = FuncSig::new(
        CallConv::Jackal,
        [FuncParam { ty: Ty::I32 }],
        [FuncParam { ty: Ty::I32 }],
    );
    let func_symbol2 = module.create_symbol("f2", SymbolBinding::Global);
    let func_sig2 = FuncSig::new(
        CallConv::Jackal,
        [FuncParam { ty: Ty::I32 }],
        [FuncParam { ty: Ty::I32 }],
    );
    let func1 = module.create_func(func_symbol1, func_sig1);
    let func2 = module.create_func(func_symbol2, func_sig2);
    let param1 = func1.get_param(0);
    let param2 = func2.get_param(0);
    let entry1 = func1.entry_block();
    let entry2 = func2.entry_block();
    entry1.push_return([param2]);
    entry2.push_return([param1]);
    println!("{func1}");
    println!("{func2}");
    let code = module.codegen();
    println!("{code}");
}
