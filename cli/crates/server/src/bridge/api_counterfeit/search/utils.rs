use tantivy::schema::{IndexRecordOption, Schema as TantivySchema, TextFieldIndexing, TextOptions};
use tantivy::schema::{INDEXED, STORED, STRING};

use super::tokenizer::TOKENIZER_NAME;

use super::{FieldType, Schema};

pub const ID_FIELD: &str = "#id";
const TOKENIZED_PREFIX: &str = "tokenized#";

pub(crate) fn tokenized_field_name(name: &str) -> String {
    format!("{TOKENIZED_PREFIX}{name}")
}

pub(super) fn to_tantivy(schema: &Schema) -> TantivySchema {
    use FieldType::{Boolean, Date, DateTime, Email, Float, IPAddress, Int, PhoneNumber, String, Timestamp, URL};

    let mut builder = TantivySchema::builder();
    builder.add_bytes_field(ID_FIELD, INDEXED | STORED);
    for (name, entry) in &schema.fields {
        match entry.ty {
            URL { .. } | Email { .. } | String { .. } => {
                // Storing the "raw" field directly avoiding any tokenization. This allows us
                // to provide a sensible filter API. Otherwise filtering on a String "Hello
                // world!" would end up filtering on the tokens ["hello", "world"].
                builder.add_text_field(name, STRING);
                builder.add_text_field(
                    &tokenized_field_name(name),
                    // equivalent to the standard TEXT with our tokenzier
                    TextOptions::default().set_indexing_options(
                        TextFieldIndexing::default()
                            .set_tokenizer(TOKENIZER_NAME)
                            .set_fieldnorms(true)
                            .set_index_option(IndexRecordOption::WithFreqsAndPositions),
                    ),
                )
            }
            // There is little benefit to tokenize phone numbers currently.
            PhoneNumber { .. } => builder.add_text_field(name, STRING),
            Date { .. } | DateTime { .. } | Timestamp { .. } => builder.add_date_field(name, INDEXED),
            Int { .. } => builder.add_i64_field(name, INDEXED),
            Float { .. } => builder.add_f64_field(name, INDEXED),
            Boolean { .. } => builder.add_bool_field(name, INDEXED),
            IPAddress { .. } => builder.add_ip_addr_field(name, INDEXED),
        };
    }
    builder.build()
}
