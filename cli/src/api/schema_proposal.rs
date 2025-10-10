use super::{
    client::create_client,
    errors::{ApiError, SchemaProposalError, SchemaProposalParserError},
    graphql::{
        mutations::schema_proposal::{
            SchemaProposalCreateArguments, SchemaProposalCreateInput, SchemaProposalCreateMutation,
            SchemaProposalCreatePayload, SchemaProposalEditArguments, SchemaProposalEditInput,
            SchemaProposalEditMutation, SchemaProposalEditPayload, SchemaProposalEditSubgraph,
        },
        queries::fetch_branch_by_ref::{FetchBranchByRefArguments, FetchBranchByRefQuery},
    },
};
use crate::common::environment::PlatformData;
use cynic::{Id, MutationBuilder, QueryBuilder, http::ReqwestExt};

pub(crate) struct SchemaEditSubgraph<'a> {
    pub name: &'a str,
    pub schema: Option<&'a str>,
}

pub async fn create(
    account_slug: &str,
    graph_slug: &str,
    branch_name: &str,
    title: &str,
    description: Option<&str>,
) -> Result<String, ApiError> {
    let branch_id = fetch_branch_id(account_slug, graph_slug, branch_name).await?;
    let platform_data = PlatformData::get();
    let client = create_client()?;

    let operation = SchemaProposalCreateMutation::build(SchemaProposalCreateArguments {
        input: SchemaProposalCreateInput {
            title,
            branch_id,
            description,
        },
    });

    let cynic::GraphQlResponse { data, errors } = client.post(platform_data.api_url()).run_graphql(operation).await?;

    let Some(data) = data else {
        return Err(ApiError::RequestError(format!("{errors:#?}")));
    };

    match data.schema_proposal_create {
        SchemaProposalCreatePayload::SchemaProposalCreateSuccess(success) => {
            Ok(success.schema_proposal.id.into_inner())
        }
        SchemaProposalCreatePayload::Unknown(message) => Err(SchemaProposalError::Unknown { message }.into()),
    }
}

pub async fn edit(
    schema_proposal_id: &str,
    description: Option<&str>,
    subgraphs: &[SchemaEditSubgraph<'_>],
) -> Result<(), ApiError> {
    let platform_data = PlatformData::get();
    let client = create_client()?;

    let mut subgraph_inputs = Vec::with_capacity(subgraphs.len());

    for subgraph in subgraphs {
        subgraph_inputs.push(SchemaProposalEditSubgraph {
            name: subgraph.name,
            schema: subgraph.schema,
        });
    }

    let operation = SchemaProposalEditMutation::build(SchemaProposalEditArguments {
        input: SchemaProposalEditInput {
            schema_proposal_id: Id::new(schema_proposal_id),
            subgraphs: subgraph_inputs,
            description,
        },
    });

    let cynic::GraphQlResponse { data, errors } = client.post(platform_data.api_url()).run_graphql(operation).await?;

    let Some(data) = data else {
        return Err(ApiError::RequestError(format!("{errors:#?}")));
    };

    match data.schema_proposal_edit {
        SchemaProposalEditPayload::SchemaProposalEditSuccess(_) => Ok(()),
        SchemaProposalEditPayload::SchemaProposalDoesNotExistError(_) => {
            Err(SchemaProposalError::ProposalDoesNotExist {
                proposal_id: schema_proposal_id.to_owned(),
            }
            .into())
        }
        SchemaProposalEditPayload::SchemaProposalEditParserErrors(errors) => {
            Err(SchemaProposalError::EditParserErrors {
                errors: errors
                    .errors
                    .into_iter()
                    .map(|error| SchemaProposalParserError {
                        subgraph_name: error.subgraph_name,
                        error: error.error,
                        span_start: error.span_start,
                        span_end: error.span_end,
                    })
                    .collect(),
            }
            .into())
        }
        SchemaProposalEditPayload::Unknown(message) => Err(SchemaProposalError::Unknown { message }.into()),
    }
}

async fn fetch_branch_id(account_slug: &str, graph_slug: &str, branch_name: &str) -> Result<Id, ApiError> {
    let platform_data = PlatformData::get();
    let client = create_client()?;

    let operation = FetchBranchByRefQuery::build(FetchBranchByRefArguments {
        account_slug,
        graph_slug,
        branch_name,
    });

    let cynic::GraphQlResponse { data, errors } = client.post(platform_data.api_url()).run_graphql(operation).await?;

    let Some(data) = data else {
        return Err(ApiError::RequestError(format!("{errors:#?}")));
    };

    let branch = data.branch.ok_or_else(|| SchemaProposalError::BranchNotFound {
        branch_ref: format!("{account_slug}/{graph_slug}@{branch_name}"),
    })?;

    Ok(branch.id)
}
