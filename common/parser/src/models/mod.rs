//! Models are the internal representation in the Grafbase database of every entities
//! It's the source of truth of every values stored inside the database.
//!
//! The idea is to generate from the *User inputed schema* every associated models
//!
//! For instance:
//!
//! type Something @model {
//!   id: ID!
//!   name: String!
//!   nameList: [String!]!
//!   age: Int
//!   relation: Truc @relation
//! }
//!
//! type Truc @model {
//!   id: ID!
//!   something: Something! @relation
//! }
//!
//! Would produce a schema like:
//!
//! Something {
//!   id: [Utf8]
//!   name: [Utf8]
//!   nameList: [List<["": Utf8]>]
//!   age: [i64?]
//! }
//!
//! It doesn't include the related data from the implementation which would for
//! instance need the `parent_id` or the `relation_name`.
//!
//! The relation is also not modelized in the Entity schema type. A relation is a link between
//! entities with possible metadata which will be something else.
//!
//! # Custom Scalar
//!
//! For custom scalar, the schema should reflect the internal coercion in a standarzied type.

use arrow_schema::{DataType, Field, Schema};
use dynaql::registry::{MetaType, Registry};
use dynaql_parser::types::{BaseType, Type};
use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum ConversionError {
        ParsingSchema(err: String) {
            display("parsing schema error: {}", err)
        }
        Unknown {
            display("Unknown")
        }
    }
}

fn primitive_to_datatype(registry: &Registry, scalar: &str) -> DataType {
    match scalar {
        "ID" => DataType::Utf8,
        "String" => DataType::Utf8,
        "Int" => DataType::Int64,
        "Float" => DataType::Float64,
        "Boolean" => DataType::Boolean,
        "DateTime" => DataType::Utf8,
        rest => {
            let meta_ty = registry.types.get(rest).expect("can't find the scalar: {scalar}");

            if meta_ty.is_leaf() {
                // It means it's a scalar not supported or an enum.
                return DataType::Utf8;
            }

            DataType::Struct(from_meta_type(registry, meta_ty).expect("can't fail").fields)
        }
    }
}

fn temp_base_tyto_datatype(registry: &Registry, scalar: &BaseType) -> DataType {
    match scalar {
        // Here it **HAS** to be a Scalar we know as we cancel every relations.
        BaseType::Named(value) => primitive_to_datatype(registry, value),
        BaseType::List(list) => {
            let base_data = scalar_to_datatype(registry, "", list);
            DataType::List(Box::new(base_data))
        }
    }
}

//  nameList: [String!]! -> nameList: List<[Utf8]>
//  nameList: [String]! -> nameList: List<[Utf8;?]>
//  nameList: [String] -> nameList [List<[Utf8;?]>;?]
fn scalar_to_datatype(registry: &Registry, field: &str, scalar: &Type) -> Field {
    Field::new(
        field.to_string(),
        temp_base_tyto_datatype(registry, &scalar.base),
        scalar.nullable,
    )
}

/// System fields for Entities
pub fn entity_fields() -> Vec<Field> {
    vec![Field::new("__type", DataType::Utf8, false)]
}

pub fn from_meta_type(registry: &Registry, ty: &MetaType) -> Result<Schema, ConversionError> {
    match ty {
        // input @ MetaType::InputObject { .. } => from_meta_type_input(registry, input),
        obj @ MetaType::Object { .. } => from_meta_type_object(registry, obj),
        _ => Err(ConversionError::Unknown),
    }
}

/// We have a [`MetaType`] which we want to store in our Main Database, we compute the schema out
/// of it.
///
/// -> It must be an Object
/// -> For each field:
///   -> Is not a relation
///   -> We map every custom scalar by the internal representation associated
pub fn from_meta_type_object(registry: &Registry, ty: &MetaType) -> Result<Schema, ConversionError> {
    if let MetaType::Object {
         ref fields, ..
    } = ty
    {
        let mut arrow_fields = Vec::with_capacity(fields.len());
        for (_key, field) in fields {
            if field.relation.is_none() {
                let ty = Type::new(&field.ty).ok_or_else(|| {
                    ConversionError::ParsingSchema(format!("The Type {ty} is not a proper GraphQL type", ty = field.ty))
                })?;

                let arrow_field = scalar_to_datatype(registry, &field.name, &ty);
                arrow_fields.push(arrow_field);
            }
        }

        arrow_fields.extend(entity_fields());
        return Ok(Schema::new(arrow_fields));
    }
    Err(ConversionError::ParsingSchema(format!(
        "The Type {name} is not an Object, we can't infer the proper schema.",
        name = ty.name()
    )))
}

/// We have a [`MetaType`] which we want to store in our Main Database, we compute the schema out
/// of it.
pub fn from_meta_type_input(registry: &Registry, ty: &MetaType) -> Result<Schema, ConversionError> {
    if let MetaType::InputObject {
        
        ref input_fields,
        ..
    } = ty
    {
        let mut arrow_fields = Vec::with_capacity(input_fields.len());
        for (_key, input_value) in input_fields {
            let ty = Type::new(&input_value.ty).ok_or_else(|| {
                ConversionError::ParsingSchema(format!(
                    "The Type {ty} is not a proper GraphQL type",
                    ty = input_value.ty
                ))
            })?;

            let arrow_field = scalar_to_datatype(registry, &input_value.name, &ty);
            arrow_fields.push(arrow_field);
        }

        arrow_fields.extend(entity_fields());
        return Ok(Schema::new(arrow_fields));
    }
    Err(ConversionError::ParsingSchema(format!(
        "The Type {name} is not an Object, we can't infer the proper schema.",
        name = ty.name()
    )))
}
