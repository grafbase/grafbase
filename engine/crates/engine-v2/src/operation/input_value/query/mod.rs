mod de;
mod ser;

use id_newtypes::IdRange;
use schema::{EnumValueId, InputValue, InputValueDefinitionId, SchemaInputValue, SchemaInputValueId};

use crate::operation::{OperationWalker, VariableDefinitionId};

#[derive(Default)]
pub struct QueryInputValues {
    /// Individual input values and list values
    values: Vec<QueryInputValue>,
    /// InputObject's fields
    input_fields: Vec<(InputValueDefinitionId, QueryInputValue)>,
    /// Object's fields (for JSON)
    key_values: Vec<(String, QueryInputValue)>,
}

id_newtypes::NonZeroU32! {
    QueryInputValues.values[QueryInputValueId] => QueryInputValue,
    QueryInputValues.input_fields[QueryInputObjectFieldValueId] => (InputValueDefinitionId, QueryInputValue),
    QueryInputValues.key_values[QueryInputKeyValueId] => (String, QueryInputValue),
}

#[derive(Default)]
pub enum QueryInputValue {
    #[default]
    Null,
    String(String),
    EnumValue(EnumValueId),
    Int(i32),
    BigInt(i64),
    Float(f64),
    Boolean(bool),
    InputObject(IdRange<QueryInputObjectFieldValueId>),
    List(IdRange<QueryInputValueId>),

    /// for JSON
    Map(IdRange<QueryInputKeyValueId>),
    U64(u64),

    DefaultValue(SchemaInputValueId),
    Variable(VariableDefinitionId),
}

impl QueryInputValues {
    pub fn push_value(&mut self, value: QueryInputValue) -> QueryInputValueId {
        let id = QueryInputValueId::from(self.values.len());
        self.values.push(value);
        id
    }

    /// Reserve InputValue slots for a list, avoiding the need for an intermediate
    /// Vec to hold values as we need them to be contiguous.
    pub fn reserve_list(&mut self, n: usize) -> IdRange<QueryInputValueId> {
        let start = self.values.len();
        self.values.reserve(n);
        for _ in 0..n {
            self.values.push(QueryInputValue::Null);
        }
        (start..self.values.len()).into()
    }
    /// Reserve InputKeyValue slots for a map, avoiding the need for an intermediate
    /// Vec to hold values as we need them to be contiguous.
    pub fn reserve_map(&mut self, n: usize) -> IdRange<QueryInputKeyValueId> {
        let start = self.key_values.len();
        self.key_values.reserve(n);
        for _ in 0..n {
            self.key_values.push((String::new(), QueryInputValue::Null));
        }
        (start..self.key_values.len()).into()
    }

    pub fn append_input_object(
        &mut self,
        fields: &mut Vec<(InputValueDefinitionId, QueryInputValue)>,
    ) -> IdRange<QueryInputObjectFieldValueId> {
        let start = self.input_fields.len();
        self.input_fields.append(fields);
        (start..self.input_fields.len()).into()
    }
}

pub type QueryInputValueWalker<'a> = OperationWalker<'a, &'a QueryInputValue, ()>;

impl<'a> QueryInputValueWalker<'a> {
    pub fn is_undefined(&self) -> bool {
        match self.item {
            QueryInputValue::Variable(id) => self.walk(*id).is_undefined(),
            _ => false,
        }
    }

    /// Used for GraphQL query generation to only include values in the query string that would be
    /// present after query normalization.
    pub fn to_normalized_query_const_value_str(self) -> Option<&'a str> {
        Some(match self.item {
            QueryInputValue::EnumValue(id) => self.schema_walker.walk(*id).name(),
            QueryInputValue::Boolean(b) => {
                if *b {
                    "true"
                } else {
                    "false"
                }
            }
            QueryInputValue::DefaultValue(id) => match &self.schema_walker.as_ref()[*id] {
                SchemaInputValue::EnumValue(id) => self.schema_walker.walk(*id).name(),
                SchemaInputValue::Boolean(b) => {
                    if *b {
                        "true"
                    } else {
                        "false"
                    }
                }
                _ => return None,
            },
            _ => return None,
        })
    }
}

