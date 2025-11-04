use fpp_macros::EnumMap;

#[derive(EnumMap, Copy, Clone, Debug)]
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
