#![allow(unused_mut, unused_variables)]
use iron_rs::*;

fn main() {
    Module::new(Arch::Xr17032, System::Freestanding, |module1| {
        Module::new(Arch::Xr17032, System::Freestanding, |module2| {
            let symbol1 = module1.create_symbol("f1", SymbolBinding::Global);
            let symbol2 = module2.create_symbol("f2", SymbolBinding::Global);
            let sig = FuncSig::new(
                CallConv::Jackal,
                [FuncParam { ty: Ty::I32 }],
                [FuncParam { ty: Ty::I32 }],
            );
            let func_ref1 = module1.create_func(symbol1, sig.clone(), |f| f.get_ref());
            module2.create_func(symbol2, sig, |func2| {
                let block = func2.entry_block();
                // The following line causes a borrow check error.
                //block.push_direct_call(func_ref1);
            });
            // As does this one.
            //module2.edit_func(func_ref1, |_| {});

            // But this is fine!
            module1.edit_func(func_ref1, |func1| {
                let block = func1.entry_block();
                block.push_direct_call(func_ref1);
            });
        });
    });
}
