use {
    crate::symbols::{Symbol, Symbols},
    alloc::string::{String, ToString},
    thiserror::Error,
};

#[derive(Debug)]
pub struct NopSymbols;

impl Symbols for NopSymbols {
    type Error = NopSymbolsError;

    fn lookup(name: &str) -> Result<Symbol, Self::Error> {
        Err(NopSymbolsError::SymbolNotFound(name.to_string()))
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum NopSymbolsError {
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),
}
