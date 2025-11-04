use crate::errors::{SemanticError, SemanticResult};
use crate::semantics::SymbolInterface;
use fpp_core::Spanned;
use std::collections::HashMap;

#[derive(Debug)]
pub struct GenericNameSymbolMap<'a, S: SymbolInterface<'a>>(HashMap<&'a str, S>);

impl<'a, S: SymbolInterface<'a>> GenericNameSymbolMap<'a, S> {
    pub fn new() -> Self {
        Self { 0: HashMap::new() }
    }

    /** Get a symbol from the map. Return none if the name is not there. */
    pub fn get(&self, name: &str) -> Option<S> {
        match self.0.get(name) {
            None => None,
            Some(s) => Some(s.clone()),
        }
    }

    pub fn put(&mut self, symbol: S) -> SemanticResult {
        let name = symbol.name().data.as_str();
        match self.0.get(name) {
            None => {
                self.0.insert(name, symbol);
                Ok(())
            }
            Some(prev) => Err(SemanticError::RedefinedSymbol {
                name: name.to_string(),
                loc: symbol.name().span(),
                prev_loc: prev.node().span(),
            }),
        }
    }
}
