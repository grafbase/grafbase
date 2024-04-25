use engine_parser::types as ast;
use engine_value::ConstValue;
use std::{collections::HashMap, iter, ops, str::FromStr};

#[must_use]
pub fn analyze(graphql_document: &ast::ServiceDocument) -> AnalyzedSchema<'_> {
    let mut schema = AnalyzedSchema::default();

    analyze_top_level(graphql_document, &mut schema);
    analyze_fields(graphql_document, &mut schema);

    schema
}

/// First pass. Resolve what definitions exist in the schema (objects, unions, etc.).
fn analyze_top_level<'doc>(graphql_document: &'doc ast::ServiceDocument, schema: &mut AnalyzedSchema<'doc>) {
    for definition in &graphql_document.definitions {
        match definition {
            ast::TypeSystemDefinition::Type(ty) => {
                if ty.node.extend {
                    // We ignore type extensions in the first pass because we can't know if they extend
                    // types defined in the document yet.
                    continue;
                }
                let name = &ty.node.name.node;
                let docs = ty.node.description.as_ref().map(|desc| desc.node.as_str());

                match &ty.node.kind {
                    ast::TypeKind::Scalar => {
                        schema.push_custom_scalar(CustomScalar { name, docs });
                    }
                    ast::TypeKind::Object(obj) => {
                        schema.push_output_type(Object {
                            name,
                            docs,
                            kind: ObjectKind::Object,
                        });

                        for interface in &obj.implements {
                            schema.interface_implementations.push(InterfaceImplementation {
                                interface: interface.node.as_str(),
                                implementer: name,
                            });
                        }
                    }
                    ast::TypeKind::InputObject(_) => {
                        schema.push_output_type(Object {
                            name,
                            docs,
                            kind: ObjectKind::InputObject,
                        });
                    }
                    ast::TypeKind::Union(_) => {
                        schema.push_union(Union { name });
                    }
                    ast::TypeKind::Enum(r#enum) => {
                        let id = schema.push_enum(Enum { name, docs });

                        for variant in &r#enum.values {
                            schema.push_enum_variant(id, &variant.node.value.node);
                        }
                    }
                    // Interfaces are only interesting insofar as they are implemented, so they are handled in the Object branch.
                    ast::TypeKind::Interface(_) => {}
                }
            }
            ast::TypeSystemDefinition::Schema(_) | ast::TypeSystemDefinition::Directive(_) => (), // not interested
        }
    }

    for name in ["Query", "Mutation", "Subscription"] {
        if !schema.definition_names.contains_key(name) {
            schema.push_output_type(Object {
                name,
                docs: None,
                kind: ObjectKind::Object,
            });
        }
    }

    schema.interface_implementations.sort();
}

