use std::{borrow::Cow, num::NonZero};

use engine_parser::{
    types::{DocumentOperations, SelectionSet},
    Positioned,
};
use schema::{Definition, FieldDefinitionId, ObjectDefinition, Schema, Version};
use walker::Walk;

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
struct Operation {
    // Query
    root_selection_set: Vec<FieldId>,
    #[indexed_by(FieldId)]
    fields: Vec<Field>,
}

#[derive(Debug)]
struct Field {
    name: String,
    definition_id: Option<FieldDefinitionId>,
    subselection: Vec<FieldId>,
}

impl Operation {
    fn bind(schema: &Schema, query: &str) -> Self {
        let mut ctx = Operation {
            root_selection_set: Vec::new(),
            fields: Vec::new(),
        };

        let query = engine_parser::parse_query(query).unwrap();
        let DocumentOperations::Single(Positioned { node: op, .. }) = &query.operations else {
            unreachable!()
        };
        ctx.root_selection_set = ctx.bind_selection_set(
            schema.graph.root_operation_types_record.query_id.walk(schema),
            &op.selection_set,
        );
        ctx
    }

    fn bind_selection_set(&mut self, parent: ObjectDefinition<'_>, selection_set: &SelectionSet) -> Vec<FieldId> {
        let mut field_ids = Vec::new();
        for Positioned { node: selection, .. } in &selection_set.items {
            let field = selection.as_field().unwrap();
            if let Some(definition) = parent.fields().find(|def| def.name() == field.name.node.as_str()) {
                let subselection = match definition.ty().definition() {
                    Definition::Object(obj) => self.bind_selection_set(obj, &field.selection_set.node),
                    _ => Vec::new(),
                };

                self.fields.push(Field {
                    name: field.name.node.to_string(),
                    definition_id: Some(definition.id()),
                    subselection,
                });
            } else {
                self.fields.push(Field {
                    name: field.name.node.to_string(),
                    definition_id: None,
                    subselection: Vec::new(),
                });
            }
            let field_id = (self.fields.len() - 1).into();
            field_ids.push(field_id);
        }
        field_ids
    }
}

impl crate::Operation for Operation {
    type FieldId = FieldId;

    fn field_ids(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + '_ {
        (0..self.fields.len()).map(FieldId::from)
    }

    fn root_selection_set(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + '_ {
        self.root_selection_set.iter().copied()
    }

    fn subselection(&self, field_id: Self::FieldId) -> impl ExactSizeIterator<Item = Self::FieldId> + '_ {
        self[field_id].subselection.iter().copied()
    }

    fn field_label(&self, field_id: Self::FieldId) -> Cow<'_, str> {
        Cow::Borrowed(&self[field_id].name)
    }

    fn field_defintion(&self, field_id: Self::FieldId) -> Option<FieldDefinitionId> {
        self[field_id].definition_id
    }

    fn field_satisfies(&self, field_id: Self::FieldId, requirement: schema::RequiredField<'_>) -> bool {
        self[field_id].definition_id == Some(requirement.definition_id)
    }

    fn create_extra_field(&mut self, requirement: schema::RequiredField<'_>) -> Self::FieldId {
        self.fields.push(Field {
            name: requirement.definition().name().to_string(),
            definition_id: Some(requirement.definition_id),
            subselection: Vec::new(),
        });
        (self.fields.len() - 1).into()
    }
}

const SCHEMA: &str = r###"
enum join__Graph {
  ACCOUNTS @join__graph(name: "accounts", url: "http://accounts:4001/graphql")
  INVENTORY @join__graph(name: "inventory", url: "http://inventory:4002/graphql")
  PRODUCTS @join__graph(name: "products", url: "http://products:4003/graphql")
  REVIEWS @join__graph(name: "reviews", url: "http://reviews:4004/graphql")
}

type Product
  @join__type(graph: INVENTORY, key: "upc")
  @join__type(graph: PRODUCTS, key: "upc")
  @join__type(graph: REVIEWS, key: "upc")
{
  upc: String!
  weight: Int @join__field(graph: INVENTORY, external: true) @join__field(graph: PRODUCTS)
  price: Int @join__field(graph: INVENTORY, external: true) @join__field(graph: PRODUCTS)
  inStock: Boolean @join__field(graph: INVENTORY)
  shippingEstimate: Int @join__field(graph: INVENTORY, requires: "price weight")
  name: String @join__field(graph: PRODUCTS)
  reviews: [Review] @join__field(graph: REVIEWS)
}

