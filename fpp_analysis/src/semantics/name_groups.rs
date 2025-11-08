use fpp_macros::EnumMap;

#[derive(EnumMap, Copy, Clone, Debug)]
pub enum NameGroup {
    Component,
    Port,
    StateMachine,
    PortInterfaceInstance,
    PortInterface,
    Template,
    Type,
    Value,
}
