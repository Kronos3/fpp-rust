use crate::analysis::Analysis;
use crate::errors::{SemanticError, SemanticResult};
use crate::semantics::{NameGroup, Scope, Symbol, SymbolInterface};
use fpp_ast::*;
use fpp_core::Spanned;
use std::ops::ControlFlow;

pub struct EnterSymbols {}

impl<'ast> EnterSymbols {
    pub fn new() -> EnterSymbols {
        Self {}
    }

    fn update_parent_symbol_map(&self, a: &mut Analysis<'ast>, sym: Symbol<'ast>) {
        match a.parent_symbol {
            None => {}
            Some(parent) => {
                a.parent_symbol_map.insert(sym, parent);
            }
        }
    }

    /// Enter a symbol into its own name group
    fn enter_symbol(
        &self,
        a: &mut Analysis<'ast>,
        sym: Symbol<'ast>,
        ng: NameGroup,
    ) -> SemanticResult {
        let res = a.nested_scope.current_mut().borrow_mut().put(ng, sym);
        match res {
            Ok(_) => {
                // We successfully added the symbol to the scope
                // Update the parent symbol map
                self.update_parent_symbol_map(a, sym);
            }
            _ => {}
        }

        res
    }
}

impl<'ast> Visitor<'ast> for EnterSymbols {
    type Break = ();
    type State = Analysis<'ast>;

    fn visit_trans_unit(
        &self,
        a: &mut Self::State,
        node: &'ast TransUnit,
    ) -> ControlFlow<Self::Break> {
        node.walk(a, self)
    }

    fn visit_def_abs_type(
        &self,
        a: &mut Analysis<'ast>,
        def: &'ast DefAbsType,
    ) -> ControlFlow<Self::Break> {
        self.enter_symbol(a, Symbol::AbsType(def), NameGroup::Type)
            .unwrap_or_else(|err| err.emit());
        ControlFlow::Continue(())
    }

    fn visit_def_alias_type(
        &self,
        a: &mut Analysis<'ast>,
        def: &'ast DefAliasType,
    ) -> ControlFlow<Self::Break> {
        self.enter_symbol(a, Symbol::AliasType(def), NameGroup::Type)
            .unwrap_or_else(|err| err.emit());
        ControlFlow::Continue(())
    }

    fn visit_def_array(
        &self,
        a: &mut Analysis<'ast>,
        def: &'ast DefArray,
    ) -> ControlFlow<Self::Break> {
        self.enter_symbol(a, Symbol::Array(def), NameGroup::Type)
            .unwrap_or_else(|err| err.emit());
        ControlFlow::Continue(())
    }

    fn visit_def_component_instance(
        &self,
        a: &mut Analysis<'ast>,
        def: &'ast DefComponentInstance,
    ) -> ControlFlow<Self::Break> {
        self.enter_symbol(
            a,
            Symbol::ComponentInstance(def),
            NameGroup::PortInterfaceInstance,
        )
        .unwrap_or_else(|err| err.emit());
        ControlFlow::Continue(())
    }

    fn visit_def_component(
        &self,
        a: &mut Analysis<'ast>,
        def: &'ast DefComponent,
    ) -> ControlFlow<Self::Break> {
        let sym = Symbol::Component(def);

        (|| -> SemanticResult {
            self.enter_symbol(a, sym, NameGroup::Component)?;
            self.enter_symbol(a, sym, NameGroup::StateMachine)?;
            self.enter_symbol(a, sym, NameGroup::Type)?;
            self.enter_symbol(a, sym, NameGroup::Value)?;

            Ok(())
        })()
        .unwrap_or_else(|err| err.emit());

        let scope = Scope::new();
        a.symbol_scope_map.insert(sym, scope.clone());
        a.nested_scope.push(scope);

        let save_paren = a.parent_symbol;
        a.parent_symbol = Some(sym);
        let res = def.walk(a, self);
        a.parent_symbol = save_paren;
        a.nested_scope.pop();

        res
    }

