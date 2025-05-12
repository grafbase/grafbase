mod concrete;
mod derived;
mod field;
mod polymorphic;
mod root;
mod typename;

pub(crate) use concrete::*;
pub(crate) use derived::*;
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
    pub default_fields: Vec<DefaultFieldShape>,
}
