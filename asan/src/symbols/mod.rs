//! # symbols
//! This module provides implementations symbol lookups. The ability to
//! substitute this functionality may be helpful for targets where
//! conventional symbol lookup is not possible, e.g. if libc is statically
//! linked
use {crate::GuestAddr, alloc::fmt::Debug, core::ffi::c_void, thiserror::Error};

pub mod cached;

#[cfg(feature = "libc")]
pub mod dlsym;

pub mod nop;

#[readonly::make]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: &'static str,
    pub addr: GuestAddr,
}

impl Symbol {
    pub fn new(name: &'static str, addr: GuestAddr) -> Self {
        Symbol { name, addr }
    }
}

pub trait Symbols: Debug + Sized + Send {
    type Error: Debug;
    fn lookup(name: &'static str) -> Result<Symbol, Self::Error>;
}

pub trait Function {
    type Func: Copy;
    const NAME: &'static str;
}

pub trait FunctionPointer: Function {
    fn as_ptr(symbol: Symbol) -> Result<Self::Func, FunctionPointerError>;
}

impl<T: Function> FunctionPointer for T {
    fn as_ptr(symbol: Symbol) -> Result<Self::Func, FunctionPointerError> {
        if symbol.name != Self::NAME {
            Err(FunctionPointerError::BadFunctionName(
                symbol.name,
                Self::NAME,
            ))?;
        }
        let pp_sym = (&symbol.addr) as *const GuestAddr as *const *mut c_void;
        let p_f = pp_sym as *const Self::Func;
        let f = unsafe { *p_f };
        Ok(f)
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum FunctionPointerError {
    #[error("Bad function name -  expected: {0:?}")]
    BadFunctionName(&'static str, &'static str),
}
