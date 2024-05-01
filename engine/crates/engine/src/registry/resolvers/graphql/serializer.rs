use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Write},
    ops::Deref,
};

use engine_parser::{
    types::{
        Directive, Field, FragmentDefinition, FragmentSpread, InlineFragment, Selection, TypeCondition,
        VariableDefinition,
    },
    Positioned,
};
use engine_value::{Name, Value};
use meta_type_name::MetaTypeName;
use registry_v2::{MetaField, Registry};
use type_names::WrappingType;

use super::Target;
use crate::registry::{type_kinds::SelectionSetTarget, type_names, RegistryV2Ext};

/// Serialize a list of [`Selection`]s into a GraphQL query string.
///
/// The serializer is specifically tailored for the [`graphql::Resolver`](super::Resolver), as it
/// has logic to prepend/remove namespaced prefixes to global types, and injects `__typename`
/// fields into queries that need it for the resolver to properly parse the returned data.
pub struct Serializer<'a, 'b> {
    /// The prefix string to strip from any global type, before serializing the query.
    prefix: Option<&'a str>,

    /// Buffer used to write operation string to.
    buf: &'a mut String,

    /// Global list of fragment definitions, to allow the serializer to embed the definitions of
    /// any fragments used within the query.
    fragment_definitions: HashMap<&'b Name, &'b FragmentDefinition>,

    /// Internal tracking of all fragment spreads used within the execution document.
    /// These are linked to the known `fragment_definitions` to embed the required fragment
    /// definitions in the document.
    fragment_spreads: HashSet<Name>,
    serialized_fragments: HashSet<Name>,

    /// Internal tracking of indentation to pretty-print query.
    indent: usize,

    /// A list of serialized variable references.
    ///
    /// This allows the caller to pass along the relevant variable values to the upsteam server.
    variable_references: HashSet<Name>,

    /// Variable definitions from the original query
    ///
    /// These allow us to define any variables we need to use in the upstream query
    variable_definitions: HashMap<&'b Name, &'b VariableDefinition>,

    registry: Option<&'b Registry>,
}

impl<'a, 'b> Serializer<'a, 'b> {
    pub fn new(
        prefix: Option<&'a str>,
        fragment_definitions: HashMap<&'b Name, &'b FragmentDefinition>,
        variable_definitions: HashMap<&'b Name, &'b VariableDefinition>,
        buf: &'a mut String,
        registry: Option<&'b Registry>,
    ) -> Self {
        Serializer {
            prefix,
            buf,
            fragment_definitions,
            fragment_spreads: HashSet::new(),
            serialized_fragments: HashSet::new(),
            indent: 0,
            variable_references: HashSet::new(),
            variable_definitions,
            registry,
        }
    }

    /// Get an iterator over variable references the serializer has serialized.
    ///
    /// This list will be empty, until [`Serializer::query()`] or [`Serializer::mutation()`] is
    /// called.
    pub fn variable_references(&self) -> impl Iterator<Item = &Name> {
        self.variable_references.iter()
    }
}

impl<'a: 'b, 'b: 'a, 'c: 'a> Serializer<'a, 'b> {
    /// Serialize query.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the buffer fails.
    pub fn query(&mut self, target: Target<'a>, current_type: Option<SelectionSetTarget<'a>>) -> Result<(), Error> {
        match target {
            Target::SelectionSet(selections) => {
                self.serialize_selections(selections, current_type)?;
            }
            Target::Field(field, metafield) => {
                self.open_object()?;
                self.serialize_field(&field, Some(metafield))?;
                self.close_object()?;
            }
        }

        self.serialize_fragment_definitions(current_type.is_some())?;

        self.prepend_declaration("query")
    }

    /// Serialize mutation.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the buffer fails.
    pub fn mutation(&mut self, target: Target<'a>, current_type: Option<SelectionSetTarget<'a>>) -> Result<(), Error> {
        match target {
            Target::SelectionSet(selections) => {
                self.serialize_selections(selections, current_type)?;
            }
            Target::Field(field, schema_field) => {
                self.open_object()?;
                self.serialize_field(&field, Some(schema_field))?;
                self.close_object()?;
            }
        }

        self.serialize_fragment_definitions(current_type.is_some())?;

        self.prepend_declaration("mutation")
    }

