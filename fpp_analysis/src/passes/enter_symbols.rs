use crate::analysis::Analysis;
use crate::semantics::{NameGroups, Symbol};
use fpp_ast::*;
use std::ops::ControlFlow;

pub(crate) struct EnterSymbols<'a> {
    a: &'a Analysis<'a>,
    parent: Option<Symbol<'a>>,
}

impl<'a> EnterSymbols<'a> {
    pub fn run(a: &Analysis, ast: &TranslationUnit) {
        let mut pass = EnterSymbols { a, parent: None };
        let _ = ast.walk_ref(&mut pass);
    }

    fn enter_symbol(&mut self, sym: Symbol<'a>, ng: NameGroups) {

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
        self.enter_symbol(Symbol::ComponentInstance(def), NameGroups::PortInterfaceInstance);
        ControlFlow::Continue(())
    }

    fn visit_def_component(&mut self, def: &'ast DefComponent) -> ControlFlow<Self::Break> {
        let sym = Symbol::Component(def);
        self.enter_symbol(sym, NameGroups::Component);
        self.enter_symbol(sym, NameGroups::StateMachine);
        self.enter_symbol(sym, NameGroups::Type);
        self.enter_symbol(sym, NameGroups::Value);

        let save_paren = self.parent;
        self.parent = Some(sym);
        let res = def.walk_ref(self);
        self.parent = save_paren;

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

        let save_paren = self.parent;
        self.parent = Some(sym);
        let res = def.walk_ref(self);
        self.parent = save_paren;

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
        let sym = Symbol::Module(def);
        for ng in NameGroups::iter_variants() {
            self.enter_symbol(sym, ng);
        }

        let save_paren = self.parent;
        self.parent = Some(sym);
        let res = def.walk_ref(self);
        self.parent = save_paren;

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
