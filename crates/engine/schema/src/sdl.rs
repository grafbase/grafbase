use std::fmt::{self, Write};

use itertools::Itertools;
use walker::Walk as _;

use crate::{
    DeprecatedDirective, EnumDefinition, EnumValue, FieldDefinition, InputObjectDefinition, InputValueDefinition,
    InterfaceDefinition, ObjectDefinition, ScalarDefinition, Schema, TypeDefinition, UnionDefinition,
};

pub(crate) fn to_sdl(schema: &Schema) -> String {
    let mut sdl = String::with_capacity(1024);

    let _ = write_schema_definition(schema, &mut sdl);

    for definition in schema.type_definitions() {
        if definition.is_inaccessible()
            || matches!(
                definition.name(),
                "__TypeKind"
                    | "__DirectiveLocation"
                    | "__EnumValue"
                    | "__InputValue"
                    | "__Field"
                    | "__Directive"
                    | "__Type"
                    | "__Schema"
            )
            || definition.as_scalar().is_some_and(|scalar| scalar.is_builtin())
        {
            continue;
        }
        let _ = write!(&mut sdl, "{definition}\n\n");
    }

    sdl
}

fn write_schema_definition(schema: &Schema, output: &mut impl Write) -> fmt::Result {
    let query = schema.query().name();
    let mutation = schema.mutation().map(|ty| ty.name());
    let subscription = schema.subscription().map(|ty| ty.name());

    let needs_schema_block = query != "Query"
        || mutation.is_some_and(|name| name != "Mutation")
        || subscription.is_some_and(|name| name != "Subscription");

    if !needs_schema_block {
        return Ok(());
    }

    if let Some(description) = schema.graph.description_id.walk(schema) {
        let _ = write_description(output, Some(description), 0);
    }

    writeln!(output, "schema {{").unwrap();
    writeln!(output, "  query: {query}").unwrap();
    if let Some(mutation) = mutation {
        writeln!(output, "  mutation: {mutation}").unwrap();
    }
    if let Some(subscription) = subscription {
        writeln!(output, "  subscription: {subscription}").unwrap();
    }
    output.write_str("}\n\n")
}

/// Writes a number of spaces to the given writer.
pub(crate) fn write_indent(writer: &mut impl Write, indent: usize) -> fmt::Result {
    for _ in 0..indent {
        writer.write_char(' ')?;
    }

    Ok(())
}

/// Writes a GraphQL quoted string with the required escaping.
pub(crate) fn write_quoted_string(writer: &mut impl Write, value: &str) -> fmt::Result {
    writer.write_char('"')?;
    for c in value.chars() {
        match c {
            '\r' => writer.write_str("\\r")?,
            '\n' => writer.write_str("\\n")?,
            '\t' => writer.write_str("\\t")?,
            '"' => writer.write_str("\\\"")?,
            '\\' => writer.write_str("\\\\")?,
            c if c.is_control() => write!(writer, "\\u{:04}", c as u32)?,
            c => writer.write_char(c)?,
        }
    }
    writer.write_char('"')
}

pub(crate) fn write_description(writer: &mut impl Write, description: Option<&str>, indent: usize) -> fmt::Result {
    if let Some(description) = description {
        write_indent(writer, indent)?;
        write_quoted_string(writer, description)?;
        writer.write_char('\n')?;
    }

    Ok(())
}

pub(crate) fn write_deprecated(writer: &mut impl Write, directive: DeprecatedDirective<'_>) -> fmt::Result {
    writer.write_str("@deprecated")?;
    if let Some(reason) = directive.reason() {
        writer.write_str("(reason: ")?;
        write_quoted_string(writer, reason)?;
        writer.write_char(')')?;
    }

    Ok(())
}

impl fmt::Display for TypeDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeDefinition::Enum(definition) => definition.fmt(f),
            TypeDefinition::InputObject(definition) => definition.fmt(f),
            TypeDefinition::Interface(definition) => definition.fmt(f),
            TypeDefinition::Object(definition) => definition.fmt(f),
            TypeDefinition::Scalar(definition) => definition.fmt(f),
            TypeDefinition::Union(definition) => definition.fmt(f),
        }
    }
}

impl fmt::Display for ScalarDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_description(f, self.description(), 0)?;
        write!(f, "scalar {}", self.name())?;
        if let Some(url) = self.specified_by_url() {
            f.write_str(" @specifiedBy(url: ")?;
            write_quoted_string(f, url)?;
            f.write_char(')')?;
        }
        Ok(())
    }
}

