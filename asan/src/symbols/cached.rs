use {
    crate::symbols::{Symbol, Symbols},
    alloc::{
        collections::BTreeMap,
        string::{String, ToString},
    },
    core::marker::PhantomData,
    spin::Mutex,
};

static SYMBOLS: Mutex<BTreeMap<String, Symbol>> = Mutex::new(BTreeMap::new());

#[derive(Debug)]
pub struct CachedSymbols<S: Symbols> {
    _phantom: PhantomData<S>,
}

impl<S: Symbols> Symbols for CachedSymbols<S> {
    type Error = S::Error;

    fn lookup(name: &'static str) -> Result<Symbol, Self::Error> {
        let mut lock = SYMBOLS.lock();
        let p_sym = if let Some(pp_sym) = lock.get(name) {
            pp_sym.clone()
        } else {
            let p_sym = S::lookup(name)?;
            lock.insert(name.to_string(), p_sym.clone());
            p_sym
        };
        Ok(p_sym)
    }
}
