mod concrete;
mod derive;
mod field;
mod polymorphic;
mod root;
mod typename;

pub(crate) use concrete::*;
pub(crate) use derive::*;
pub(crate) use field::*;
pub(crate) use polymorphic::*;
pub(crate) use root::*;
pub(crate) use typename::*;

#[derive(Default, serde::Serialize, serde::Deserialize, id_derives::IndexedFields)]
pub(crate) struct Shapes {
    #[indexed_by(PolymorphicShapeId)]
    pub polymorphic: Vec<PolymorphicShapeRecord>,
    #[indexed_by(ConcreteShapeId)]
    pub concrete: Vec<ConcreteShapeRecord>,
    #[indexed_by(FieldShapeId)]
    pub fields: Vec<FieldShapeRecord>,
    #[indexed_by(TypenameShapeId)]
    pub typename_fields: Vec<TypenameShapeRecord>,
    #[indexed_by(DerivedEntityShapeId)]
    pub derived_entities: Vec<DerivedEntityShapeRecord>,
    #[indexed_by(RootFieldsShapeId)]
    pub root_fields: Vec<RootFieldsShapeRecord>,
    #[indexed_by(DefaultFieldShapeId)]
    pub default_fields: Vec<DefaultFieldShapeRecord>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_field_size() {
        assert_eq!(std::mem::size_of::<FieldShapeRecord>(), 20);
        assert_eq!(std::mem::align_of::<FieldShapeRecord>(), 4);
    }

    #[test]
    fn check_concrete_size() {
        assert_eq!(std::mem::size_of::<ConcreteShapeRecord>(), 32);
        assert_eq!(std::mem::align_of::<ConcreteShapeRecord>(), 4);
    }

    #[test]
    fn check_polymorphic_size() {
        assert_eq!(std::mem::size_of::<PolymorphicShapeRecord>(), 32);
        assert_eq!(std::mem::align_of::<PolymorphicShapeRecord>(), 8);
    }
}
