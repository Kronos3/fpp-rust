use crate::semantics::generic_name_symbol_map::GenericNameSymbolMap;
use crate::semantics::generic_scope::GenericScope;
use crate::semantics::SymbolInterface;
use fpp_util::EnumMap;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct GenericNestedScope<
    'a,
    NG: Copy,
    S: SymbolInterface<'a>,
    M: EnumMap<NG, GenericNameSymbolMap<'a, S>>,
>(Vec<Rc<RefCell<GenericScope<'a, NG, S, M>>>>);

impl<'a, NG: Copy, S: SymbolInterface<'a>, M: EnumMap<NG, GenericNameSymbolMap<'a, S>>>
    GenericNestedScope<'a, NG, S, M>
{
    pub fn new(global_scope: Rc<RefCell<GenericScope<'a, NG, S, M>>>) -> Self {
        Self {
            0: vec![global_scope],
        }
    }

    /// Push a new scope onto the stack
    pub fn push(&mut self, scope: Rc<RefCell<GenericScope<'a, NG, S, M>>>) {
        self.0.push(scope);
    }

    pub fn pop(&mut self) {
        self.0.pop();
    }

    /// Look up a symbol in all the scopes nested in this scope
    pub fn get(&self, name_group: NG, name: &str) -> Option<S> {
        // Work in the current scope and work out to the outermost
        self.0
            .iter()
            .rev()
            .find_map(|s| s.borrow().get(name_group, name))
    }

    pub fn current(&self) -> &Rc<RefCell<GenericScope<'a, NG, S, M>>> {
        self.0.last().unwrap()
    }

    pub fn current_mut(&mut self) -> &Rc<RefCell<GenericScope<'a, NG, S, M>>> {
        self.0.last_mut().unwrap()
    }
}
