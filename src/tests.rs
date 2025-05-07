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
    let meower = module.create_symbol("", SymbolBinding::Global);
    let func = module.create_func(func_symbol, func_sig);
    //todo!()
}

#[test]
fn interior_mutability_test() {
    let ptr = Box::into_raw(Box::new(42i32));
    let new_box = unsafe {
        foo(&NonNull::new_unchecked(ptr));
        Box::from_raw(ptr)
    };
    assert_eq!(*new_box, 43);
}

unsafe fn foo(ptr: &NonNull<i32>) {
    unsafe {
        *ptr.as_ptr() += 1;
    }
}
