use crate::errors::SemanticResult;
use crate::semantics::generic_name_symbol_map::GenericNameSymbolMap;
use crate::semantics::SymbolInterface;
use fpp_util::EnumMap;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
pub struct GenericScope<
    'ast,
    NG,
    S: SymbolInterface<'ast>,
    M: EnumMap<NG, GenericNameSymbolMap<'ast, S>>,
>(M, PhantomData<NG>, PhantomData<&'ast S>);

impl<'ast, NG, S: SymbolInterface<'ast>, M: EnumMap<NG, GenericNameSymbolMap<'ast, S>>>
    GenericScope<'ast, NG, S, M>
{
    /// Construct a new scope
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            0: M::new(|_| GenericNameSymbolMap::new()),
            1: Default::default(),
            2: Default::default(),
        }))
    }

    /// Look up a symbol in this scope
    pub fn get(&self, name_group: NG, name: &str) -> Option<S> {
        self.0.get(name_group).get(name)
    }

    /// Put a name and symbol into the map.
    pub fn put(&mut self, name_group: NG, symbol: S) -> SemanticResult {
        self.0.get_mut(name_group).put(symbol)
    }
}
