pub type Cost = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Edge {
    Resolver(Cost),
    CanResolveField(Cost),
    Resolves,
    Field,
    TypenameField,
    Requires,
}
