use crate::analysis::Analysis;
use crate::errors::SemanticError;
use crate::semantics::{NameGroups, Scope, Symbol, SymbolInterface};
use fpp_ast::*;
use fpp_core::Spanned;
use std::cell::RefCell;
use std::ops::ControlFlow;
use std::rc::Rc;

pub(crate) struct EnterSymbols<'a> {
    a: &'a mut Analysis<'a>,
}

impl<'a> EnterSymbols<'a> {
    pub fn run<'analysis>(a: &'analysis mut Analysis<'a>, ast: &'a TranslationUnit)
    where
        'analysis: 'a,
    {
        let mut pass = EnterSymbols { a };
        let _ = ast.walk_ref(&mut pass);
    }

    fn update_parent_symbol_map(&mut self, sym: Symbol<'a>) {
        match self.a.parent_symbol {
            None => {}
            Some(parent) => {
                self.a.parent_symbol_map.insert(sym, parent);
            }
        }
    }

    fn enter_symbol(&mut self, sym: Symbol<'a>, ng: NameGroups) {
        let res = self.a.nested_scope.current_mut().borrow_mut().put(ng, sym);
        match res {
            Ok(_) => {
                // We successfully added the symbol to the scope
                // Update the parent symbol map
                self.update_parent_symbol_map(sym);
            }
            Err(err) => {
                // Emit the error to the compiler context
                err.emit();
            }
        }
    }
}

impl<'ast> Visitor<'ast> for EnterSymbols<'ast> {
    type Break = ();

    fn visit_def_abs_type(&mut self, def: &'ast DefAbsType) -> ControlFlow<Self::Break> {
        self.enter_symbol(Symbol::AbsType(def), NameGroups::Type);
        ControlFlow::Continue(())
    }

    fn visit_def_alias_type(&mut self, def: &'ast DefAliasType) -> ControlFlow<Self::Break> {
        self.enter_symbol(Symbol::AliasType(def), NameGroups::Type);
        ControlFlow::Continue(())
    }

    fn visit_def_array(&mut self, def: &'ast DefArray) -> ControlFlow<Self::Break> {
        self.enter_symbol(Symbol::Array(def), NameGroups::Type);
        ControlFlow::Continue(())
    }

    fn visit_def_component_instance(
        &mut self,
        def: &'ast DefComponentInstance,
    ) -> ControlFlow<Self::Break> {
        self.enter_symbol(
            Symbol::ComponentInstance(def),
            NameGroups::PortInterfaceInstance,
        );
        ControlFlow::Continue(())
    }

    fn visit_def_component(&mut self, def: &'ast DefComponent) -> ControlFlow<Self::Break> {
        let sym = Symbol::Component(def);
        self.enter_symbol(sym, NameGroups::Component);
        self.enter_symbol(sym, NameGroups::StateMachine);
        self.enter_symbol(sym, NameGroups::Type);
        self.enter_symbol(sym, NameGroups::Value);

        let save_paren = self.a.parent_symbol;
        self.a.parent_symbol = Some(sym);
        let res = def.walk_ref(self);
        self.a.parent_symbol = save_paren;

        res
    }

    fn visit_def_constant(&mut self, def: &'ast DefConstant) -> ControlFlow<Self::Break> {
        self.enter_symbol(Symbol::Constant(def), NameGroups::Value);
        ControlFlow::Continue(())
    }

    fn visit_def_enum(&mut self, def: &'ast DefEnum) -> ControlFlow<Self::Break> {
        let sym = Symbol::Enum(def);
        self.enter_symbol(sym, NameGroups::Type);
        self.enter_symbol(sym, NameGroups::Value);

        let save_paren = self.a.parent_symbol;
        self.a.parent_symbol = Some(sym);
        let res = def.walk_ref(self);
        self.a.parent_symbol = save_paren;

        res
    }

    fn visit_def_enum_constant(&mut self, def: &'ast DefEnumConstant) -> ControlFlow<Self::Break> {
        self.enter_symbol(Symbol::EnumConstant(def), NameGroups::Value);
        ControlFlow::Continue(())
    }

    fn visit_def_interface(&mut self, def: &'ast DefInterface) -> ControlFlow<Self::Break> {
        self.enter_symbol(Symbol::Interface(def), NameGroups::PortInterface);
        ControlFlow::Continue(())
    }

    fn visit_def_module(&mut self, def: &'ast DefModule) -> ControlFlow<Self::Break> {
        // Modules exist in all name groups and overlaps should not be detected as an error
        // We first need to check if a scope that this module will create already exists
        // If so we can just open that scope without adding a new one
        let existing_symbol = self
            .a
            .nested_scope
            .current()
            .borrow()
            .get(NameGroups::Value, &def.name.data);

        let (sym, scope) = match existing_symbol {
            Some(other @ Symbol::Module(_)) => {
                // We found a module symbol with the same name at the current level.
                // Re-open the scope.
                (
                    other,
                    self.a
                        .symbol_scope_map
                        .get(&other)
                        .expect("could not find scope for existing symbol")
                        .clone(),
                )
            }
            Some(other) => {
                // We found a non-module symbol with the same name at the current level.
                // This is an error.
                SemanticError::RedefinedSymbol {
                    name: def.name.data.clone(),
                    loc: def.name.span(),
                    prev_loc: other.name().span(),
                }
                .emit();

                return ControlFlow::Continue(());
            }
            None => {
                // We did not find a symbol with the same name at the current level.
                // Create a new module symbol now.
                let sym = Symbol::Module(def);
                let scope = Scope::new();

                for ng in NameGroups::all() {
                    self.a
                        .nested_scope
                        .current_mut()
                        .borrow_mut()
                        .put(ng, sym)
                        .expect("failed to add module to name group");
                }

                (sym, Rc::new(RefCell::new(scope)))
            }
        };

        self.a.nested_scope = self.a.nested_scope.push(scope);

        let save_paren = self.a.parent_symbol;
        self.a.parent_symbol = Some(sym);
        let res = def.walk_ref(self);
        self.a.parent_symbol = save_paren;

        res
    }

    fn visit_def_state_machine(&mut self, def: &'ast DefStateMachine) -> ControlFlow<Self::Break> {
        self.enter_symbol(Symbol::StateMachine(def), NameGroups::StateMachine);
        ControlFlow::Continue(())
    }

    fn visit_def_struct(&mut self, def: &'ast DefStruct) -> ControlFlow<Self::Break> {
        self.enter_symbol(Symbol::Struct(def), NameGroups::Type);
        ControlFlow::Continue(())
    }

    fn visit_def_topology(&mut self, def: &'ast DefTopology) -> ControlFlow<Self::Break> {
        self.enter_symbol(Symbol::Topology(def), NameGroups::PortInterfaceInstance);
        ControlFlow::Continue(())
    }
}
