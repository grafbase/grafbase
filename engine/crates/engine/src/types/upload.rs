use std::{borrow::Cow, fs::File, io::Read};

use crate::{
    registry::{self, LegacyRegistryExt},
    ContextField, InputValueError, InputValueResult, LegacyInputType, Value,
};

/// A file upload value.
pub struct UploadValue {
    /// The name of the file.
    pub filename: String,
    /// The content type of the file.
    pub content_type: Option<String>,
    /// The file data.
    pub content: File,
}

impl UploadValue {
    /// Attempt to clone the upload value. This type's `Clone` implementation simply calls this and
    /// panics on failure.
    ///
    /// # Errors
    ///
    /// Fails if cloning the inner `File` fails.
    pub fn try_clone(&self) -> std::io::Result<Self> {
        Ok(Self {
            filename: self.filename.clone(),
            content_type: self.content_type.clone(),
            content: self.content.try_clone()?,
        })
    }

    /// Convert to a `Read`.
    ///
    /// **Note**: this is a *synchronous/blocking* reader.
    pub fn into_read(self) -> impl Read + Sync + Send + 'static {
        self.content
    }

    /// Returns the size of the file, in bytes.
    pub fn size(&self) -> std::io::Result<u64> {
        self.content.metadata().map(|meta| meta.len())
    }
}

/// Uploaded file
///
/// **Reference:** <https://github.com/jaydenseric/graphql-multipart-request-spec>
///
///
/// Graphql supports file uploads via `multipart/form-data`.
/// Enable this feature by accepting an argument of type `Upload` (single file) or
/// `Vec<Upload>` (multiple files) in your mutation like in the example blow.
///
///
/// # Example
///
/// ```
/// use engine::*;
///
/// struct Mutation;
///
/// #[Object]
/// impl Mutation {
///     async fn upload(&self, ctx: &Context<'_>, file: Upload) -> bool {
///         println!("upload: filename={}", file.value(ctx).unwrap().filename);
///         true
///     }
/// }
///
/// ```
/// # Example Curl Request
///
/// Assuming you have defined your Mutation like in the example above,
/// you can now upload a file `myFile.txt` with the below curl command:
///
/// ```curl
/// curl 'localhost:8000' \
/// --form 'operations={
///         "query": "mutation ($file: Upload!) { upload(file: $file)  }",
///         "variables": { "file": null }}' \
/// --form 'map={ "0": ["variables.file"] }' \
/// --form '0=@myFile.txt'
/// ```
pub struct Upload(usize);

impl Upload {
    /// Get the upload value.
    pub fn value(&self, ctx: &ContextField<'_>) -> std::io::Result<UploadValue> {
        ctx.query_env.uploads[self.0].try_clone()
    }
}

impl LegacyInputType for Upload {
    type RawValueType = Self;

    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("Upload")
    }

    fn create_type_info(registry: &mut registry::Registry) -> crate::registry::InputValueType {
        registry.create_input_type::<Self, _>(|_| {
            registry::MetaType::Scalar(registry::ScalarType {
                name: Self::type_name().to_string(),
                description: None,
                is_valid: Some(|value| matches!(value, Value::String(_))),
                specified_by_url: Some("https://github.com/jaydenseric/graphql-multipart-request-spec".to_string()),
                parser: registry::ScalarParser::BestEffort,
            })
        })
    }

    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        const PREFIX: &str = "#__graphql_file__:";
        let value = value.unwrap_or_default();
        if let Value::String(s) = &value {
            if let Some(filename) = s.strip_prefix(PREFIX) {
                return Ok(Upload(filename.parse::<usize>().unwrap()));
            }
        }
        Err(InputValueError::expected_type(value))
    }

    fn to_value(&self) -> Value {
        Value::Null
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }
}
