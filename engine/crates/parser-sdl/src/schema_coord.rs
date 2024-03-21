/// Helper enum for specifying the location of errors
#[derive(Clone, Copy, Debug)]
pub enum SchemaCoord<'a> {
    Field(&'a str, &'a str),
    Entity(&'a str, &'a str),
}

impl std::fmt::Display for SchemaCoord<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaCoord::Field(ty, field) => {
                write!(f, "{ty}.{field}")
            }
            SchemaCoord::Entity(ty, key) => {
                write!(f, "federation key `{key}` on the type {ty}")
            }
        }
    }
}

impl SchemaCoord<'_> {
    pub fn into_owned(self) -> OwnedSchemaCoord {
        match self {
            SchemaCoord::Field(ty, field) => OwnedSchemaCoord::Field(ty.into(), field.into()),
            SchemaCoord::Entity(ty, key) => OwnedSchemaCoord::Entity(ty.into(), key.into()),
        }
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum OwnedSchemaCoord {
    Field(String, String),
    Entity(String, String),
}

impl std::fmt::Display for OwnedSchemaCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OwnedSchemaCoord::Field(ty, field) => {
                write!(f, "{ty}.{field}")
            }
            OwnedSchemaCoord::Entity(ty, key) => {
                write!(f, "federation key `{key}` on the type {ty}")
            }
        }
    }
}