type Query
  @join__type(graph: ACCOUNTS)
  @join__type(graph: INVENTORY)
  @join__type(graph: PRODUCTS)
  @join__type(graph: REVIEWS)
{
  me: User @join__field(graph: ACCOUNTS)
  user(id: ID!): User @join__field(graph: ACCOUNTS)
  users: [User] @join__field(graph: ACCOUNTS)
  topProducts(first: Int = 5): [Product] @join__field(graph: PRODUCTS)
}

type Review
  @join__type(graph: REVIEWS, key: "id")
{
  id: ID!
  body: String
  product: Product
  author: User @join__field(graph: REVIEWS, provides: "username")
}

type User
  @join__type(graph: ACCOUNTS, key: "id")
  @join__type(graph: REVIEWS, key: "id")
{
  id: ID!
  name: String @join__field(graph: ACCOUNTS)
  username: String @join__field(graph: ACCOUNTS) @join__field(graph: REVIEWS, external: true)
  birthday: Int @join__field(graph: ACCOUNTS)
  reviews: [Review] @join__field(graph: REVIEWS)
}
"###;

#[test]
fn test_basic_operation_graph() {
    let graph = federated_graph::from_sdl(SCHEMA).unwrap();
    let config = config::VersionedConfig::V6(config::latest::Config::from_graph(graph)).into_latest();
    let schema = Schema::build(config, Version::from(Vec::new())).unwrap();

    let mut operation = Operation::bind(
        &schema,
        r#"
    {
        topProducts {
            name
            reviews {
                author {
                    name
                }
            }
        }
    }
    "#,
    );
    let graph = crate::OperationGraph::new(&schema, &mut operation);

    insta::assert_snapshot!(graph.to_dot_graph(), @r##"
    digraph {
        0 [ label = "root"]
        1 [ label = "name"]
        2 [ label = "name"]
        3 [ label = "author"]
        4 [ label = "reviews"]
        5 [ label = "topProducts"]
        6 [ label = "topProducts@Root#products",shape=box,style=dashed,color=blue]
        7 [ label = "Root#products",shape=box,color=blue]
        8 [ label = "reviews@FedEntity#reviews",shape=box,style=dashed,color=blue]
        9 [ label = "FedEntity#reviews",shape=box,color=blue]
        10 [ label = "author@FedEntity#reviews",shape=box,style=dashed,color=blue]
        11 [ label = "name@FedEntity#accounts",shape=box,style=dashed,color=blue]
        12 [ label = "FedEntity#accounts",shape=box,color=blue]
        13 [ label = "name@Root#products",shape=box,style=dashed,color=blue]
        14 [ label = "id"]
        15 [ label = "id@FedEntity#reviews",shape=box,style=dashed,color=blue]
        16 [ label = "upc"]
        17 [ label = "upc@Root#products",shape=box,style=dashed,color=blue]
        0 -> 5 [ label = "Field" ]
        0 -> 7 [ label = "Resolver(0)" color=blue]
        7 -> 6 [ label = "CanResolveField(0)" color=blue]
        6 -> 5 [ label = "Resolves" color=blue]
        5 -> 4 [ label = "Field" ]
        6 -> 9 [ label = "Resolver(0)" color=blue]
        9 -> 8 [ label = "CanResolveField(0)" color=blue]
        8 -> 4 [ label = "Resolves" color=blue]
        4 -> 3 [ label = "Field" ]
        10 -> 3 [ label = "Resolves" color=blue]
        8 -> 10 [ label = "CanResolveField(0)" color=blue]
        3 -> 2 [ label = "Field" ]
        10 -> 12 [ label = "Resolver(0)" color=blue]
        12 -> 11 [ label = "CanResolveField(0)" color=blue]
        11 -> 2 [ label = "Resolves" color=blue]
        5 -> 1 [ label = "Field" ]
        13 -> 1 [ label = "Resolves" color=blue]
        6 -> 13 [ label = "CanResolveField(0)" color=blue]
        3 -> 14 [ label = "Field" ]
        12 -> 14 [ label = "Requires" color=green]
        3 -> 14 [ label = "Field" ]
        15 -> 14 [ label = "Resolves" color=blue]
        10 -> 15 [ label = "CanResolveField(0)" color=blue]
        5 -> 16 [ label = "Field" ]
        9 -> 16 [ label = "Requires" color=green]
        5 -> 16 [ label = "Field" ]
        17 -> 16 [ label = "Resolves" color=blue]
        6 -> 17 [ label = "CanResolveField(0)" color=blue]
    }
    "##);
}
