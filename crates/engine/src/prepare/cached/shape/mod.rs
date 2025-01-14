mod concrete;
mod field;
mod polymorphic;

pub(crate) use concrete::*;
pub(crate) use field::*;
pub(crate) use polymorphic::*;

#[derive(Default, serde::Serialize, serde::Deserialize, id_derives::IndexedFields)]
pub(crate) struct Shapes {
    #[indexed_by(PolymorphicShapeId)]
    pub polymorphic: Vec<PolymorphicShapeRecord>,
    #[indexed_by(ConcreteShapeId)]
    pub concrete: Vec<ConcreteShapeRecord>,
    #[indexed_by(FieldShapeId)]
    pub fields: Vec<FieldShapeRecord>,
}
