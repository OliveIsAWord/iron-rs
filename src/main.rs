#![allow(unused_mut, unused_variables)]
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
    let func_sig2 = func_sig1.clone();
    let mut meower: Vec<Func> = vec![];
    module.create_func(func_symbol1, func_sig1, |func1| {
        module.create_func(func_symbol2, func_sig2, |func2| {
            let param1 = func1.get_param(0);
            let entry2 = func2.entry_block();
            // both of these give errors for a borrow escaping a closure
            //entry2.push_return([param1]);
            //meower.push(func1);
        });
    });
}