    fn serialize_selection(
        &mut self,
        selection: Selection,
        current_type: Option<SelectionSetTarget<'a>>,
    ) -> Result<(), Error> {
        use Selection::{Field, FragmentSpread, InlineFragment};

        match selection {
            Field(Positioned { node: field, .. }) => {
                let schema_field = current_type
                    .map(|current_type| {
                        current_type
                            .field(field.name.as_str())
                            .ok_or_else(|| Error::UnknownField(field.name.to_string(), current_type.name().to_string()))
                    })
                    .transpose()?;

                self.serialize_field(&field, schema_field)
            }
            FragmentSpread(Positioned { node, .. }) => self.serialize_fragment_spread(&node),
            InlineFragment(Positioned { node, .. }) => self.serialize_inline_fragment(&node, current_type),
        }
    }

    fn serialize_field(&mut self, field: &Field, schema_field: Option<MetaField<'a>>) -> Result<(), Error> {
        if let Some(schema_field) = schema_field {
            if schema_field.resolver().is_custom() || schema_field.resolver().is_join() {
                // Skip fields that have resolvers or are joins - they won't exist in the downstream
                // server
                return Ok(());
            }
        }

        self.indent()?;

        // Alias
        //
        // <https://graphql.org/learn/queries/#aliases>
        if let Some(Positioned { node, .. }) = &field.alias {
            self.write_str(node)?;
            self.write_str(": ")?;
        }

        // Field name
        self.write_str(field.name.as_str())?;

        // Arguments
        self.serialize_arguments(&field.arguments)?;

        // Directives
        {
            let directives = field.directives.iter().map(|v| v.node.clone());
            self.serialize_directives(directives)?;
        }

        // Selection Sets
        if !field.selection_set.items.is_empty() {
            let selections = field.selection_set.deref().items.iter().map(|v| v.node.clone());
            let field_type = schema_field
                .map(|field| SelectionSetTarget::try_from(field.ty().named_type()))
                .transpose()?;

            self.serialize_selections(selections, field_type)?;
        }

        self.write_str("\n")
    }

    /// Arguments
    ///
    /// <https://graphql.org/learn/queries/#arguments>
    fn serialize_arguments(&mut self, arguments: &[(Positioned<Name>, Positioned<Value>)]) -> Result<(), Error> {
        if arguments.is_empty() {
            return Ok(());
        }

        self.write_str("(")?;

        let mut arguments = arguments
            .iter()
            .map(|(k, v)| (k.node.clone(), v.node.clone()))
            .peekable();

        while let Some((name, value)) = arguments.next() {
            // If the argument references variables, we track them so that the caller knows which
            // variable values are needed to execute the document.
            self.variable_references.extend(value.variables_used().cloned());

            self.write_str(name)?;
            self.write_str(": ")?;
            self.write_str(value.to_string())?;

            if arguments.peek().is_some() {
                self.write_str(", ")?;
            }
        }

        self.write_str(")")
    }

    /// Selection Sets
    ///
    /// <https://spec.graphql.org/June2018/#sec-Selection-Sets>
    fn serialize_selections(
        &mut self,
        selections: impl Iterator<Item = Selection>,
        current_type: Option<SelectionSetTarget<'a>>,
    ) -> Result<(), Error> {
        let mut selections = selections.peekable();

        if selections.peek().is_none() {
            return Ok(());
        }

        self.open_object()?;

        for selection in selections {
            self.serialize_selection(selection, current_type)?;
        }

        self.close_object()
    }

    fn serialize_directives(&mut self, directives: impl Iterator<Item = Directive>) -> Result<(), Error> {
        for directive in directives {
            if !should_forward_directive(directive.name.as_str()) {
                continue;
            }

            self.write_str(" @")?;
            self.write_str(directive.name.as_str())?;
            self.serialize_arguments(&directive.arguments)?;
        }

        Ok(())
    }