impl<'a> From<QueryInputValueWalker<'a>> for InputValue<'a> {
    fn from(walker: QueryInputValueWalker<'a>) -> Self {
        match walker.item {
            QueryInputValue::Null => InputValue::Null,
            QueryInputValue::String(s) => InputValue::String(s.as_str()),
            QueryInputValue::EnumValue(id) => InputValue::EnumValue(*id),
            QueryInputValue::Int(n) => InputValue::Int(*n),
            QueryInputValue::BigInt(n) => InputValue::BigInt(*n),
            QueryInputValue::Float(f) => InputValue::Float(*f),
            QueryInputValue::Boolean(b) => InputValue::Boolean(*b),
            QueryInputValue::InputObject(ids) => {
                let mut fields = Vec::with_capacity(ids.len());
                for (input_value_definition_id, value) in &walker.operation[*ids] {
                    let value = walker.walk(value);
                    // https://spec.graphql.org/October2021/#sec-Input-Objects.Input-Coercion
                    if !value.is_undefined() {
                        fields.push((*input_value_definition_id, value.into()));
                    }
                }
                InputValue::InputObject(fields.into_boxed_slice())
            }
            QueryInputValue::List(ids) => {
                let mut values = Vec::with_capacity(ids.len());
                for id in *ids {
                    values.push(walker.walk(&walker.operation[id]).into());
                }
                InputValue::List(values.into_boxed_slice())
            }
            QueryInputValue::Map(ids) => {
                let mut key_values = Vec::with_capacity(ids.len());
                for (key, value) in &walker.operation[*ids] {
                    let value = walker.walk(value);
                    key_values.push((key.as_ref(), value.into()));
                }
                InputValue::Map(key_values.into_boxed_slice())
            }
            QueryInputValue::U64(n) => InputValue::U64(*n),
            QueryInputValue::DefaultValue(id) => walker.schema_walker.walk(&walker.schema_walker.as_ref()[*id]).into(),
            QueryInputValue::Variable(id) => walker.walk(*id).to_input_value().unwrap_or_default(),
        }
    }
}

impl std::fmt::Debug for QueryInputValueWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.item {
            QueryInputValue::Null => write!(f, "Null"),
            QueryInputValue::String(s) => s.fmt(f),
            QueryInputValue::EnumValue(id) => f
                .debug_tuple("EnumValue")
                .field(&self.schema_walker.walk(*id).name())
                .finish(),
            QueryInputValue::Int(n) => f.debug_tuple("Int").field(n).finish(),
            QueryInputValue::BigInt(n) => f.debug_tuple("BigInt").field(n).finish(),
            QueryInputValue::U64(n) => f.debug_tuple("U64").field(n).finish(),
            QueryInputValue::Float(n) => f.debug_tuple("Float").field(n).finish(),
            QueryInputValue::Boolean(b) => b.fmt(f),
            QueryInputValue::InputObject(ids) => {
                let mut map = f.debug_struct("InputObject");
                for (input_value_definition_id, value) in &self.operation[*ids] {
                    map.field(
                        self.schema_walker.walk(*input_value_definition_id).name(),
                        &self.walk(value),
                    );
                }
                map.finish()
            }
            QueryInputValue::List(ids) => {
                let mut seq = f.debug_list();
                for value in &self.operation[*ids] {
                    seq.entry(&self.walk(value));
                }
                seq.finish()
            }
            QueryInputValue::Map(ids) => {
                let mut map = f.debug_map();
                for (key, value) in &self.operation[*ids] {
                    map.entry(&key, &self.walk(value));
                }
                map.finish()
            }
            QueryInputValue::DefaultValue(id) => f
                .debug_tuple("DefaultValue")
                .field(&self.schema_walker.walk(&self.schema_walker.as_ref()[*id]))
                .finish(),
            QueryInputValue::Variable(id) => f.debug_tuple("Variable").field(&self.walk(*id)).finish(),
        }
    }
}
