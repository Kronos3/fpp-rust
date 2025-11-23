pub enum SemanticTokenKind {
    Module,
    Topology,
    Component,
    Interface,
    ComponentInstance,
    Constant,
    GraphGroup,
    Port,
    Type,
    Modifier,
    InputPortInstance,
    OutputPortInstance,
    InputPortDecl,
    OutputPortDecl,
    SpecialPort,
    FormalParameter,

    StateMachine,
    StateMachineInstance,

    Action,
    Guard,
    Signal,
    State,

    // Dictionary entries
    Command,
    Event,
    Parameter,
    Telemetry,
    Record,
    Container,
}
