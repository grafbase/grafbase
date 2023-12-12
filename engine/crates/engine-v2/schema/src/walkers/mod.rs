use crate::{Names, Schema};

mod definition;
mod r#enum;
mod field;
mod field_set;
mod input_object;
mod input_value;
mod interface;
mod object;
mod resolver;
mod scalar;
mod r#type;
mod union;

pub use definition::DefinitionWalker;
pub use field::{FieldResolverWalker, FieldWalker};
pub use field_set::{FieldSetItemWalker, FieldSetWalker};
pub use input_object::InputObjectWalker;
pub use input_value::InputValueWalker;
pub use interface::InterfaceWalker;
pub use object::ObjectWalker;
pub use r#enum::EnumWalker;
pub use r#type::TypeWalker;
pub use resolver::ResolverWalker;
pub use scalar::ScalarWalker;
pub use union::UnionWalker;

#[derive(Clone, Copy)]
pub struct SchemaWalker<'a, I> {
    // 'wrapped' instead of 'inner' to avoid confusion with TypeWalker.inner()
    pub(crate) wrapped: I,
    pub(crate) schema: &'a Schema,
    pub(crate) names: &'a dyn Names,
}

impl<'a, I> SchemaWalker<'a, I> {
    pub fn new(wrapped: I, schema: &'a Schema, names: &'a dyn Names) -> Self {
        Self { wrapped, schema, names }
    }

    pub fn walk<Other>(self, wrapped: Other) -> SchemaWalker<'a, Other> {
        SchemaWalker {
            wrapped,
            schema: self.schema,
            names: self.names,
        }
    }
}

impl<'a, Id: Copy> SchemaWalker<'a, Id>
where
    Schema: std::ops::Index<Id>,
{
    pub fn get(&self) -> &'a <Schema as std::ops::Index<Id>>::Output {
        &self.schema[self.wrapped]
    }

    pub fn id(&self) -> Id {
        self.wrapped
    }
}

impl<'a, Id: Copy> std::ops::Deref for SchemaWalker<'a, Id>
where
    Schema: std::ops::Index<Id>,
{
    type Target = <Schema as std::ops::Index<Id>>::Output;
    fn deref(&self) -> &Self::Target {
        &self.schema[self.wrapped]
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

    pub fn get(&self) -> &'a Schema {
        self.schema
    }
}

impl<'a> std::ops::Deref for SchemaWalker<'a, ()> {
    type Target = Schema;

    fn deref(&self) -> &'a Self::Target {
        self.schema
    }
}
