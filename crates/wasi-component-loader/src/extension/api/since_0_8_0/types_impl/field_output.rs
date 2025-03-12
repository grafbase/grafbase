use engine::{ErrorCode, GraphqlError};
use runtime::extension::Data;

use super::super::wit::grafbase::sdk::types::FieldOutput;

impl From<FieldOutput> for Vec<Result<Data, GraphqlError>> {
    fn from(value: FieldOutput) -> Self {
        let mut results = Vec::new();

        for result in value.outputs {
            match result {
                Ok(data) => results.push(Ok(Data::CborBytes(data))),
                Err(error) => {
                    let error = error.into_graphql_error(ErrorCode::InternalServerError);
                    results.push(Err(error))
                }
            }
        }

        results
    }
}