impl fmt::Display for EnumDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_description(f, self.description(), 0)?;
        writeln!(f, "enum {} {{", self.name())?;

        for value in self.values().filter(|value| !value.is_inaccessible()) {
            write_enum_value(f, value, 2)?;
        }

        f.write_char('}')
    }
}

impl fmt::Display for InputObjectDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_description(f, self.description(), 0)?;
        write!(f, "input {}", self.name())?;
        if self.is_one_of {
            f.write_str(" @oneOf")?;
        }
        f.write_str(" {")?;

        let fields: Vec<_> = self.input_fields().filter(|field| !field.is_inaccessible()).collect();
        if fields.is_empty() {
            return f.write_str("}");
        }

        f.write_char('\n')?;
        for field in fields {
            write_input_value(f, field, 2)?;
        }
        f.write_char('}')
    }
}

impl fmt::Display for ObjectDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_description(f, self.description(), 0)?;
        write!(f, "type {}", self.name())?;

        let interfaces: Vec<_> = self
            .interfaces()
            .filter(|interface| !interface.is_inaccessible())
            .map(|interface| interface.name())
            .collect();
        if !interfaces.is_empty() {
            write!(f, " implements {}", interfaces.into_iter().format(" & "))?;
        }

        f.write_str(" {")?;
        let fields: Vec<_> = if self.schema.graph.root_operation_types_record.query_id == self.id {
            self.fields()
                .filter(|field| !field.is_inaccessible() && !matches!(field.name(), "__type" | "__schema"))
                .collect()
        } else {
            self.fields().filter(|field| !field.is_inaccessible()).collect()
        };
        if fields.is_empty() {
            return f.write_str("}");
        }

        f.write_char('\n')?;
        for field in fields {
            write_field(f, field, 2)?;
        }
        f.write_char('}')
    }
}

impl fmt::Display for InterfaceDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_description(f, self.description(), 0)?;
        write!(f, "interface {}", self.name())?;

        let interfaces: Vec<_> = self
            .interfaces()
            .filter(|interface| !interface.is_inaccessible())
            .map(|interface| interface.name())
            .collect();
        if !interfaces.is_empty() {
            write!(f, " implements {}", interfaces.into_iter().format(" & "))?;
        }

        f.write_str(" {")?;
        let fields: Vec<_> = self.fields().filter(|field| !field.is_inaccessible()).collect();
        if fields.is_empty() {
            return f.write_str("}");
        }

        f.write_char('\n')?;
        for field in fields {
            write_field(f, field, 2)?;
        }
        f.write_char('}')
    }
}

impl fmt::Display for UnionDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_description(f, self.description(), 0)?;

        let members: Vec<_> = self
            .possible_types()
            .filter(|object| !object.is_inaccessible())
            .map(|object| object.name())
            .collect();

        write!(f, "union {}", self.name())?;
        if !members.is_empty() {
            write!(f, " = {}", members.into_iter().format(" | "))?;
        }
        Ok(())
    }
}

fn write_field(writer: &mut impl Write, field: FieldDefinition<'_>, indent: usize) -> fmt::Result {
    write_description(writer, field.description(), indent)?;
    write_indent(writer, indent)?;

    writer.write_str(field.name())?;

    let arguments: Vec<_> = field
        .arguments()
        .filter(|argument| !argument.is_inaccessible())
        .collect();
    if !arguments.is_empty() {
        write_field_arguments(writer, &arguments, indent)?;
    }

    write!(writer, ": {}", field.ty())?;

    if let Some(deprecated) = field.has_deprecated() {
        writer.write_char(' ')?;
        write_deprecated(writer, deprecated)?;
    }

    writer.write_char('\n')
}