    fn visit_def_constant(
        &self,
        a: &mut Analysis<'ast>,
        def: &'ast DefConstant,
    ) -> ControlFlow<Self::Break> {
        self.enter_symbol(a, Symbol::Constant(def), NameGroup::Value)
            .unwrap_or_else(|err| err.emit());
        ControlFlow::Continue(())
    }

    fn visit_def_enum(
        &self,
        a: &mut Analysis<'ast>,
        def: &'ast DefEnum,
    ) -> ControlFlow<Self::Break> {
        let sym = Symbol::Enum(def);

        (|| -> SemanticResult {
            self.enter_symbol(a, sym, NameGroup::Type)?;
            self.enter_symbol(a, sym, NameGroup::Value)?;
            Ok(())
        })()
        .unwrap_or_else(|err| err.emit());

        let scope = Scope::new();
        a.symbol_scope_map.insert(sym, scope.clone());
        a.nested_scope.push(scope);

        let save_paren = a.parent_symbol;
        a.parent_symbol = Some(sym);
        let res = def.walk(a, self);
        a.parent_symbol = save_paren;
        a.nested_scope.pop();

        res
    }

    fn visit_def_enum_constant(
        &self,
        a: &mut Analysis<'ast>,
        def: &'ast DefEnumConstant,
    ) -> ControlFlow<Self::Break> {
        self.enter_symbol(a, Symbol::EnumConstant(def), NameGroup::Value)
            .unwrap_or_else(|err| err.emit());
        ControlFlow::Continue(())
    }

    fn visit_def_interface(
        &self,
        a: &mut Analysis<'ast>,
        def: &'ast DefInterface,
    ) -> ControlFlow<Self::Break> {
        self.enter_symbol(a, Symbol::Interface(def), NameGroup::PortInterface)
            .unwrap_or_else(|err| err.emit());
        ControlFlow::Continue(())
    }

    fn visit_def_module(
        &self,
        a: &mut Analysis<'ast>,
        def: &'ast DefModule,
    ) -> ControlFlow<Self::Break> {
        // Modules exist in all name groups and overlaps should not be detected as an error
        // We first need to check if a scope that this module will create already exists
        // If so we can just open that scope without adding a new one
        let existing_symbol = a
            .nested_scope
            .current()
            .borrow()
            .get(NameGroup::Value, &def.name.data);

        let (sym, scope) = match existing_symbol {
            Some(other @ Symbol::Module(_)) => {
                // We found a module symbol with the same name at the current level.
                // Re-open the scope.
                (
                    other,
                    a.symbol_scope_map
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

                for ng in NameGroup::all() {
                    a.nested_scope
                        .current_mut()
                        .borrow_mut()
                        .put(ng, sym)
                        .expect("failed to add module to name group");
                }

                (sym, Scope::new())
            }
        };

        a.symbol_scope_map.insert(sym, scope.clone());
        a.nested_scope.push(scope);

        let save_paren = a.parent_symbol;
        a.parent_symbol = Some(sym);
        let res = def.walk(a, self);
        a.parent_symbol = save_paren;

        a.nested_scope.pop();

        res
    }

    fn visit_def_state_machine(
        &self,
        a: &mut Analysis<'ast>,
        def: &'ast DefStateMachine,
    ) -> ControlFlow<Self::Break> {
        self.enter_symbol(a, Symbol::StateMachine(def), NameGroup::StateMachine)
            .unwrap_or_else(|err| err.emit());
        ControlFlow::Continue(())
    }

    fn visit_def_struct(
        &self,
        a: &mut Analysis<'ast>,
        def: &'ast DefStruct,
    ) -> ControlFlow<Self::Break> {
        self.enter_symbol(a, Symbol::Struct(def), NameGroup::Type)
            .unwrap_or_else(|err| err.emit());
        ControlFlow::Continue(())
    }

    fn visit_def_topology(
        &self,
        a: &mut Analysis<'ast>,
        def: &'ast DefTopology,
    ) -> ControlFlow<Self::Break> {
        self.enter_symbol(a, Symbol::Topology(def), NameGroup::PortInterfaceInstance)
            .unwrap_or_else(|err| err.emit());
        ControlFlow::Continue(())
    }
}
