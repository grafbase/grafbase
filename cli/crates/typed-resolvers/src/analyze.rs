use graphql_parser::schema as ast;
use std::{collections::HashMap, ops, str::FromStr};

pub(crate) fn analyze<'doc>(graphql_document: &'doc ast::Document<'doc, &'doc str>) -> AnalyzedSchema<'doc> {
    let mut schema = AnalyzedSchema::default();

    analyze_top_level(graphql_document, &mut schema);
    analyze_fields(graphql_document, &mut schema);

    schema
}

/// First pass. Resolve what definitions exist in the schema (objects, unions, etc.).
fn analyze_top_level<'doc>(graphql_document: &'doc ast::Document<'doc, &'doc str>, schema: &mut AnalyzedSchema<'doc>) {
    for definition in &graphql_document.definitions {
        match definition {
            ast::Definition::DirectiveDefinition(_) | ast::Definition::SchemaDefinition(_) => (), // not interested
            ast::Definition::TypeDefinition(ast::TypeDefinition::Object(output_type)) => {
                schema.push_output_type(Object {
                    name: output_type.name,
                    docs: output_type.description.as_deref(),
                    kind: ObjectKind::Object,
                });
            }
            ast::Definition::TypeDefinition(ast::TypeDefinition::InputObject(object_definition)) => {
                schema.push_output_type(Object {
                    name: object_definition.name,
                    docs: object_definition.description.as_deref(),
                    kind: ObjectKind::InputObject,
                });
            }
            ast::Definition::TypeDefinition(ast::TypeDefinition::Interface(object_definition)) => {
                schema.push_output_type(Object {
                    name: object_definition.name,
                    docs: object_definition.description.as_deref(),
                    kind: ObjectKind::Interface,
                });
            }
            ast::Definition::TypeDefinition(ast::TypeDefinition::Scalar(scalar)) => {
                schema.push_custom_scalar(CustomScalar {
                    name: scalar.name,
                    docs: scalar.description.as_deref(),
                });
            }
            ast::Definition::TypeDefinition(ast::TypeDefinition::Union(union_definition)) => {
                schema.push_union(Union {
                    name: union_definition.name,
                });
            }
            ast::Definition::TypeDefinition(ast::TypeDefinition::Enum(enum_definition)) => {
                let id = schema.push_enum(Enum {
                    name: enum_definition.name,
                    docs: enum_definition.description.as_deref(),
                });

                for variant in &enum_definition.values {
                    schema.push_enum_variant(id, variant.name);
                }
            }
            ast::Definition::TypeExtension(_) => {
                // We ignore type extensions in the first pass because we can't know if they extend
                // types defined in the document yet.
            }
        }
    }
}

/// Second pass. We know about all definitions, now we analyze fields inside object and interface
/// types, and variants inside unions.
fn analyze_fields<'doc>(graphql_document: &'doc ast::Document<'doc, &'doc str>, schema: &mut AnalyzedSchema<'doc>) {
    for definition in &graphql_document.definitions {
        match definition {
            ast::Definition::DirectiveDefinition(_) | ast::Definition::SchemaDefinition(_) => (), // not interested
            ast::Definition::TypeDefinition(ast::TypeDefinition::Union(union_definition)) => {
                let Definition::Union(union_id) = schema.definition_names[union_definition.name] else {
                    continue;
                };

                for variant_name in &union_definition.types {
                    match schema.definition_names.get(variant_name) {
                        Some(Definition::Object(object_id)) => schema.push_union_variant(union_id, *object_id),
                        None | Some(_) => (), // invalid: union variant is not an object name. Ignore here.
                    }
                }
            }
            ast::Definition::TypeDefinition(ast::TypeDefinition::Object(object_definition)) => {
                let Definition::Object(object_id) = schema.definition_names[object_definition.name] else {
                    continue;
                };

                for field in &object_definition.fields {
                    if let Some(field) = analyze_ast_field(field, schema) {
                        schema.object_fields.push((object_id, field));
                    }
                }
            }
            ast::Definition::TypeDefinition(ast::TypeDefinition::InputObject(object_definition)) => {
                let Definition::Object(object_id) = schema.definition_names[object_definition.name] else {
                    continue;
                };

                for field in &object_definition.fields {
                    if let Some(field) = analyze_ast_input_field(field, schema) {
                        schema.object_fields.push((object_id, field));
                    }
                }
            }
            ast::Definition::TypeDefinition(ast::TypeDefinition::Interface(object_definition)) => {
                let Definition::Object(object_id) = schema.definition_names[object_definition.name] else {
                    continue;
                };

                for field in &object_definition.fields {
                    if let Some(field) = analyze_ast_field(field, schema) {
                        schema.object_fields.push((object_id, field));
                    }
                }
            }
            ast::Definition::TypeExtension(ast::TypeExtension::Object(obj)) => {
                let Some(Definition::Object(extended_object_id)) = schema.definition_names.get(obj.name) else {
                    continue;
                };

                for field in &obj.fields {
                    if let Some(field) = analyze_ast_field(field, schema) {
                        schema.object_fields.push((*extended_object_id, field));
                    }
                }
            }
            ast::Definition::TypeExtension(ast::TypeExtension::InputObject(obj)) => {
                let Some(Definition::Object(extended_object_id)) = schema.definition_names.get(obj.name) else {
                    continue;
                };

                for field in &obj.fields {
                    if let Some(field) = analyze_ast_input_field(field, schema) {
                        schema.object_fields.push((*extended_object_id, field));
                    }
                }
            }
            // Already completely handled in first pass.
            ast::Definition::TypeDefinition(ast::TypeDefinition::Scalar(_) | ast::TypeDefinition::Enum(_))
            // Not handled
            | ast::Definition::TypeExtension(_) => {}
        }
    }

    schema.object_fields.sort_by_key(|(object_id, _)| *object_id);
}

