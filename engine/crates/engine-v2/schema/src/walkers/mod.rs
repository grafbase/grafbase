use crate::{Names, Schema};

mod definition;
mod r#enum;
mod field;
mod input_object;
mod input_value;
mod interface;
mod object;
mod scalar;
mod r#type;
mod union;

pub use definition::DefinitionWalker;
pub use field::{FieldResolverWalker, FieldWalker};
pub use input_object::InputObjectWalker;
pub use input_value::InputValueWalker;
pub use interface::InterfaceWalker;
pub use object::ObjectWalker;
pub use r#enum::EnumWalker;
pub use r#type::TypeWalker;
pub use scalar::ScalarWalker;
pub use union::UnionWalker;

#[derive(Clone, Copy)]
pub struct SchemaWalker<'a, Id> {
    pub id: Id,
    schema: &'a Schema,
    names: &'a dyn Names,
}

impl<'a, Id> SchemaWalker<'a, Id>
where
    Id: Copy,
{
    pub fn new(id: Id, schema: &'a Schema, names: &'a dyn Names) -> Self {
        Self { id, schema, names }
    }

    pub fn id(self) -> Id {
        self.id
    }

    pub fn walk<OtherId>(self, id: OtherId) -> SchemaWalker<'a, OtherId>
    where
        OtherId: Copy,
    {
        SchemaWalker {
            id,
            schema: self.schema,
            names: self.names,
        }
    }
}

impl<'a, Id: Copy> SchemaWalker<'a, Id>
where
    Schema: std::ops::Index<Id>,
{
    pub fn get(&self) -> &<Schema as std::ops::Index<Id>>::Output {
        &self.schema[self.id]
    }
}

impl<'a, Id: Copy> std::ops::Deref for SchemaWalker<'a, Id>
where
    Schema: std::ops::Index<Id>,
{
    type Target = <Schema as std::ops::Index<Id>>::Output;
    fn deref(&self) -> &Self::Target {
        &self.schema[self.id]
    }
}

pub struct RangeWalker<'a, T, Key> {
    schema: &'a Schema,
    names: &'a dyn Names,
    range: &'a [T],
    index: usize,
    key: Key,
}

impl<'a, T, Key, Id> Iterator for RangeWalker<'a, T, Key>
where
    Id: Copy,
    Key: Fn(&T) -> Option<Id>,
{
    type Item = SchemaWalker<'a, Id>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.range.get(self.index)?;
        let id = (self.key)(item)?;
        self.index += 1;
        Some(SchemaWalker::new(id, self.schema, self.names))
    }
}

impl<'a> SchemaWalker<'a, ()> {
    pub fn definitions(&'a self) -> impl Iterator<Item = DefinitionWalker<'a>> + 'a {
        let walker = self;
        self.schema
            .definitions
            .iter()
            .map(move |definition| walker.walk(*definition))
    }

    pub fn query(&'a self) -> ObjectWalker<'a> {
        self.walk(self.schema.root_operation_types.query)
    }

    pub fn mutation(&'a self) -> Option<ObjectWalker<'a>> {
        self.schema.root_operation_types.mutation.map(|id| self.walk(id))
    }

    pub fn subscription(&'a self) -> Option<ObjectWalker<'a>> {
        self.schema.root_operation_types.subscription.map(|id| self.walk(id))
    }

    pub fn names(&self) -> &'a dyn Names {
        self.names
    }
}

impl<'a> std::ops::Deref for SchemaWalker<'a, ()> {
    type Target = Schema;

    fn deref(&self) -> &Self::Target {
        self.schema
    }
}
