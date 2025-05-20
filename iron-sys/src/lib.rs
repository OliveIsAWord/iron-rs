#[allow(clippy::pedantic, clippy::nursery)]
#[allow(unsafe_op_in_unsafe_fn, unnecessary_transmutes, non_camel_case_types, non_upper_case_globals)]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
pub use ffi::*;

impl PartialEq for InstKind {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for InstKind {}

impl From<InstKindGeneric> for InstKind {
    fn from(value: InstKindGeneric) -> Self {
        Self(value as u16)
    }
}

#[cfg(test)]
mod tests {
    use std::mem::MaybeUninit;

    use crate::*;
    #[test]
    fn its_alive() {
        unsafe {
            let module = module_new(Arch::Xr17032, System::Freestanding);
            let mut ipool = MaybeUninit::uninit();
            ipool_init(ipool.as_mut_ptr());
            let mut ipool = ipool.assume_init();
            let mut vregs = MaybeUninit::uninit();
            vrbuf_init(vregs.as_mut_ptr(), 2048);
            let mut vregs = vregs.assume_init();
            let f_sig = funcsig_new(CallConv::Jackal, 1, 1);
            (*funcsig_param(f_sig, 0)).ty = Ty::I32;
            (*funcsig_return(f_sig, 0)).ty = Ty::I32;
            let f_sym = symbol_new(module, c"id".as_ptr(), 0, SymbolBinding::Global);
            let f = func_new(module, f_sym, f_sig, &raw mut ipool, &raw mut vregs);
            let entry = (*f).entry_block;
            let ret = insert_before((*entry).bookend, inst_return(f));
            return_set_arg(ret, 0, *(*f).params);
            codegen(f);

            let mut db = MaybeUninit::uninit();
            db_init(db.as_mut_ptr(), 2048);
            let mut db = db.assume_init();
            //emit_ir_func(&raw mut db, f, false);
            emit_asm(&raw mut db, module);
            let bytes = std::slice::from_raw_parts(db.at, db.len);
            let string = std::str::from_utf8_unchecked(bytes).trim();
            assert_eq!(
                string,
                ".section text\n\nid:\n.global id\n.b0:\n    mov  t0, a0\n    mov  a3, t0\n    ret"
            );
        }
    }
}
