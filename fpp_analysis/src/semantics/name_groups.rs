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

impl NameGroups {
    pub fn iter_variants() -> impl Iterator<Item = NameGroups> {
        vec![
            NameGroups::Component,
            NameGroups::Port,
            NameGroups::StateMachine,
            NameGroups::PortInterfaceInstance,
            NameGroups::PortInterface,
            NameGroups::Template,
            NameGroups::Type,
            NameGroups::Value,
        ].into_iter()
    }
}
