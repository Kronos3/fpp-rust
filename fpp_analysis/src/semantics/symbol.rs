use fpp_core::{Node};

pub trait SymbolInterface<'ast>: Clone {
    fn node(&self) -> Node;
    fn name(&self) -> &'ast fpp_ast::Ident;
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Symbol<'ast> {
    AbsType(&'ast fpp_ast::DefAbsType),
    AliasType(&'ast fpp_ast::DefAliasType),
    Array(&'ast fpp_ast::DefArray),
    Component(&'ast fpp_ast::DefComponent),
    ComponentInstance(&'ast fpp_ast::DefComponentInstance),
    Constant(&'ast fpp_ast::DefConstant),
    Enum(&'ast fpp_ast::DefEnum),
    EnumConstant(&'ast fpp_ast::DefEnumConstant),
    Interface(&'ast fpp_ast::DefInterface),
    Module(&'ast fpp_ast::DefModule),
    StateMachine(&'ast fpp_ast::DefStateMachine),
    Struct(&'ast fpp_ast::DefStruct),
    Topology(&'ast fpp_ast::DefTopology),
}

impl<'a> SymbolInterface<'a> for Symbol<'a> {
    fn node(&self) -> Node {
        match self {
            Symbol::AbsType(node) => node.node_id,
            Symbol::AliasType(node) => node.node_id,
            Symbol::Array(node) => node.node_id,
            Symbol::Component(node) => node.node_id,
            Symbol::ComponentInstance(node) => node.node_id,
            Symbol::Constant(node) => node.node_id,
            Symbol::Enum(node) => node.node_id,
            Symbol::EnumConstant(node) => node.node_id,
            Symbol::Interface(node) => node.node_id,
            Symbol::Module(node) => node.node_id,
            Symbol::StateMachine(node) => node.node_id,
            Symbol::Struct(node) => node.node_id,
            Symbol::Topology(node) => node.node_id,
        }
    }

    fn name(&self) -> &'a fpp_ast::Ident {
        match self {
            Symbol::AbsType(def) => &def.name,
            Symbol::AliasType(def) => &def.name,
            Symbol::Array(def) => &def.name,
            Symbol::Component(def) => &def.name,
            Symbol::ComponentInstance(def) => &def.name,
            Symbol::Constant(def) => &def.name,
            Symbol::Enum(def) => &def.name,
            Symbol::EnumConstant(def) => &def.name,
            Symbol::Interface(def) => &def.name,
            Symbol::Module(def) => &def.name,
            Symbol::StateMachine(def) => &def.name,
            Symbol::Struct(def) => &def.name,
            Symbol::Topology(def) => &def.name,
        }
    }
}
