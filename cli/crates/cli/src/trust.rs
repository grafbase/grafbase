use common::trusted_documents::TrustedDocumentsManifest;

use crate::{cli_input::TrustCommand, errors::CliError, output::report};

pub(crate) fn trust(
    TrustCommand {
        project_ref,
        client_name,
        manifest,
    }: TrustCommand,
) -> Result<(), CliError> {
    let Some(branch) = project_ref.branch() else {
        return Err(CliError::MissingArgument("branch"));
    };

    let file = std::fs::File::open(manifest).map_err(CliError::TrustedDocumentsManifestReadError)?;
    let manifest: TrustedDocumentsManifest = serde_json::from_reader(file).map_err(|err| {
        CliError::TrustedDocumentsManifestReadError(std::io::Error::new(std::io::ErrorKind::InvalidData, err))
    })?;

    report::trust_start(&manifest);

    match backend::api::submit_trusted_documents::submit_trusted_documents(
        backend::api::submit_trusted_documents::TrustedDocumentsSubmitVariables {
            account: project_ref.account(),
            project: project_ref.project(),
            branch,
            client_name: &client_name,
            documents: manifest
                .into_documents()
                .map(
                    |common::trusted_documents::TrustedDocument {
                         document_id,
                         document_text,
                     }| backend::api::submit_trusted_documents::TrustedDocumentInput {
                        document_id,
                        document_text,
                    },
                )
                .collect(),
        },
    ) {
        Ok(payload) => match payload {
            backend::api::submit_trusted_documents::TrustedDocumentsSubmitPayload::TrustedDocumentsSubmitSuccess(
                success,
            ) => {
                report::trust_success(success.count);
            }
            backend::api::submit_trusted_documents::TrustedDocumentsSubmitPayload::ReusedIds(reused_ids) => {
                report::trust_reused_ids(&reused_ids)
            }
            backend::api::submit_trusted_documents::TrustedDocumentsSubmitPayload::OldToken(_) => {
                report::old_access_token()
            }
            backend::api::submit_trusted_documents::TrustedDocumentsSubmitPayload::Unknown => report::trust_failed(),
        },
        Err(err) => return Err(CliError::BackendApiError(err)),
    }

    Ok(())
}