fn write_field_arguments(
    writer: &mut impl Write,
    arguments: &[InputValueDefinition<'_>],
    indent: usize,
) -> fmt::Result {
    let has_descriptions = arguments.iter().any(|argument| argument.description().is_some());
    if !has_descriptions {
        writer.write_char('(')?;
        for (idx, argument) in arguments.iter().enumerate() {
            if idx > 0 {
                writer.write_str(", ")?;
            }
            write!(writer, "{argument}")?;
        }
        writer.write_char(')')?;
        return Ok(());
    }

    writer.write_str("(\n")?;
    let argument_indent = indent + 2;
    for argument in arguments {
        write_description(writer, argument.description(), argument_indent)?;
        write_indent(writer, argument_indent)?;
        write!(writer, "{argument}")?;
        writer.write_char('\n')?;
    }

    write_indent(writer, indent)?;
    writer.write_char(')')
}

fn write_enum_value(writer: &mut impl Write, value: EnumValue<'_>, indent: usize) -> fmt::Result {
    write_description(writer, value.description(), indent)?;
    write_indent(writer, indent)?;
    writer.write_str(value.name())?;

    if let Some(deprecated) = value.has_deprecated() {
        writer.write_char(' ')?;
        write_deprecated(writer, deprecated)?;
    }

    writer.write_char('\n')
}

fn write_input_value(writer: &mut impl Write, value: InputValueDefinition<'_>, indent: usize) -> fmt::Result {
    write_description(writer, value.description(), indent)?;
    write_indent(writer, indent)?;
    write!(writer, "{value}")?;
    writer.write_char('\n')
}

#[cfg(test)]
mod tests {
    use crate::Schema;
    use insta::assert_snapshot;

    #[tokio::test]
    async fn to_sdl_formats_simple_schema() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            type Mutation {
              do: Boolean
            }

            type Query {
              hello(argument: String = "hi"): Custom
            }

            scalar Custom
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r###"
        scalar Custom

        type Mutation {
          do: Boolean
        }

        type Query {
          hello(argument: String = "hi"): Custom
        }
        "###
        );
    }

    #[tokio::test]
    async fn to_sdl_formats_argument_descriptions() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            type Query {
              user(
                """User id"""
                id: ID!
                """Whether to request the short profile"""
                short: Boolean
              ): String
            }
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r###"
        type Query {
          user(
            "User id"
            id: ID!
            "Whether to request the short profile"
            short: Boolean
          ): String
        }
        "###
        );
    }

    #[tokio::test]
    async fn to_sdl_emits_schema_block_for_custom_roots() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            schema {
              query: RootQuery
            }

            type RootQuery {
              ping: String
            }
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r###"
        schema {
          query: RootQuery
        }

        type RootQuery {
          ping: String
        }
        "###
        );
    }

    #[tokio::test]
    async fn to_sdl_skips_inaccessible_definitions_and_fields() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            directive @inaccessible on FIELD_DEFINITION | OBJECT | ENUM | ENUM_VALUE | INPUT_OBJECT | INPUT_FIELD_DEFINITION | UNION | SCALAR | INTERFACE

            type Query {
              visible: Visible
              hidden: Hidden @inaccessible
            }

            type Visible {
              public: String
              secret: String @inaccessible
            }

            type Hidden @inaccessible {
              id: ID!
            }

            enum Status @inaccessible {
              ACTIVE
            }
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r###"
        type Query {
          visible: Visible
        }

        type Visible {
          public: String
        }
        "###
        );
    }

    #[tokio::test]
    async fn to_sdl_formats_interfaces() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            type Query {
              nodes: [Node!]!
            }

            interface Node {
              id: ID!
            }

            interface Timestamped {
              createdAt: String!
              updatedAt: String
            }

            type User implements Node & Timestamped {
              id: ID!
              name: String!
              createdAt: String!
              updatedAt: String
            }
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r"
        interface Node {
          id: ID!
        }

        type Query {
          nodes: [Node!]!
        }

        interface Timestamped {
          createdAt: String!
          updatedAt: String
        }

        type User implements Node & Timestamped {
          id: ID!
          name: String!
          createdAt: String!
          updatedAt: String
        }
        "
        );
    }

    #[tokio::test]
    async fn to_sdl_formats_interfaces_implementing_interfaces() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            type Query {
              resources: [Resource!]!
            }

            interface Node {
              id: ID!
            }

            interface Resource implements Node {
              id: ID!
              name: String!
            }

            type Document implements Resource & Node {
              id: ID!
              name: String!
              content: String!
            }
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r"
        type Document implements Resource & Node {
          id: ID!
          name: String!
          content: String!
        }

        interface Node {
          id: ID!
        }

        type Query {
          resources: [Resource!]!
        }

        interface Resource implements Node {
          id: ID!
          name: String!
        }
        "
        );
    }

    #[tokio::test]
    async fn to_sdl_formats_unions() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            type Query {
              search(term: String!): [SearchResult!]!
            }

            union SearchResult = User | Post | Comment

            type User {
              name: String!
            }

            type Post {
              title: String!
              body: String!
            }

            type Comment {
              text: String!
            }
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r"
        type Comment {
          text: String!
        }

        type Post {
          title: String!
          body: String!
        }

        type Query {
          search(term: String!): [SearchResult!]!
        }

        union SearchResult = User | Post | Comment

        type User {
          name: String!
        }
        "
        );
    }

    #[tokio::test]
    async fn to_sdl_formats_enums_with_deprecated_values() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            type Query {
              status: Status
            }

            """Status of a resource"""
            enum Status {
              """The resource is active"""
              ACTIVE
              PENDING @deprecated
              INACTIVE @deprecated(reason: "Use ARCHIVED instead")
              ARCHIVED
            }
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r#"
        type Query {
          status: Status
        }

        "Status of a resource"
        enum Status {
          "The resource is active"
          ACTIVE
          PENDING @deprecated
          INACTIVE @deprecated(reason: "Use ARCHIVED instead")
          ARCHIVED
        }
        "#
        );
    }

    #[tokio::test]
    async fn to_sdl_formats_deprecated_fields() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            type Query {
              user(id: ID!): User
            }

            type User {
              id: ID!
              name: String!
              username: String! @deprecated
              email: String @deprecated(reason: "Use contactEmail instead")
              contactEmail: String
            }
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r#"
        type Query {
          user(id: ID!): User
        }

        type User {
          id: ID!
          name: String!
          username: String! @deprecated
          email: String @deprecated(reason: "Use contactEmail instead")
          contactEmail: String
        }
        "#
        );
    }

    #[tokio::test]
    async fn to_sdl_formats_input_objects() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            type Query {
              createUser(input: CreateUserInput!): User
            }

            type User {
              id: ID!
            }

            """Input for creating a user"""
            input CreateUserInput {
              """The user's name"""
              name: String!
              email: String!
              age: Int = 18
            }
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r#"
        "Input for creating a user"
        input CreateUserInput {
          "The user's name"
          name: String!
          email: String!
          age: Int = 18
        }

        type Query {
          createUser(input: CreateUserInput!): User
        }

        type User {
          id: ID!
        }
        "#
        );
    }

    #[tokio::test]
    async fn to_sdl_formats_input_objects_with_oneof() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            type Query {
              search(by: SearchBy!): [Result!]!
            }

            type Result {
              id: ID!
            }

            input SearchBy @oneOf {
              id: ID
              name: String
              email: String
            }
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r"
        type Query {
          search(by: SearchBy!): [Result!]!
        }

        type Result {
          id: ID!
        }

        input SearchBy @oneOf {
          id: ID
          name: String
          email: String
        }
        "
        );
    }

    #[tokio::test]
    async fn to_sdl_formats_scalar_with_specified_by() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            type Query {
              time: DateTime
            }

            """An RFC 3339 compliant date-time scalar"""
            scalar DateTime @specifiedBy(url: "https://datatracker.ietf.org/doc/html/rfc3339")
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r#"
        "An RFC 3339 compliant date-time scalar"
        scalar DateTime

        type Query {
          time: DateTime
        }
        "#
        );
    }

    #[tokio::test]
    async fn to_sdl_formats_descriptions_with_special_characters() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            type Query {
              """
              Search with special chars: "quotes", \backslash, and
              newlines
              """
              search(
                """Use format: "term"	with tabs"""
                term: String!
              ): String
            }
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r#"
        type Query {
          "Search with special chars: \"quotes\", \\backslash, and\nnewlines"
          search(
            "Use format: \"term\"\twith tabs"
            term: String!
          ): String
        }
        "#
        );
    }

    #[tokio::test]
    async fn to_sdl_handles_empty_types() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            directive @inaccessible on FIELD_DEFINITION

            type Query {
              empty: EmptyType
            }

            type EmptyType {
              hidden: String @inaccessible
            }

            enum EmptyEnum {
              VALUE @inaccessible
            }

            input EmptyInput {
              field: String @inaccessible
            }

            union EmptyUnion
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @"type Query {}"
        );
    }

    #[tokio::test]
    async fn to_sdl_formats_complex_nested_schema() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            type Query {
              """Get a user by their unique identifier"""
              user(
                """The unique user ID"""
                id: ID!
                """Include deleted users in the search"""
                includeDeleted: Boolean = false
              ): User
            }

            type Mutation {
              """Update a user's profile"""
              updateUser(input: UpdateUserInput!): User
            }

            """A user in the system"""
            type User implements Node & Timestamped {
              id: ID!
              """The user's display name"""
              name: String!
              email: String @deprecated(reason: "Use emails field instead")
              emails: [String!]!
              role: Role!
              posts: [Post!]!
              createdAt: String!
              updatedAt: String
            }

            """Base interface for all nodes"""
            interface Node {
              id: ID!
            }

            """Interface for timestamped entities"""
            interface Timestamped {
              createdAt: String!
              updatedAt: String
            }

            type Post implements Node {
              id: ID!
              title: String!
              content: String!
              author: User!
            }

            """User role in the system"""
            enum Role {
              ADMIN
              USER
              GUEST @deprecated
            }

            input UpdateUserInput {
              name: String
              emails: [String!]
            }
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r#"
        type Mutation {
          "Update a user's profile"
          updateUser(input: UpdateUserInput!): User
        }

        "Base interface for all nodes"
        interface Node {
          id: ID!
        }

        type Post implements Node {
          id: ID!
          title: String!
          content: String!
          author: User!
        }

        type Query {
          "Get a user by their unique identifier"
          user(
            "The unique user ID"
            id: ID!
            "Include deleted users in the search"
            includeDeleted: Boolean = false
          ): User
        }

        "User role in the system"
        enum Role {
          ADMIN
          USER
          GUEST @deprecated
        }

        "Interface for timestamped entities"
        interface Timestamped {
          createdAt: String!
          updatedAt: String
        }

        input UpdateUserInput {
          name: String
          emails: [String!]
        }

        "A user in the system"
        type User implements Node & Timestamped {
          id: ID!
          "The user's display name"
          name: String!
          email: String @deprecated(reason: "Use emails field instead")
          emails: [String!]!
          role: Role!
          posts: [Post!]!
          createdAt: String!
          updatedAt: String
        }
        "#
        );
    }

    #[tokio::test]
    async fn to_sdl_skips_introspection_fields_from_query() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            type Query {
              user(id: ID!): User
            }

            type User {
              id: ID!
              name: String!
            }
            "#,
        )
        .await;

        let sdl = schema.to_sdl();
        assert!(!sdl.contains("__type"));
        assert!(!sdl.contains("__schema"));
        assert_snapshot!(sdl, @r"
        type Query {
          user(id: ID!): User
        }

        type User {
          id: ID!
          name: String!
        }
        ");
    }

    #[tokio::test]
    async fn to_sdl_filters_inaccessible_enum_values() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            directive @inaccessible on ENUM_VALUE

            type Query {
              status: Status
            }

            enum Status {
              ACTIVE
              INTERNAL @inaccessible
              PENDING
            }
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r"
        type Query {
          status: Status
        }

        enum Status {
          ACTIVE
          PENDING
        }
        "
        );
    }

    #[tokio::test]
    async fn to_sdl_filters_inaccessible_union_members() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            directive @inaccessible on OBJECT

            type Query {
              search: SearchResult
            }

            union SearchResult = PublicDoc | InternalDoc

            type PublicDoc {
              title: String!
            }

            type InternalDoc @inaccessible {
              secret: String!
            }
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r"
        type PublicDoc {
          title: String!
        }

        type Query {
          search: SearchResult
        }

        union SearchResult = PublicDoc
        "
        );
    }

    #[tokio::test]
    async fn to_sdl_filters_inaccessible_interface_implementations() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            directive @inaccessible on INTERFACE

            type Query {
              nodes: [Node!]!
            }

            interface Node {
              id: ID!
            }

            interface Internal @inaccessible {
              secret: String!
            }

            type Document implements Node & Internal {
              id: ID!
              secret: String!
            }
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r"
        type Document implements Node {
          id: ID!
          secret: String!
        }

        interface Node {
          id: ID!
        }

        type Query {
          nodes: [Node!]!
        }
        "
        );
    }

    #[tokio::test]
    async fn to_sdl_formats_field_arguments_without_descriptions() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            type Query {
              users(first: Int = 10, after: String, filter: UserFilter): [User!]!
            }

            type User {
              id: ID!
            }

            input UserFilter {
              name: String
            }
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r"
        type Query {
          users(first: Int = 10, after: String, filter: UserFilter): [User!]!
        }

        type User {
          id: ID!
        }

        input UserFilter {
          name: String
        }
        "
        );
    }

    #[tokio::test]
    async fn to_sdl_handles_subscription_root() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            schema {
              query: Query
              mutation: Mutation
              subscription: Subscription
            }

            type Query {
              user: User
            }

            type Mutation {
              createUser: User
            }

            type Subscription {
              userCreated: User
            }

            type User {
              id: ID!
            }
            "#,
        )
        .await;

        assert_snapshot!(
            schema.to_sdl(),
            @r"
        type Mutation {
          createUser: User
        }

        type Query {
          user: User
        }

        type Subscription {
          userCreated: User
        }

        type User {
          id: ID!
        }
        "
        );
    }

    #[tokio::test]
    async fn to_sdl_formats_federated_supergraph() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            directive @join__graph(name: String!, url: String!) on ENUM_VALUE

            directive @join__type(
              graph: join__Graph!
              key: join__FieldSet
              extension: Boolean! = false
              resolvable: Boolean! = true
              isInterfaceObject: Boolean! = false
            ) repeatable on OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT | SCALAR

            directive @join__field(
              graph: join__Graph
              requires: join__FieldSet
              provides: join__FieldSet
              type: String
              external: Boolean
              override: String
              overrideLabel: String
              usedOverridden: Boolean
            ) repeatable on FIELD_DEFINITION | INPUT_FIELD_DEFINITION

            scalar join__FieldSet

            enum join__Graph {
              PRODUCTS @join__graph(name: "products", url: "https://products.invalid/graphql")
              REVIEWS @join__graph(name: "reviews", url: "https://reviews.invalid/graphql")
            }

            type Query
              @join__type(graph: PRODUCTS)
              @join__type(graph: REVIEWS)
            {
              topProducts(first: Int = 5): [Product] @join__field(graph: PRODUCTS)
              product(upc: ID!): Product @join__field(graph: PRODUCTS)
            }

            type Product
              @join__type(graph: PRODUCTS, key: "upc")
              @join__type(graph: REVIEWS, key: "upc")
            {
              upc: ID!
              name: String @join__field(graph: PRODUCTS)
              price: Int @join__field(graph: PRODUCTS)
              inStock: Boolean @join__field(graph: PRODUCTS)
              reviews: [Review] @join__field(graph: REVIEWS)
            }

            type Review
              @join__type(graph: REVIEWS, key: "id")
            {
              id: ID!
              body: String @join__field(graph: REVIEWS)
              product: Product @join__field(graph: REVIEWS)
            }
            "#,
        )
        .await;

        let sdl = schema.to_sdl();
        assert!(
            !sdl.contains("join__"),
            "API schema should not expose federation metadata"
        );
        assert_snapshot!(
            sdl,
            @r###"
        type Product {
          upc: ID!
          name: String
          price: Int
          inStock: Boolean
          reviews: [Review]
        }

        type Query {
          topProducts(first: Int = 5): [Product]
          product(upc: ID!): Product
        }

        type Review {
          id: ID!
          body: String
          product: Product
        }
        "###
        );
    }

    #[tokio::test]
    async fn to_sdl_filters_inaccessible_fields_in_federated_schema() {
        let schema = Schema::from_sdl_or_panic(
            r#"
            directive @join__graph(name: String!, url: String!) on ENUM_VALUE

            directive @join__type(
              graph: join__Graph!
              key: join__FieldSet
              extension: Boolean! = false
              resolvable: Boolean! = true
              isInterfaceObject: Boolean! = false
            ) repeatable on OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT | SCALAR

            directive @join__field(
              graph: join__Graph
              requires: join__FieldSet
              provides: join__FieldSet
              type: String
              external: Boolean
              override: String
              overrideLabel: String
              usedOverridden: Boolean
            ) repeatable on FIELD_DEFINITION | INPUT_FIELD_DEFINITION

            scalar join__FieldSet

            enum join__Graph {
              CATALOG @join__graph(name: "catalog", url: "https://catalog.invalid/graphql")
            }

            type Query
              @join__type(graph: CATALOG)
            {
              product(id: ID!): Product @join__field(graph: CATALOG)
              adminProduct(id: ID!): Product @join__field(graph: CATALOG) @inaccessible
            }

            type Product
              @join__type(graph: CATALOG, key: "id")
            {
              id: ID!
              name: String @join__field(graph: CATALOG)
              secretNotes: String @join__field(graph: CATALOG) @inaccessible
            }
            "#,
        )
        .await;

        let sdl = schema.to_sdl();
        assert!(!sdl.contains("join__"));
        assert!(!sdl.contains("@inaccessible"));
        assert_snapshot!(
            sdl,
            @r###"
        type Product {
          id: ID!
          name: String
        }

        type Query {
          product(id: ID!): Product
        }
        "###
        );
    }
}
