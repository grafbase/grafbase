use crate::diagnostics::Diagnostics;
use async_graphql_parser::{types as ast, Pos, Positioned};
use async_graphql_value::Name;
use std::collections::HashMap;

type AstField = Positioned<ast::FieldDefinition>;

pub(crate) struct Context<'a> {
    /// The source document.
    pub(crate) sdl: &'a str,

    /// Definition name -> definition AST node
    pub(crate) definition_names: HashMap<&'a str, &'a Positioned<ast::TypeDefinition>>,

    /// Directive name -> directive AST node
    pub(crate) directive_names: HashMap<&'a str, &'a Positioned<ast::DirectiveDefinition>>,

    /// Validation errors and warnings. See [Diagnostics].
    pub(crate) diagnostics: Diagnostics,

    pub(crate) options: crate::Options,

    /// The schema definition that was encountered, if any.
    ///
    /// Example schema definition:
    ///
    /// ```graphql
    /// schema {
    ///   query: Query
    /// }
    /// ```
    pub(crate) schema_definition: Option<SchemaDefinition<'a>>,

    // Definition name, extended fields. Only populated in the presence of extensions.
    pub(crate) extended_fields: HashMap<&'a str, Vec<&'a [AstField]>>,

    // Union name, extended members. Only populated in the presence of extensions.
    pub(crate) extended_unions: HashMap<&'a str, Vec<&'a [Positioned<Name>]>>,

    // Implementer extensions -> interfaces
    pub(crate) extended_interface_implementations: HashMap<&'a str, Vec<&'a Positioned<Name>>>,

    // Enum name, extended members. Only populated in the presence of extensions.
    pub(crate) extended_enums: HashMap<&'a str, Vec<&'a [Positioned<ast::EnumValueDefinition>]>>,

    // Reusable buffer for duplicate name detection.
    strings_buf: HashMap<&'a str, usize>,
}

impl<'a> Context<'a> {
    pub(crate) fn new(
        sdl: &'a str,
        definition_names: HashMap<&'a str, &'a Positioned<ast::TypeDefinition>>,
        diagnostics: Diagnostics,
        options: crate::Options,
    ) -> Self {
        Context {
            sdl,
            definition_names,
            diagnostics,
            options,
            schema_definition: None,

            strings_buf: HashMap::default(),
            directive_names: HashMap::default(),
            extended_interface_implementations: HashMap::default(),
            extended_fields: HashMap::default(),
            extended_unions: HashMap::default(),
            extended_enums: HashMap::default(),
        }
    }

    /// With an enum name and the enum declaration values, iterate all these values _and the values
    /// defined in enum extensions_ together.
    pub(crate) fn with_enum_values<F>(
        &mut self,
        enum_name: &str,
        base_values: &'a [Positioned<ast::EnumValueDefinition>],
        mut handler: F,
    ) where
        F: FnMut(&mut Self, &[&'a Positioned<ast::EnumValueDefinition>]),
    {
        let all_values: Vec<_> = base_values
            .iter()
            .chain(
                self.extended_enums
                    .get(enum_name)
                    .into_iter()
                    .flat_map(|vecs| vecs.iter())
                    .flat_map(|values| values.iter()),
            )
            .collect();
        handler(self, &all_values);
    }

    /// With an union name and the union declaration members, iterate all these members _and the
    /// members defined in any extensions for this union_ together.
    pub(crate) fn with_union_members<F>(
        &mut self,
        union_name: &str,
        base_values: &'a [Positioned<Name>],
        mut handler: F,
    ) where
        F: FnMut(&mut Self, &[&'a Positioned<Name>]),
    {
        let all_values: Vec<_> = base_values
            .iter()
            .chain(
                self.extended_unions
                    .get(union_name)
                    .into_iter()
                    .flat_map(|vecs| vecs.iter())
                    .flat_map(|values| values.iter()),
            )
            .collect();
        handler(self, &all_values);
    }

    /// With an object/interface name and the object/interface declaration fields, iterate all
    /// these fields _and the fields defined in any extensions for this object/interface_ together.
    pub(crate) fn with_fields<F>(&mut self, name: &str, base_fields: &'a [AstField], mut handler: F)
    where
        F: FnMut(&mut Self, &[&'a AstField]),
    {
        let all_fields: Vec<_> = base_fields
            .iter()
            .chain(
                self.extended_fields
                    .get(name)
                    .into_iter()
                    .flat_map(|fields| fields.iter())
                    .flat_map(|f| f.iter()),
            )
            .collect();
        handler(self, &all_fields);
    }

    /// With an object/interface name and the object/interface `implements` list, iterate all these
    /// implemented interfaces _and the `implements` defined in any extensions for this
    /// object/interface_ together.
    pub(crate) fn with_implements(
        &mut self,
        type_name: &str,
        base_implements: &'a [Positioned<Name>],
        mut handler: impl FnMut(&mut Self, &[&'a Positioned<Name>]),
    ) {
        let extended = self
            .extended_interface_implementations
            .get(type_name)
            .into_iter()
            .flatten()
            .copied();
        let implements: Vec<_> = base_implements.iter().chain(extended).collect();
        handler(self, &implements);
    }

    pub(crate) fn miette_pos(&self, pos: async_graphql_parser::Pos) -> miette::SourceOffset {
        miette::SourceOffset::from_location(self.sdl, pos.line, pos.column)
    }

    pub(crate) fn push_error(&mut self, err: miette::Report) {
        self.diagnostics.errors.push(err.with_source_code(self.sdl.to_owned()));
    }

    /// The handler is called once for each duplicate name with (first occurence index, second
    /// occurence index).
    pub(crate) fn find_duplicates<F>(&mut self, names: impl Iterator<Item = &'a str>, mut handle_duplicates: F)
    where
        F: FnMut(&mut Self, usize, usize),
    {
        self.strings_buf.clear();

        for (idx, name) in names.enumerate() {
            if let Some(previous) = self.strings_buf.insert(name, idx) {
                handle_duplicates(self, previous, idx);
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct SchemaDefinition<'a> {
    pub(crate) pos: Pos,
    pub(crate) query: &'a str,
    pub(crate) mutation: Option<&'a str>,
    pub(crate) subscription: Option<&'a str>,
}
