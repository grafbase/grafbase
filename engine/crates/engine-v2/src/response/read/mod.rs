use crate::plan::PlanInput;

use super::ResponseBuilder;
mod selection_set;
mod ser;
mod view;

use schema::SchemaWalker;
pub use selection_set::{ReadField, ReadSelectionSet};
pub use view::{ResponseBoundaryItem, ResponseBoundaryObjectsView};

impl ResponseBuilder {
    pub fn read<'a>(&'a self, schema: SchemaWalker<'a, ()>, plan_input: PlanInput) -> ResponseBoundaryObjectsView<'a> {
        ResponseBoundaryObjectsView {
            schema,
            response: self,
            plan_input,
            extra_constant_fields: vec![],
        }
    }
}