fn analyze_ast_input_field<'doc>(
    field: &'doc ast::InputValue<'doc, &'doc str>,
    schema: &AnalyzedSchema<'doc>,
) -> Option<Field<'doc>> {
    let (name, inner_is_nullable, list_wrappers) = type_from_nested(&field.value_type);
    Some(Field {
        name: field.name,
        docs: field.description.as_deref(),
        kind: BuiltinScalar::from_str(name)
            .ok()
            .map(FieldTypeKind::BuiltinScalar)
            .or_else(|| schema.definition_names.get(name).map(|d| FieldTypeKind::Definition(*d)))?,
        inner_is_nullable,
        list_wrappers,
    })
}

fn analyze_ast_field<'doc>(
    field: &'doc ast::Field<'doc, &'doc str>,
    schema: &AnalyzedSchema<'doc>,
) -> Option<Field<'doc>> {
    let (name, inner_is_nullable, list_wrappers) = type_from_nested(&field.field_type);
    Some(Field {
        name: field.name,
        docs: field.description.as_deref(),
        kind: BuiltinScalar::from_str(name)
            .ok()
            .map(FieldTypeKind::BuiltinScalar)
            .or_else(|| schema.definition_names.get(name).map(|d| FieldTypeKind::Definition(*d)))?,
        inner_is_nullable,
        list_wrappers,
    })
}

fn type_from_nested<'a>(ty: &ast::Type<'a, &'a str>) -> (&'a str, bool, Vec<ListWrapper>) {
    match ty {
        ast::Type::NonNullType(ty) => match ty.as_ref() {
            ast::Type::ListType(inner) => {
                let (name, inner_is_nullable, mut wrappers) = type_from_nested(inner);
                wrappers.push(ListWrapper::NonNullList);
                (name, inner_is_nullable, wrappers)
            }
            ast::Type::NamedType(name) => (name, false, Vec::new()),
            ast::Type::NonNullType(_) => {
                unreachable!("unreachable double required (!!)");
            }
        },
        ast::Type::NamedType(name) => (name, true, Vec::new()),
        ast::Type::ListType(inner) => {
            let (name, inner, mut wrappers) = type_from_nested(inner);
            wrappers.push(ListWrapper::NullableList);
            (name, inner, wrappers)
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct AnalyzedSchema<'doc> {
    pub(crate) definitions: Vec<Definition>,

    // Index mapping names to definitions.
    definition_names: HashMap<&'doc str, Definition>,

    pub(crate) objects: Vec<Object<'doc>>,

    object_fields: Vec<(ObjectId, Field<'doc>)>,

    pub(crate) unions: Vec<Union<'doc>>,
    // Invariant: This is sorted because we iterate unions in order. We rely on that for binary
    // search.
    pub(crate) union_variants: Vec<(UnionId, ObjectId)>,

    pub(crate) enums: Vec<Enum<'doc>>,
    // Invariant: This is sorted. We rely on that for binary search.
    pub(crate) enum_variants: Vec<(EnumId, &'doc str)>,

    pub(crate) custom_scalars: Vec<CustomScalar<'doc>>,
}

#[derive(Debug)]
pub(crate) struct Field<'doc> {
    pub(crate) name: &'doc str,
    pub(crate) kind: FieldTypeKind,
    pub(crate) docs: Option<&'doc str>,

    inner_is_nullable: bool,
    // TODO: a more compact and/or normalized representation
    /// The list wrapper types, from innermost to outermost.
    list_wrappers: Vec<ListWrapper>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ListWrapper {
    NullableList,
    NonNullList,
}

impl Field<'_> {
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

#[derive(Debug)]
pub(crate) enum FieldTypeKind {
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
    Int,
    Float,
    String,
    Boolean,
    Id,
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
            _ => Err(()),
        }
    }
}

impl<'doc> AnalyzedSchema<'doc> {
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
    Interface,
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
