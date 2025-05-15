#![allow(unused_mut, unused_variables)]
use iron_rs::*;

fn main() {
    Module::new(Arch::Xr17032, System::Freestanding, |module| {
        let func_symbol = module.create_symbol("infinite_loop", SymbolBinding::Global);
        let func_sig = FuncSig::new(CallConv::Jackal, [], []);
        module.create_func(func_symbol, func_sig, |func| {
            let entry = func.entry_block();
            entry.push_jump(entry);
            println!("{func}");
        });
    });
}
