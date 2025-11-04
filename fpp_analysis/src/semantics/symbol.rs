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

impl<'a> Symbol<'a> {
    pub fn name(&self) -> &fpp_ast::Ident {
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