/// Second pass. We know about all definitions, now we analyze fields inside object and interface
/// types, and variants inside unions.
fn analyze_fields<'doc>(graphql_document: &'doc ast::ServiceDocument, schema: &mut AnalyzedSchema<'doc>) {
    for definition in &graphql_document.definitions {
        match definition {
            ast::TypeSystemDefinition::Type(ty) => {
                let name = ty.node.name.node.as_str();
                match &ty.node.kind {
                    ast::TypeKind::Union(union_definition) => {
                        let Some(Definition::Union(union_id)) = schema.definition_names.get(name).copied() else {
                            continue;
                        };

                        for variant_name in &union_definition.members {
                            match schema.definition_names.get(variant_name.node.as_str()) {
                                Some(Definition::Object(object_id)) => schema.push_union_variant(union_id, *object_id),
                                None | Some(_) => (), // invalid: union variant is not an object name. Ignore here.
                            }
                        }
                    }
                    ast::TypeKind::Object(object_definition) => {
                        let Some(Definition::Object(object_id)) = schema.definition_names.get(name).copied() else {
                            continue;
                        };

                        for ast_field in &object_definition.fields {
                            if let Some(field) = analyze_ast_field(&ast_field.node, schema) {
                                let field_id = schema.push_object_field(object_id, field);
                                for arg in &ast_field.node.arguments {
                                    if let Some(r#type) = GraphqlType::resolve(&arg.node.ty.node, schema) {
                                        schema.object_field_args.push((
                                            field_id,
                                            FieldArgument {
                                                name: arg.node.name.node.as_str(),
                                                r#type,
                                            },
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    ast::TypeKind::Interface(iface_definition) => {
                        let Some(Definition::Object(object_id)) = schema.definition_names.get(name).copied() else {
                            continue;
                        };

                        for field in &iface_definition.fields {
                            if let Some(field) = analyze_ast_field(&field.node, schema) {
                                schema.object_fields.push((object_id, field));
                            }
                        }
                    }
                    ast::TypeKind::InputObject(input_object_definition) => {
                        let Some(Definition::Object(object_id)) = schema.definition_names.get(name).copied() else {
                            continue;
                        };

                        for field in &input_object_definition.fields {
                            if let Some(field) = analyze_ast_input_field(&field.node, schema) {
                                schema.object_fields.push((object_id, field));
                            }
                        }
                    }
                    // Already completely handled in first pass.
                    ast::TypeKind::Scalar | ast::TypeKind::Enum(_) => (),
                }
            }

            ast::TypeSystemDefinition::Schema(_) | ast::TypeSystemDefinition::Directive(_) => (), // not interested
        }
    }

    schema.object_fields.sort_by_key(|(object_id, _)| *object_id);
    schema.object_field_args.sort_by_key(|(object_id, _)| *object_id);
}

#[derive(Debug)]
pub(crate) struct GraphqlType {
    pub(crate) kind: TypeKind,
    inner_is_nullable: bool,
    // TODO: a more compact and/or normalized representation
    /// The list wrapper types, from innermost to outermost.
    list_wrappers: Vec<ListWrapper>,
}

impl GraphqlType {
    fn resolve(ty: &ast::Type, schema: &AnalyzedSchema<'_>) -> Option<Self> {
        let (name, inner_is_nullable, list_wrappers) = type_from_nested(ty);
        Some(GraphqlType {
            inner_is_nullable,
            list_wrappers,
            kind: BuiltinScalar::from_str(name)
                .ok()
                .map(TypeKind::BuiltinScalar)
                .or_else(|| schema.definition_names.get(name).map(|d| TypeKind::Definition(*d)))?,
        })
    }

    /// Returns whether the innermost type is nullable.
    ///
    /// For example `[Test]!` would return true, `[Test!]` would return false, `Test` would be
    /// true, and `Test!` would be false.
    pub(crate) fn inner_is_nullable(&self) -> bool {
        self.inner_is_nullable
    }

    /// Iterate list wrapper types from innermost to outermost.
    ///
    /// Example: `[[Int!]!]` would yield `NonNull, NonNullList, List`. You can then look up the
    /// unwrapped type in `OutputField::kind`.
    pub(crate) fn iter_list_wrappers(&self) -> impl Iterator<Item = ListWrapper> + '_ {
        self.list_wrappers.iter().copied()
    }
}

fn analyze_ast_input_field<'doc>(
    field: &'doc ast::InputValueDefinition,
    schema: &AnalyzedSchema<'doc>,
) -> Option<Field<'doc>> {
    Some(Field {
        name: field.name.node.as_str(),
        docs: field.description.as_ref().map(|d| d.node.as_str()),
        r#type: GraphqlType::resolve(&field.ty.node, schema)?,
        resolver_name: None, // no resolvers on input fields
        has_arguments: false,
    })
}

fn analyze_ast_field<'doc>(field: &'doc ast::FieldDefinition, schema: &AnalyzedSchema<'doc>) -> Option<Field<'doc>> {
    let resolver_name = field
        .directives
        .iter()
        .find(|directive| directive.node.name.node == "resolver")
        .and_then(|directive| {
            directive
                .node
                .arguments
                .iter()
                .find(|(name, _)| name.node == "name")
                .and_then(|(_, value)| match &value.node {
                    ConstValue::String(s) => Some(s.clone()),
                    _ => None,
                })
        });
    let r#type = GraphqlType::resolve(&field.ty.node, schema)?;

    Some(Field {
        name: field.name.node.as_str(),
        docs: field.description.as_ref().map(|d| d.node.as_str()),
        r#type,
        resolver_name,
        has_arguments: !field.arguments.is_empty(),
    })
}

fn type_from_nested(ty: &ast::Type) -> (&str, bool, Vec<ListWrapper>) {
    if ty.nullable {
        match &ty.base {
            ast::BaseType::Named(name) => (name, true, Vec::new()),
            ast::BaseType::List(inner) => {
                let (name, inner, mut wrappers) = type_from_nested(inner);
                wrappers.push(ListWrapper::NullableList);
                (name, inner, wrappers)
            }
        }
    } else {
        match &ty.base {
            ast::BaseType::List(inner) => {
                let (name, inner_is_nullable, mut wrappers) = type_from_nested(inner);
                wrappers.push(ListWrapper::NonNullList);
                (name, inner_is_nullable, wrappers)
            }
            ast::BaseType::Named(name) => (name, false, Vec::new()),
        }
    }
}

#[derive(Default, Debug)]
pub struct AnalyzedSchema<'doc> {
    pub(crate) definitions: Vec<Definition>,

    // Index mapping names to definitions.
    pub(crate) definition_names: HashMap<&'doc str, Definition>,

    pub(crate) objects: Vec<Object<'doc>>,

    object_fields: Vec<(ObjectId, Field<'doc>)>,
    object_field_args: Vec<(FieldId, FieldArgument<'doc>)>,

    interface_implementations: Vec<InterfaceImplementation<'doc>>,

    pub(crate) unions: Vec<Union<'doc>>,
    // Invariant: This is sorted because we iterate unions in order. We rely on that for binary
    // search.
    pub(crate) union_variants: Vec<(UnionId, ObjectId)>,

    pub(crate) enums: Vec<Enum<'doc>>,
    // Invariant: This is sorted. We rely on that for binary search.
    pub(crate) enum_variants: Vec<(EnumId, &'doc str)>,

    pub(crate) custom_scalars: Vec<CustomScalar<'doc>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct InterfaceImplementation<'doc> {
    pub(crate) interface: &'doc str,
    pub(crate) implementer: &'doc str,
}

#[derive(Debug)]
pub(crate) struct FieldArgument<'doc> {
    pub(crate) name: &'doc str,
    pub(crate) r#type: GraphqlType,
}

#[derive(Debug)]
pub(crate) struct Field<'doc> {
    pub(crate) name: &'doc str,
    pub(crate) docs: Option<&'doc str>,
    pub(crate) r#type: GraphqlType,
    pub(crate) has_arguments: bool,

    /// ```graphql,ignore
    /// @resolver(name: "user/fullName")
    /// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// ```
    pub(crate) resolver_name: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ListWrapper {
    NullableList,
    NonNullList,
}

#[derive(Debug)]
pub(crate) enum TypeKind {
    BuiltinScalar(BuiltinScalar),
    Definition(Definition),
}

#[derive(Debug)]
pub(crate) struct CustomScalar<'doc> {
    pub(crate) name: &'doc str,
    pub(crate) docs: Option<&'doc str>,
}

#[derive(Debug)]
pub(crate) enum BuiltinScalar {
    // GraphQL builtin scalars
    Int,
    Float,
    String,
    Boolean,
    Id,

    // Grafbase builtin scalars. They are not defined in user configuration, so we can't treat them
    // as custom scalars. This will change once we base the resolver codegen on the registry.
    Email,
    Date,
    DateTime,
    IPAddress,
    Timestamp,
    Url,
    Json,
    PhoneNumber,
    Decimal,
    Bytes,
    BigInt,
}

impl FromStr for BuiltinScalar {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "String" => Ok(BuiltinScalar::String),
            "Int" => Ok(BuiltinScalar::Int),
            "Float" => Ok(BuiltinScalar::Float),
            "Boolean" => Ok(BuiltinScalar::Boolean),
            "ID" => Ok(BuiltinScalar::Id),
            "Email" => Ok(BuiltinScalar::Email),
            "Date" => Ok(BuiltinScalar::Date),
            "DateTime" => Ok(BuiltinScalar::DateTime),
            "IPAddress" => Ok(BuiltinScalar::IPAddress),
            "Timestamp" => Ok(BuiltinScalar::Timestamp),
            "Url" => Ok(BuiltinScalar::Url),
            "Json" => Ok(BuiltinScalar::Json),
            "PhoneNumber" => Ok(BuiltinScalar::PhoneNumber),
            "Decimal" => Ok(BuiltinScalar::Decimal),
            "Bytes" => Ok(BuiltinScalar::Bytes),
            "BigInt" => Ok(BuiltinScalar::BigInt),
            _ => Err(()),
        }
    }
}

impl<'doc> AnalyzedSchema<'doc> {
    pub(crate) fn iter_field_arguments(&self, field_id: FieldId) -> impl Iterator<Item = &FieldArgument<'_>> {
        let start = self.object_field_args.partition_point(|(id, _)| *id < field_id);
        self.object_field_args[start..]
            .iter()
            .take_while(move |(id, _)| field_id == *id)
            .map(|(_, arg)| arg)
    }

    pub(crate) fn iter_fields(&self) -> impl Iterator<Item = (&ObjectId, FieldId, &Field<'doc>)> {
        self.object_fields
            .iter()
            .enumerate()
            .map(|(idx, (object_id, field))| (object_id, FieldId(idx), field))
    }

    pub(crate) fn iter_interfaces(&self) -> impl Iterator<Item = (&'doc str, &[InterfaceImplementation<'doc>])> {
        let mut start_idx = 0;

        iter::from_fn(move || {
            let first_element = self.interface_implementations.get(start_idx)?;
            let end = self.interface_implementations[start_idx..]
                .iter()
                .position(|iface_impl| iface_impl.interface != first_element.interface)
                .map(|pos| start_idx + pos)
                .unwrap_or_else(|| self.interface_implementations.len());

            let elements = &self.interface_implementations[start_idx..end];
            start_idx = end;

            Some((first_element.interface, elements))
        })
    }

    pub(crate) fn iter_object_fields(&self, object_id: ObjectId) -> impl Iterator<Item = &Field<'doc>> {
        let start = self.object_fields.partition_point(|(id, _)| *id < object_id);
        self.object_fields[start..]
            .iter()
            .take_while(move |(id, _)| object_id == *id)
            .map(|(_, output_field)| output_field)
    }

    pub(crate) fn iter_enum_variants(&self, enum_id: EnumId) -> impl Iterator<Item = &'doc str> + '_ {
        let start = self.enum_variants.partition_point(|(id, _)| *id < enum_id);
        self.enum_variants[start..]
            .iter()
            .take_while(move |(id, _)| enum_id == *id)
            .map(|(_, variant)| *variant)
    }

    pub(crate) fn iter_union_variants(&self, union_id: UnionId) -> impl Iterator<Item = &Object<'doc>> {
        let start = self.union_variants.partition_point(|(id, _)| *id < union_id);
        self.union_variants[start..]
            .iter()
            .take_while(move |(id, _)| *id == union_id)
            .map(|(_, output_type_id)| &self[*output_type_id])
    }

    fn push_definition(&mut self, name: &'doc str, definition: Definition) {
        self.definitions.push(definition);
        self.definition_names.insert(name, definition);
    }

    fn push_custom_scalar(&mut self, scalar: CustomScalar<'doc>) -> CustomScalarId {
        let id = CustomScalarId(self.custom_scalars.len());
        self.push_definition(scalar.name, Definition::CustomScalar(id));
        self.custom_scalars.push(scalar);
        id
    }

    fn push_enum(&mut self, enum_definition: Enum<'doc>) -> EnumId {
        let id = EnumId(self.enums.len());
        self.push_definition(enum_definition.name, Definition::Enum(id));
        self.enums.push(enum_definition);
        id
    }

    fn push_enum_variant(&mut self, id: EnumId, variant: &'doc str) {
        self.enum_variants.push((id, variant));
    }

    fn push_object_field(&mut self, object_id: ObjectId, field: Field<'doc>) -> FieldId {
        let field_id = FieldId(self.object_fields.len());
        self.object_fields.push((object_id, field));
        field_id
    }

    fn push_output_type(&mut self, output_type: Object<'doc>) -> ObjectId {
        let id = ObjectId(self.objects.len());
        self.push_definition(output_type.name, Definition::Object(id));
        self.objects.push(output_type);
        id
    }

    fn push_union(&mut self, union: Union<'doc>) -> UnionId {
        let id = UnionId(self.unions.len());
        self.push_definition(union.name, Definition::Union(id));
        self.unions.push(union);
        id
    }

    fn push_union_variant(&mut self, union: UnionId, variant: ObjectId) {
        self.union_variants.push((union, variant));
    }

    pub fn push_custom_resolver(&mut self, custom_resolver: &crate::CustomResolver) {
        let Some(Definition::Object(parent_object_id)) = self
            .definition_names
            .get(custom_resolver.parent_type_name.as_str())
            .copied()
        else {
            return;
        };

        let partition_point = self.object_fields.partition_point(|(id, _)| *id < parent_object_id);

        for (object_id, field) in &mut self.object_fields[partition_point..] {
            if *object_id != parent_object_id {
                return;
            }

            if field.name == custom_resolver.field_name {
                assert!(field.resolver_name.is_none());
                field.resolver_name = Some(custom_resolver.resolver_name.to_owned());
            }
        }
    }
}

