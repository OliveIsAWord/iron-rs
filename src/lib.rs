//! The Iron compiler backend.

#[cfg(test)]
mod tests;

use std::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

use iron_sys as ffi;

// these are probably safe to export too
#[allow(unused_imports)]
use ffi::{InstKind, InstKindGeneric, RegStatus, Regclass, SymbolKind, Trait, VReg};

pub use ffi::{Arch, CallConv, SymbolBinding, System, Ty};

#[must_use]
const unsafe fn nonnull<T>(ptr: *mut T) -> NonNull<T> {
    #[cfg(debug_assertions)]
    {
        NonNull::new(ptr).unwrap()
    }
    #[cfg(not(debug_assertions))]
    unsafe {
        NonNull::new_unchecked(ptr)
    }
}

#[must_use]
fn ipool_new() -> ffi::InstPool {
    let mut ipool = MaybeUninit::uninit();
    unsafe {
        ffi::ipool_init(ipool.as_mut_ptr());
        ipool.assume_init()
    }
}

#[must_use]
fn vrbuf_new(cap: usize) -> ffi::VRegBuffer {
    let mut vregs = MaybeUninit::uninit();
    assert!(cap >= 2);
    unsafe {
        ffi::vrbuf_init(vregs.as_mut_ptr(), cap);
        vregs.assume_init()
    }
}

pub struct Module {
    inner: NonNull<ffi::Module>,
    // TODO: some methods like `create_func` would like to take a shared reference to `self`,
    // in which case these would need to be wrapped with some interior mutability type.
    ipool: ffi::InstPool,
    vregs: ffi::VRegBuffer,
    // We own the memory for `Symbol` and `FuncSig` for each function
    _func_data: Vec<(Symbol, FuncSig)>,
}

impl Module {
    #[must_use]
    pub fn new(arch: Arch, system: System) -> Self {
        let inner = unsafe {
            let ptr = ffi::module_new(arch, system);
            nonnull(ptr)
        };
        Self {
            inner,
            ipool: ipool_new(),
            vregs: vrbuf_new(64),
            _func_data: vec![],
        }
    }

    #[must_use]
    pub fn create_symbol(&mut self, name: impl Into<String>, binding: SymbolBinding) -> Symbol {
        let name = name.into();
        let len = name.len();
        let Ok(len) = u16::try_from(len) else {
            panic!("attempted to create a symbol with a name more than u16::MAX bytes: {name:?}");
        };
        let ptr = if len == 0 {
            std::ptr::null()
        } else {
            name.as_ptr().cast()
        };
        let inner = unsafe { nonnull(ffi::symbol_new(self.inner.as_ptr(), ptr, len, binding)) };
        Symbol { inner, _name: name }
    }

    #[must_use]
    pub fn create_func<'a>(&'a mut self, symbol: Symbol, sig: FuncSig) -> Func<'a> {
        unsafe {
            (*self.inner.as_ptr()).funcs.first = std::ptr::null_mut();
        }
        //panic!("wtf");
        let inner = unsafe {
            nonnull(ffi::func_new(
                self.inner.as_ptr(),
                symbol.inner.as_ptr(),
                sig.0.as_ptr(),
                // NOTE: `FeFunc` holds a pointer to these two for its entire lifetime
                &raw mut self.ipool,
                &raw mut self.vregs,
            ))
        };
        self._func_data.push((symbol, sig));
        Func {
            inner,
            phantom: PhantomData,
        }
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        let &mut Self {
            inner,
            mut ipool,
            mut vregs,
            _func_data: _,
        } = self;
        unsafe {
            // `fe_func_destroy` is broken lmao
            let _ = inner;
            // ffi::module_destroy(inner.as_ptr());
            ffi::ipool_destroy(&raw mut ipool);
            ffi::vrbuf_destroy(&raw mut vregs);
        }
    }
}

pub struct Symbol {
    inner: NonNull<ffi::Symbol>,
    _name: String,
}

impl Drop for Symbol {
    fn drop(&mut self) {
        unsafe {
            ffi::symbol_destroy(self.inner.as_ptr());
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FuncParam {
    pub ty: Ty,
}

pub struct FuncSig(NonNull<ffi::FuncSig>);

impl FuncSig {
    #[must_use]
    pub fn new<IterParams, IterReturns>(
        call_conv: CallConv,
        params: IterParams,
        returns: IterReturns,
    ) -> Self
    where
        IterParams: IntoIterator<Item = FuncParam>,
        IterParams::IntoIter: ExactSizeIterator,
        IterReturns: IntoIterator<Item = FuncParam>,
        IterReturns::IntoIter: ExactSizeIterator,
    {
        let mut params = params.into_iter();
        let mut returns = returns.into_iter();
        let param_len = params.len();
        let return_len = returns.len();
        let Ok(param_len) = u16::try_from(param_len) else {
            panic!("number of parameters ({param_len}) was bigger than u16::MAX");
        };
        let Ok(return_len) = u16::try_from(return_len) else {
            panic!("number of returns ({return_len}) was bigger than u16::MAX");
        };
        let inner = unsafe { nonnull(ffi::funcsig_new(call_conv, param_len, return_len)) };
        for i in 0..param_len {
            let param = params.next().unwrap();
            let param = ffi::FuncParam { ty: param.ty };
            unsafe {
                *ffi::funcsig_param(inner.as_ptr(), i) = param;
            }
        }
        assert!(
            params.next().is_none(),
            "`params` violated ExactSizeIterator length"
        );
        for i in 0..return_len {
            let return_param = returns.next().unwrap();
            let return_param = ffi::FuncParam {
                ty: return_param.ty,
            };
            unsafe {
                *ffi::funcsig_return(inner.as_ptr(), i) = return_param;
            }
        }
        assert!(
            returns.next().is_none(),
            "`returns` violated ExactSizeIterator length"
        );
        Self(inner)
    }
    // TODO: should we have accessor methods for params and returns? that would mean exposing the FFI types, which may be hazardous
}

impl Drop for FuncSig {
    fn drop(&mut self) {
        let &mut Self(inner) = self;
        unsafe {
            ffi::funcsig_destroy(inner.as_ptr());
        }
    }
}

pub struct Func<'a> {
    inner: NonNull<ffi::Func>,
    phantom: PhantomData<&'a ()>,
}
