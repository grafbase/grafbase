use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub indices: HashMap<String, IndexConfig>,
}

// Only contains the schema now but might have more later on.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct IndexConfig {
    pub schema: Schema,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Schema {
    pub fields: HashMap<String, FieldEntry>,
}

// Can be used to store anything relevant to the field that doesn't impact the index directly
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct FieldEntry {
    pub ty: FieldType,
}

// enum names MUST match their GraphQL scalar counterpart
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize, strum::IntoStaticStr)]
pub enum FieldType {
    URL(FieldOptions),
    Email(FieldOptions),
    PhoneNumber(FieldOptions),
    String(FieldOptions),
    Date(FieldOptions),
    DateTime(FieldOptions),
    Timestamp(FieldOptions),
    Int(FieldOptions),
    Float(FieldOptions),
    Boolean(FieldOptions),
    IPAddress(FieldOptions),
}

#[derive(Clone, Eq, PartialEq, Default, Hash, Debug, Serialize, Deserialize)]
pub struct FieldOptions {
    pub nullable: bool,
}

// Utility functions, essentially for more readable tests
impl FieldType {
    pub fn scalar_name(&self) -> &'static str {
        From::from(self)
    }

    pub fn is_nullable(&self) -> bool {
        match self {
            FieldType::URL(opts)
            | FieldType::Email(opts)
            | FieldType::PhoneNumber(opts)
            | FieldType::String(opts)
            | FieldType::Date(opts)
            | FieldType::DateTime(opts)
            | FieldType::Timestamp(opts)
            | FieldType::Int(opts)
            | FieldType::Float(opts)
            | FieldType::Boolean(opts)
            | FieldType::IPAddress(opts) => opts.nullable,
        }
    }

    pub fn url() -> Self {
        Self::URL(FieldOptions::default())
    }

    pub fn email() -> Self {
        Self::Email(FieldOptions::default())
    }

    pub fn phone() -> Self {
        Self::PhoneNumber(FieldOptions::default())
    }

    pub fn string() -> Self {
        Self::String(FieldOptions::default())
    }

    pub fn date() -> Self {
        Self::Date(FieldOptions::default())
    }

    pub fn datetime() -> Self {
        Self::DateTime(FieldOptions::default())
    }

    pub fn timestamp() -> Self {
        Self::Timestamp(FieldOptions::default())
    }

    pub fn int() -> Self {
        Self::Int(FieldOptions::default())
    }

    pub fn float() -> Self {
        Self::Float(FieldOptions::default())
    }

    pub fn bool() -> Self {
        Self::Boolean(FieldOptions::default())
    }

    pub fn ip() -> Self {
        Self::IPAddress(FieldOptions::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_backwards_compatbility() {
        let config = r#"
        {
          "indices": {
            "dummy": {
              "schema": {
                "fields": {
                  "ip": {
                    "ty": {
                      "IPAddress": {
                        "nullable": true
                      }
                    }
                  },
                  "timestamp": {
                    "ty": {
                      "Timestamp": {
                        "nullable": false
                      }
                    }
                  },
                  "text": {
                    "ty": {
                      "String": {
                        "nullable": false
                      }
                    }
                  },
                  "url": {
                    "ty": {
                      "URL": {
                        "nullable": false
                      }
                    }
                  },
                  "datetime": {
                    "ty": {
                      "DateTime": {
                        "nullable": false
                      }
                    }
                  },
                  "email": {
                    "ty": {
                      "Email": {
                        "nullable": false
                      }
                    }
                  },
                  "int": {
                    "ty": {
                      "Int": {
                        "nullable": false
                      }
                    }
                  },
                  "phone": {
                    "ty": {
                      "PhoneNumber": {
                        "nullable": false
                      }
                    }
                  },
                  "float": {
                    "ty": {
                      "Float": {
                        "nullable": false
                      }
                    }
                  },
                  "date": {
                    "ty": {
                      "Date": {
                        "nullable": false
                      }
                    }
                  },
                  "bool": {
                    "ty": {
                      "Boolean": {
                        "nullable": false
                      }
                    }
                  }
                }
              }
            }
          }
        }
        "#;
        assert_eq!(
            serde_json::from_str::<Config>(config).unwrap(),
            Config {
                indices: HashMap::from([(
                    "dummy".to_string(),
                    IndexConfig {
                        schema: Schema {
                            fields: HashMap::from([
                                ("url".to_string(), FieldEntry { ty: FieldType::url() }),
                                ("email".to_string(), FieldEntry { ty: FieldType::email() }),
                                ("phone".to_string(), FieldEntry { ty: FieldType::phone() }),
                                (
                                    "text".to_string(),
                                    FieldEntry {
                                        ty: FieldType::string()
                                    }
                                ),
                                ("date".to_string(), FieldEntry { ty: FieldType::date() }),
                                (
                                    "datetime".to_string(),
                                    FieldEntry {
                                        ty: FieldType::datetime()
                                    }
                                ),
                                (
                                    "timestamp".to_string(),
                                    FieldEntry {
                                        ty: FieldType::timestamp()
                                    }
                                ),
                                ("int".to_string(), FieldEntry { ty: FieldType::int() }),
                                ("float".to_string(), FieldEntry { ty: FieldType::float() }),
                                ("bool".to_string(), FieldEntry { ty: FieldType::bool() }),
                                (
                                    "ip".to_string(),
                                    FieldEntry {
                                        ty: FieldType::IPAddress(FieldOptions { nullable: true })
                                    }
                                ),
                            ])
                        }
                    }
                )])
            }
        );
    }
}