impl<'doc> ops::Index<CustomScalarId> for AnalyzedSchema<'doc> {
    type Output = CustomScalar<'doc>;

    fn index(&self, index: CustomScalarId) -> &Self::Output {
        &self.custom_scalars[index.0]
    }
}

impl<'doc> ops::Index<EnumId> for AnalyzedSchema<'doc> {
    type Output = Enum<'doc>;

    fn index(&self, index: EnumId) -> &Self::Output {
        &self.enums[index.0]
    }
}

impl<'doc> ops::Index<ObjectId> for AnalyzedSchema<'doc> {
    type Output = Object<'doc>;

    fn index(&self, index: ObjectId) -> &Self::Output {
        &self.objects[index.0]
    }
}

impl<'doc> ops::Index<UnionId> for AnalyzedSchema<'doc> {
    type Output = Union<'doc>;

    fn index(&self, index: UnionId) -> &Self::Output {
        &self.unions[index.0]
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Definition {
    CustomScalar(CustomScalarId),
    Enum(EnumId),
    Object(ObjectId),
    Union(UnionId),
}

#[derive(Debug)]
pub(crate) struct Object<'doc> {
    pub(crate) name: &'doc str,
    pub(crate) docs: Option<&'doc str>,
    pub(crate) kind: ObjectKind,
}

#[derive(Debug)]
pub(crate) enum ObjectKind {
    Object,
    InputObject,
}

#[derive(Debug)]
pub(crate) struct Union<'doc> {
    pub(crate) name: &'doc str,
}

#[derive(Debug)]
pub(crate) struct Enum<'doc> {
    pub(crate) name: &'doc str,
    pub(crate) docs: Option<&'doc str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub(crate) struct EnumId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ObjectId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub(crate) struct UnionId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub(crate) struct CustomScalarId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FieldId(usize);
