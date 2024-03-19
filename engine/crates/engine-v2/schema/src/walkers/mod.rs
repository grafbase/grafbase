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

pub use definition::*;
pub use field::*;
pub use field_set::*;
pub use input_object::*;
pub use input_value::*;
pub use interface::*;
pub use object::*;
pub use r#enum::*;
pub use r#type::*;
pub use resolver::*;
pub use scalar::*;
pub use union::*;

#[derive(Clone, Copy)]
pub struct SchemaWalker<'a, I = ()> {
    // 'item' instead of 'inner' to avoid confusion with TypeWalker.inner()
    pub(crate) item: I,
    pub(crate) schema: &'a Schema,
    pub(crate) names: &'a dyn Names,
}

impl<'a, I> SchemaWalker<'a, I> {
    pub fn new(item: I, schema: &'a Schema, names: &'a dyn Names) -> Self {
        Self { item, schema, names }
    }

    pub fn walk<Other>(&self, item: Other) -> SchemaWalker<'a, Other> {
        SchemaWalker {
            item,
            schema: self.schema,
            names: self.names,
        }
    }
}

impl<'a, Id: Copy> SchemaWalker<'a, Id>
where
    Schema: std::ops::Index<Id>,
{
    // Clippy complains because it's ambiguous with AsRef. But AsRef doesn't allow us to add the 'a
    // lifetime. I could rename to `to_ref()` or `ref()`, but doesn't feel better than `as_ref()`.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a <Schema as std::ops::Index<Id>>::Output {
        &self.schema[self.item]
    }

    pub fn id(&self) -> Id {
        self.item
    }
}

impl<'a> SchemaWalker<'a, ()> {
    pub fn definitions(&self) -> impl ExactSizeIterator<Item = DefinitionWalker<'a>> + 'a {
        let walker = *self;
        self.schema
            .definitions
            .iter()
            .map(move |definition| walker.walk(*definition))
    }

    pub fn query(&self) -> ObjectWalker<'a> {
        self.walk(self.schema.root_operation_types.query)
    }

    pub fn mutation(&self) -> Option<ObjectWalker<'a>> {
        self.schema.root_operation_types.mutation.map(|id| self.walk(id))
    }

    pub fn subscription(&self) -> Option<ObjectWalker<'a>> {
        self.schema.root_operation_types.subscription.map(|id| self.walk(id))
    }

    pub fn names(&self) -> &'a dyn Names {
        self.names
    }

    // See further up
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a Schema {
        self.schema
    }
}

impl<'a> std::ops::Deref for SchemaWalker<'a, ()> {
    type Target = Schema;

    fn deref(&self) -> &'a Self::Target {
        self.schema
    }
}
