use crate::{
    api, cli_input::TrustCommand, common::trusted_documents::TrustedDocumentsManifest, errors::CliError, output::report,
};

pub(crate) fn trust(
    TrustCommand {
        graph_ref,
        client_name,
        manifest,
    }: TrustCommand,
) -> Result<(), CliError> {
    let Some(branch) = graph_ref.branch() else {
        return Err(CliError::MissingArgument("branch"));
    };

    let file = std::fs::File::open(manifest).map_err(CliError::TrustedDocumentsManifestReadError)?;
    let manifest: TrustedDocumentsManifest =
        serde_json::from_reader(file).map_err(CliError::TrustedDocumentsManifestParseError)?;

    report::trust_start(&manifest);

    match api::submit_trusted_documents::submit_trusted_documents(
        api::submit_trusted_documents::TrustedDocumentsSubmitVariables {
            account: graph_ref.account(),
            graph: graph_ref.graph(),
            branch,
            client_name: &client_name,
            documents: manifest
                .into_documents()
                .map(
                    |crate::common::trusted_documents::TrustedDocument {
                         document_id,
                         document_text,
                     }| api::submit_trusted_documents::TrustedDocumentInput {
                        document_id,
                        document_text,
                    },
                )
                .collect(),
        },
    ) {
        Ok(payload) => match payload {
            api::submit_trusted_documents::TrustedDocumentsSubmitPayload::TrustedDocumentsSubmitSuccess(success) => {
                report::trust_success(success.count);
            }
            api::submit_trusted_documents::TrustedDocumentsSubmitPayload::ReusedIds(reused_ids) => {
                report::trust_reused_ids(&reused_ids)
            }
            api::submit_trusted_documents::TrustedDocumentsSubmitPayload::OldToken(_) => report::old_access_token(),
            api::submit_trusted_documents::TrustedDocumentsSubmitPayload::Unknown => report::trust_failed(),
        },
        Err(err) => return Err(CliError::BackendApiError(err)),
    }

    Ok(())
}