    /// Fragment Spread
    ///
    /// <https://spec.graphql.org/June2018/#FragmentSpread>
    fn serialize_fragment_spread(&mut self, fragment: &FragmentSpread) -> Result<(), Error> {
        let fragment_name = fragment.fragment_name.clone();

        self.indent()?;
        self.write_str("... ")?;
        self.write_str(fragment_name.as_str())?;

        self.fragment_spreads.insert(fragment_name.clone().into_inner());

        let directives = fragment.directives.iter().map(|v| v.node.clone());
        self.serialize_directives(directives)?;
        self.write_str("\n")
    }

    /// Inline Fragment
    ///
    /// <https://spec.graphql.org/June2018/#sec-Inline-Fragments>
    fn serialize_inline_fragment(
        &mut self,
        fragment: &InlineFragment,
        current_type: Option<SelectionSetTarget<'a>>,
    ) -> Result<(), Error> {
        let type_condition = fragment.type_condition.as_ref().map(|v| v.node.clone());
        let directives = fragment.directives.iter().map(|v| v.node.clone());
        let selections = fragment.selection_set.deref().items.iter().map(|v| v.node.clone());

        self.indent()?;
        self.write_str("...")?;

        self.serialize_fragment_inner(type_condition, directives, selections, current_type)
    }

    fn serialize_fragment_definitions(&mut self, check_current_type: bool) -> Result<(), Error> {
        while !self.fragment_spreads.is_empty() {
            for name in std::mem::take(&mut self.fragment_spreads) {
                if self.serialized_fragments.contains(&name) {
                    continue;
                }
                // If a spread references an unknown definition, the query will fail, but the failure
                // will be reported by the GraphQL resolver, not this serializer.
                if let Some(definition) = self.fragment_definitions.get(&name) {
                    self.serialize_fragment_definition(name, definition, check_current_type)?;
                }
            }
        }

        Ok(())
    }

    fn serialize_fragment_definition(
        &mut self,
        name: Name,
        definition: &'c FragmentDefinition,
        check_current_type: bool,
    ) -> Result<(), Error> {
        self.serialized_fragments.insert(name.clone());
        self.write_str("fragment ")?;
        self.write_str(name)?;

        let type_condition = definition.type_condition.node.clone();
        let directives = definition.directives.iter().map(|v| v.node.clone());
        let selections = definition.selection_set.deref().items.iter().map(|v| v.node.clone());

        let current_type = check_current_type
            .then(|| {
                self.registry
                    .map(|registry| registry.lookup(&type_names::TypeCondition::from(type_condition.on.node.as_str())))
            })
            .flatten()
            .transpose()?;

        self.serialize_fragment_inner(Some(type_condition), directives, selections, current_type)
    }

    fn serialize_fragment_inner(
        &mut self,
        type_condition: Option<TypeCondition>,
        directives: impl Iterator<Item = Directive>,
        selections: impl Iterator<Item = Selection>,
        current_type: Option<SelectionSetTarget<'a>>,
    ) -> Result<(), Error> {
        let mut target_type = current_type;
        if let Some(condition) = type_condition {
            self.write_str(" on ")?;

            if current_type.is_some() {
                target_type = self
                    .registry
                    .map(|registry| registry.lookup(&type_names::TypeCondition::from(condition.on.as_str())))
                    .transpose()?;
            }

            self.write_str(self.remove_prefix_from_type(condition.on.as_str()))?;
        }

        // So the new type is _either_ the TypeCondition or whatever the current type is, so we need to pass that in.

        self.serialize_directives(directives)?;
        self.serialize_selections(selections, target_type)
    }

