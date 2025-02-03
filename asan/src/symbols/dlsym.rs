//! # dlsym
//! This module performs symbol lookup using the `dlsym` function. It supports
//! two variants:
//! - LookupTypeDefault: This performs the lookup using
//!   `dlsym(RTLD_DEFAULT, name)`
//! - LookupTypeNext: This performs the lookup using
//!   `dlsym(RTLD_NEXT, name)`
use {
    crate::symbols::Symbols,
    alloc::{
        ffi::{CString, NulError},
        fmt::Debug,
    },
    core::{
        ffi::{c_void, CStr},
        marker::PhantomData,
    },
    libc::{dlerror, dlsym, RTLD_DEFAULT, RTLD_NEXT},
    thiserror::Error,
};

pub trait LookupType: Debug {
    const HANDLE: *mut c_void;
}

#[derive(Debug)]
pub struct LookupTypeDefault;
impl LookupType for LookupTypeDefault {
    const HANDLE: *mut c_void = RTLD_DEFAULT;
}

#[derive(Debug, Eq, PartialEq)]
pub struct LookupTypeNext;
impl LookupType for LookupTypeNext {
    const HANDLE: *mut c_void = RTLD_NEXT;
}

#[derive(Debug, Eq, PartialEq)]
pub struct DlSymSymbols<L: LookupType> {
    _phantom: PhantomData<L>,
}

impl<L: LookupType> Symbols for DlSymSymbols<L> {
    type Error = DlSymSymbolsError;

    fn lookup<F: Copy>(name: &str) -> Result<F, Self::Error> {
        let name_string = name.to_string();
        let name_cstring = CString::new(name)?;
        let name_cstr: &CStr = name_cstring.as_c_str();
        let p_sym = unsafe { dlsym(L::HANDLE, name_cstr.as_ptr()) };
        if p_sym.is_null() {
            Err(DlSymSymbolsError::FailedToFindFunction(
                name_string,
                Self::get_error(),
            ))?;
        }
        let pp_sym = (&p_sym) as *const *mut c_void;
        let p_f = pp_sym as *const F;
        let f = unsafe { *p_f };
        Ok(f)
    }
}

impl<L: LookupType> DlSymSymbols<L> {
    const UNKNOWN_ERROR: &str = "Unknown error";

    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    fn get_error() -> &'static str {
        let error = unsafe { dlerror() };
        if error.is_null() {
            Self::UNKNOWN_ERROR
        } else {
            unsafe {
                CStr::from_ptr(error)
                    .to_str()
                    .unwrap_or(Self::UNKNOWN_ERROR)
            }
        }
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum DlSymSymbolsError {
    #[error("Bad function name: {0:?}")]
    BadFunctionName(#[from] NulError),
    #[error("Failed to find function: {0}, error: {1}")]
    FailedToFindFunction(String, &'static str),
}
