#![allow(unused_crate_dependencies)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use graphql_lint::lint;

fn criterion_benchmark(criterion: &mut Criterion) {
    let schema = r#"
        directive @WithDeprecatedArgs(
          ARG: String @deprecated(reason: "Use `newArg`")
          newArg: String
        ) on FIELD

        enum Enum_lowercase @deprecated {
          an_enum_member @deprecated
        }

        enum lowercase_Enum {
          an_enum_member @deprecated
        }
        
        type Query {
          __test: String,
          getHello(name: String!): Enum_lowercase!
          queryHello(name: String!): Enum_lowercase!
          listHello(name: String!): Enum_lowercase!
          helloQuery(name: String!): Enum_lowercase!
        }

        type Mutation {
          __test: String,
          putHello(name: String!): Enum_lowercase!
          mutationHello(name: String!): Enum_lowercase!
          postHello(name: String!): Enum_lowercase!
          patchHello(name: String!): Enum_lowercase!
          helloMutation(name: String!): Enum_lowercase!
        }

        type Subscription {
          __test: String,
          subscriptionHello(name: String!): Enum_lowercase!
          helloSubscription(name: String!): Enum_lowercase!
        }

        type TypeTest {
          name: String @deprecated
        }

        type TestType {
           name: string
        }

        type other {
           name: string
        }

        scalar CustomScalar @specifiedBy(url: "https://specs.example.com/rfc1") @deprecated

        union UnionTest @deprecated = testType | typeTest

        union TestUnion = testType | typeTest

        interface GameInterface {
          title: String!
          publisher: String! @deprecated
        }

        interface InterfaceGame @deprecated {
          title: String!
          publisher: String!
        }

        input TEST @deprecated {
          OTHER: String @deprecated
        }

        type hello @deprecated {
          Test(NAME: String): String
        }

        extend type hello {
          GOODBYE: String
        }

        schema {
          query: Query
          mutation: Mutation
        }
    "#;

    criterion.bench_function("lint schema", |bencher| bencher.iter(|| lint(black_box(schema))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