    /// This function handles prepending the variable declarations to our buffer.
    ///
    /// We need to output variable definitions at the start of the buffer, but we
    /// don't know what variables we need till we've serialized everything else.
    ///
    /// This is not exactly an optimal solution, but the alternative was traversing
    /// the entire query looking for variables before we output anything and I
    /// didn't want to write that much code today, so :sigh: this'll do.
    fn prepend_declaration(&mut self, query_kind_str: &str) -> Result<(), Error> {
        // We can't just write directly into buffer in this function because
        // it's on self and we need to make immutable borrows from self.
        let mut declaration = query_kind_str.to_string();

        if !self.variable_references.is_empty() {
            write!(declaration, "(")?;

            let mut iter = self.variable_references().peekable();

            while let Some(variable_name) = iter.next() {
                let Some(variable_definition) = &self.variable_definitions.get(variable_name) else {
                    return Err(Error::UndeclaredVariable(variable_name.to_string()));
                };

                let VariableDefinition {
                    name,
                    var_type,
                    directives,
                    default_value,
                } = variable_definition;

                let var_type = var_type.to_string();
                let var_type = self.remove_prefix_from_type(&var_type);

                write!(declaration, "${name}: {var_type}")?;

                if let Some(default_value) = default_value {
                    write!(declaration, " = {default_value}")?;
                }
                for directive in directives {
                    let Directive { name, arguments } = &directive.node;
                    write!(declaration, "@{name}")?;
                    if !arguments.is_empty() {
                        write!(declaration, "(")?;
                        for (name, value) in arguments {
                            write!(declaration, "{name} = {value}, ")?;
                        }
                        write!(declaration, ")")?;
                    }
                }
                if iter.peek().is_some() {
                    write!(declaration, ", ")?;
                }
            }
            write!(declaration, ")")?;
        }

        declaration.push_str(self.buf);
        *self.buf = declaration;

        Ok(())
    }

    fn indent(&mut self) -> Result<(), Error> {
        self.buf.write_str(&"\t".repeat(self.indent))?;
        Ok(())
    }

    fn writeln_str(&mut self, s: impl AsRef<str>) -> Result<(), Error> {
        self.indent()?;
        self.write_str(s)
    }

    fn write_str(&mut self, s: impl AsRef<str>) -> Result<(), Error> {
        self.buf.write_str(s.as_ref())?;
        Ok(())
    }

    fn open_object(&mut self) -> Result<(), Error> {
        self.write_str(" {\n")?;
        self.indent += 1;

        // We always inject `__typename` into every selection set (except for the root). This is
        // needed in specific cases for Grafbase to correctly link responses back to known types.
        //
        // While we technically don't need to embed the field in _every_ selection set for Grafbase
        // to function properly, it's simpler to do so, and follows precedence set by clients such
        // as Apollo[1].
        //
        // [1]: https://www.apollographql.com/docs/ios/fetching/type-conditions/#type-conversion
        if self.indent > 1 {
            self.indent()?;
            self.write_str("__typename\n")?;
        }

        Ok(())
    }

    fn close_object(&mut self) -> Result<(), Error> {
        // Clean-up before closing the set.
        self.indent = self.indent.saturating_sub(1);

        self.writeln_str("}\n")
    }

    fn remove_prefix_from_type(&self, ty: &str) -> String {
        // We remove the `prefix` from condition types, as these are local to Grafbase, and
        // should not be sent to the upstream server.
        let wrappers = WrappingType::all_for(ty);
        let mut out = String::with_capacity(ty.len());
        for wrapper in &wrappers {
            if let WrappingType::List = wrapper {
                write!(&mut out, "[").ok();
            }
        }

        let stripped_type = MetaTypeName::concrete_typename(ty);
        let stripped_type = stripped_type
            .strip_prefix(self.prefix.unwrap_or_default())
            .unwrap_or(stripped_type)
            .to_string();
        write!(&mut out, "{stripped_type}",).ok();

        for wrapper in wrappers.iter().rev() {
            match wrapper {
                WrappingType::List => write!(&mut out, "]").ok(),
                WrappingType::NonNull => write!(&mut out, "!").ok(),
            };
        }
        out
    }
}

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum Error {
    #[error(transparent)]
    Fmt(#[from] fmt::Error),

    /// A variable wasn't declared.
    ///
    /// This should really be caught well before we get here, but I'm
    /// not sure that it is
    #[error("Undeclared variable: {0}")]
    UndeclaredVariable(String),

    /// We couldn't find a field on the given type
    ///
    /// Should also be caught well before we get here, but not sure it is
    #[error("Couldn't find a field {0} on the type {1}")]
    UnknownField(String, String),

    /// An error from looking up types in the registry
    #[error("{}", .0.message)]
    RegistryError(crate::Error),
}

impl From<crate::Error> for Error {
    fn from(value: crate::Error) -> Self {
        Error::RegistryError(value)
    }
}

fn should_forward_directive(name: &str) -> bool {
    // For now we only support forwarding the skip & include directives.
    //
    // defer would need some implementation work to support forwarding
    matches!(name, "skip" | "include")
}
