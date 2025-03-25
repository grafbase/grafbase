use serde::{Deserialize, Serialize};

use crate::{cbor, wit, SdkError};

use super::Error;

/// List of resolver inputs, each containing the relevant response data associated with the
/// resolved item.
#[derive(Clone, Copy)]
pub struct FieldInputs<'a>(pub(crate) &'a [Vec<u8>]);

impl<'a> From<&'a Vec<Vec<u8>>> for FieldInputs<'a> {
    fn from(inputs: &'a Vec<Vec<u8>>) -> Self {
        FieldInputs(inputs.as_slice())
    }
}

impl std::fmt::Debug for FieldInputs<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FieldInputs").finish_non_exhaustive()
    }
}

impl<'a> FieldInputs<'a> {
    /// Number of items to be resolved.
    #[allow(clippy::len_without_is_empty)] // Never empty.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Deserialize each byte slice in the `FieldInputs` to a collection of items.
    pub fn deserialize<T>(&self) -> Result<Vec<T>, SdkError>
    where
        T: Deserialize<'a>,
    {
        self.0
            .iter()
            .map(|input| cbor::from_slice(input).map_err(Into::into))
            .collect()
    }
}

impl<'a> IntoIterator for FieldInputs<'a> {
    type Item = FieldInput<'a>;

    type IntoIter = FieldInputsIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FieldInputsIterator(self.0.iter().enumerate())
    }
}

/// Iterator over the resolver inputs.
pub struct FieldInputsIterator<'a>(std::iter::Enumerate<std::slice::Iter<'a, Vec<u8>>>);

impl<'a> Iterator for FieldInputsIterator<'a> {
    type Item = FieldInput<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(i, data)| FieldInput { data, ix: i as u32 })
    }
}

/// Response data, if any, for an item to resolve.
#[derive(Clone, Copy)]
pub struct FieldInput<'a> {
    data: &'a [u8],
    ix: u32,
}

impl std::fmt::Debug for FieldInput<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FieldInput").finish_non_exhaustive()
    }
}

impl<'a> FieldInput<'a> {
    /// Deserialize a resolver input.
    pub fn deserialize<T>(&self) -> Result<T, SdkError>
    where
        T: Deserialize<'a>,
    {
        cbor::from_slice(self.data).map_err(Into::into)
    }
}

/// Output for a resolver
pub struct FieldOutputs(wit::FieldOutput);

impl FieldOutputs {
    /// If your resolver doesn't depend on response data, this function provides a convenient way
    /// to create a FieldOutput from a single element.
    pub fn new<T: Serialize>(inputs: FieldInputs<'_>, data: T) -> Result<FieldOutputs, SdkError> {
        let data = cbor::to_vec(data)?;
        let outputs = if inputs.len() > 1 {
            inputs.0.iter().map(|_| Ok(data.clone())).collect()
        } else {
            vec![Ok(data)]
        };
        Ok(FieldOutputs(wit::FieldOutput { outputs }))
    }

    /// If your resolver doesn't depend on response data, this function provides a convenient way
    /// to create a FieldOutput from an error.
    pub fn error(inputs: FieldInputs<'_>, error: impl Into<Error>) -> FieldOutputs {
        let error: wit::Error = Into::<Error>::into(error).into();
        let outputs = if inputs.len() > 1 {
            inputs.0.iter().map(|_| Err(error.clone())).collect()
        } else {
            vec![Err(error)]
        };
        FieldOutputs(wit::FieldOutput { outputs })
    }

    /// Construct a new `FieldOutput` through an accumulator which allows setting the
    /// output individually for each `FieldInput`.
    pub fn builder(inputs: FieldInputs<'_>) -> FieldOutputsBuilder {
        FieldOutputsBuilder {
            items: vec![Ok(Vec::new()); inputs.len()],
        }
    }
}

/// Accumulator for setting the output individually for each `FieldInput`.
pub struct FieldOutputsBuilder {
    items: Vec<Result<Vec<u8>, wit::Error>>,
}

impl FieldOutputsBuilder {
    /// Push the output for a given `FieldInput`.
    pub fn insert<T: Serialize>(&mut self, input: FieldInput<'_>, data: T) -> Result<(), SdkError> {
        let data = cbor::to_vec(data)?;
        self.items[input.ix as usize] = Ok(data);
        Ok(())
    }

    /// Push an error for a given `FieldInput`.
    pub fn insert_error(&mut self, input: FieldInput<'_>, error: impl Into<Error>) {
        self.items[input.ix as usize] = Err(Into::<Error>::into(error).into());
    }

    /// Build the `FieldOutput`.
    pub fn build(self) -> FieldOutputs {
        FieldOutputs(wit::FieldOutput { outputs: self.items })
    }
}

impl From<FieldOutputs> for wit::FieldOutput {
    fn from(value: FieldOutputs) -> Self {
        value.0
    }
}
