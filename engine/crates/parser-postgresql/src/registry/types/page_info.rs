use crate::registry::context::{InputContext, OutputContext};
use engine::registry::{MetaField, ObjectType};

pub(super) fn register(input_ctx: &InputContext<'_>, output_ctx: &mut OutputContext) {
    let object = ObjectType::new(
        input_ctx.type_name("PageInfo"),
        [
            MetaField::new("hasNextPage", "Boolean!"),
            MetaField::new("hasPreviousPage", "Boolean!"),
            MetaField::new("startCursor", "String"),
            MetaField::new("endCursor", "String"),
        ],
    );

    output_ctx.create_object_type(object);
}
