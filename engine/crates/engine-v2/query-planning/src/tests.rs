use std::{borrow::Cow, num::NonZero};

use engine::Positioned;
use engine_parser::types::{DocumentOperations, Selection, SelectionSet};
use itertools::Itertools;

use crate::Equivalence;

#[ctor::ctor]
fn setup_logging() {
    let filter = tracing_subscriber::filter::EnvFilter::builder()
        .parse(std::env::var("RUST_LOG").unwrap_or("engine_v2_query_planning=debug".to_string()))
        .unwrap();
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(filter)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .without_time()
        .init();
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
struct ResolverId(NonZero<u16>);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
struct FieldId(NonZero<u16>);

#[derive(Debug, id_derives::IndexedFields)]
struct Context {
    // Schema
    #[indexed_by(ResolverId)]
    resolvers: Vec<Vec<&'static str>>,

    // Query
    root_selection_set: Vec<FieldId>,
    #[indexed_by(FieldId)]
    fields: Vec<Field>,
}

#[derive(Debug)]
struct Field {
    name: String,
    subselection: Vec<FieldId>,
}

impl Context {
    fn init(resolvers: Vec<Vec<&'static str>>, query: &str) -> Self {
        let mut ctx = Context {
            resolvers,
            root_selection_set: Vec::new(),
            fields: Vec::new(),
        };

        let query = engine_parser::parse_query(query).unwrap();
        let DocumentOperations::Single(Positioned { node: op, .. }) = &query.operations else {
            unreachable!()
        };
        ctx.root_selection_set = ctx.bind_selection_set(&op.selection_set);
        ctx
    }

    fn bind_selection_set(&mut self, selection_set: &SelectionSet) -> Vec<FieldId> {
        let mut field_ids = Vec::new();
        for (field_id, Positioned { node: selection, .. }) in selection_set.items.iter().enumerate() {
            let field = selection.as_field().unwrap();
            let subselection = self.bind_selection_set(&field.selection_set.node);

            let field_id = self.fields.len().into();
            self.fields.push(Field {
                name: field.name.node.to_string(),
                subselection,
            });
            field_ids.push(field_id);
        }
        field_ids
    }

    fn requires(&self, resolver_id: usize, field: &str) -> Option<Vec<(usize, &'static str)>> {
        match (resolver_id, field) {
            (3, "book") => Some(vec![(1, "a")]),
            (4, "book") => Some(vec![(1, "b")]),
            (1, "a") => Some(vec![(2, "c")]),
            _ => None,
        }
    }
}

impl Equivalence for ResolverId {
    fn equiv(&self, other: &Self) -> bool {
        self == other
    }
}

impl crate::Context for Context {
    type FieldId = FieldId;

    type Resolver = ResolverId;

    fn field_ids(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + '_ {
        (0..self.fields.len()).map(FieldId::from)
    }

    fn root_selection_set(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + '_ {
        self.root_selection_set.iter().copied()
    }

    fn subselection(&self, field_id: Self::FieldId) -> impl ExactSizeIterator<Item = Self::FieldId> + '_ {
        self[field_id].subselection.iter().copied()
    }

    fn resolvers(
        &self,
        parent_resolver: Option<&Self::Resolver>,
        field_id: Self::FieldId,
    ) -> impl Iterator<Item = Self::Resolver> {
        self.resolvers
            .iter()
            .positions(move |resolver| resolver.contains(&self[field_id].name.as_str()))
            .map(ResolverId::from)
    }

    fn resolver_label(&self, resolver: &Self::Resolver) -> Cow<'_, str> {
        Cow::Owned(format!("{resolver:?}"))
    }

    fn field_label(&self, field_id: Self::FieldId) -> Cow<'_, str> {
        Cow::Borrowed(&self[field_id].name)
    }
}

#[test]
fn test_dummy() {
    let resolvers = vec![
        Vec::new(),
        vec!["a", "b"],
        vec!["c"],
        vec!["book", "author", "title"],
        vec!["book", "cook", "knive"],
        vec!["cook", "kitchen"],
    ];
    let ctx = Context::init(resolvers, "{ author { cook { kitchen } book { title } } }");
    let mut plan = crate::Plan::build(&ctx);

    println!("{}", plan.dot_graph());
    // let ctx = Context::init(resolvers, "{ author { cook { kitchen } book { title } } }");
    // let mut plan = Plan::build(&ctx);
    //
    // println!("{}", plan.dot_graph());

    unreachable!();
}
