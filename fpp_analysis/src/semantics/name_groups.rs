use fpp_macros::EnumMap;

#[derive(EnumMap, Copy, Clone)]
pub enum NameGroups {
    Component,
    Port,
    StateMachine,
    PortInterfaceInstance,
    PortInterface,
    Template,
    Type,
    Value,
}
