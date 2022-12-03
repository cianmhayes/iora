use std::str::FromStr;

use quick_xml::de::from_str;
use serde::{Deserialize, Serialize};

use crate::{AssetDescriptor, AssetIndex, AssetQuery, ListAssetsError, SemVer};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Metadata {
    name: String,
    version: String,
    sha1: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
struct Blob {
    name: String,
    metadata: Option<Metadata>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct NextMarker {}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Blobs {
    #[serde(rename = "$value")]
    blobs: Vec<Blob>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
struct EnumerationResults {
    blobs: Blobs,
    next_marker: NextMarker,
}

impl EnumerationResults {
    pub fn evaluate_query(&self, query: &AssetQuery) -> Vec<AssetDescriptor> {
        let mut results: Vec<AssetDescriptor> = vec![];
        for b in self.blobs.blobs.iter() {
            if let Some(m) = &b.metadata {
                let ad = AssetDescriptor::new(
                    &m.name,
                    &SemVer::from_str(&m.version).unwrap_or_default(),
                    &m.sha1,
                );
                if ad.matches_query(query) {
                    results.push(ad);
                }
            }
        }
        results
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
struct Error {
    code: String,
    message: String,
}

impl From<Error> for ListAssetsError {
    fn from(e: Error) -> Self {
        match e.code.as_str() {
            "AuthenticationFailed" => ListAssetsError::AssetIndexAccessDenied(Some(e.message)),
            "InvalidQueryParameterValue" => ListAssetsError::AssetIndexInternalError(e.message),
            _ => ListAssetsError::AssetIndexInternalError(format!(
                "Storage error '{}'. Details: {}",
                e.code, e.message
            )),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum ListBlobResponse {
    EnumerationResults(EnumerationResults),
    Error(Error),
}

pub struct AzureBlobAssetIndex {
    storage_account_name: String,
    container_name: String,
    sas: String,
}

impl AzureBlobAssetIndex {
    pub fn new(storage_account_name: &str, container_name: &str, sas: &str) -> Self {
        AzureBlobAssetIndex {
            storage_account_name: storage_account_name.to_owned(),
            container_name: container_name.to_owned(),
            sas: sas.to_owned(),
        }
    }

    fn make_request(url: String) -> Result<ListBlobResponse, crate::ListAssetsError> {
        if let Ok(response) = reqwest::blocking::get(url) {
            if let Ok(response_text) = response.text() {
                from_str::<ListBlobResponse>(response_text.trim_start_matches(|c| c != '<'))
                    .map_err(|parse_error| {
                        ListAssetsError::AssetIndexInternalError(format!(
                            "Failed to parse reponse. message: {} content: {}",
                            parse_error, &response_text
                        ))
                    })
            } else {
                Err(ListAssetsError::AssetIndexInternalError(
                    "Storage response was empty.".to_owned(),
                ))
            }
        } else {
            Err(ListAssetsError::AssetIndexInternalError(
                "No response from storage.".to_owned(),
            ))
        }
    }
}

impl AssetIndex for AzureBlobAssetIndex {
    fn list_assets(
        &self,
        query: &crate::AssetQuery,
    ) -> Result<Vec<crate::AssetDescriptor>, crate::ListAssetsError> {
        let url = format!(
            "https://{}.blob.core.windows.net/{}?restype=container&comp=list&include=metadata&{}",
            &self.storage_account_name, &self.container_name, &self.sas
        );
        match Self::make_request(url) {
            Ok(ListBlobResponse::EnumerationResults(results)) => Ok(results.evaluate_query(query)),
            Ok(ListBlobResponse::Error(e)) => Err(e.into()),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::AssetQuery;

    use super::ListBlobResponse;
    use quick_xml::de::from_str;

    #[test]
    fn parse_list_blobs_response() {
        let response = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <EnumerationResults ServiceEndpoint="https://ioratest.blob.core.windows.net/" ContainerName="assets">
            <Blobs>
                <Blob>
                    <Name>simple_test/1.0.0/asset.tar.gz</Name>
                    <Properties>
                        <Creation-Time>Tue, 29 Nov 2022 19:11:21 GMT</Creation-Time>
                        <Last-Modified>Tue, 29 Nov 2022 19:29:51 GMT</Last-Modified>
                        <Etag>0x8DAD24015DA24EF</Etag>
                        <Content-Length>266</Content-Length>
                        <Content-Type>application/octet-stream</Content-Type>
                        <Content-Encoding>gzip</Content-Encoding>
                        <Content-Language />
                        <Content-CRC64 />
                        <Content-MD5>/d2WLJaQLTNQLBpT11w1dg==</Content-MD5>
                        <Cache-Control />
                        <Content-Disposition />
                        <BlobType>BlockBlob</BlobType>
                        <AccessTier>Hot</AccessTier>
                        <AccessTierInferred>true</AccessTierInferred>
                        <LeaseStatus>unlocked</LeaseStatus>
                        <LeaseState>available</LeaseState>
                        <ServerEncrypted>true</ServerEncrypted>
                    </Properties>
                    <Metadata>
                        <version>1.0.0</version>
                        <sha1>a5dc94e2414b5445ddb4658b047166751f364f4a</sha1>
                        <name>simple_test</name>
                    </Metadata>
                    <OrMetadata />
                </Blob>
            </Blobs>
            <NextMarker />
        </EnumerationResults>"#;
        if let ListBlobResponse::EnumerationResults(results) =
            from_str::<ListBlobResponse>(response).unwrap()
        {
            let query = AssetQuery::new_from_strings("s*", &None).unwrap();
            let descriptors = results.evaluate_query(&query);
            assert_eq!(descriptors.len(), 1);
            let ad = descriptors.first().unwrap();
            assert_eq!(ad.name, "simple_test");
            assert_eq!(ad.version.major, 1);
            assert_eq!(ad.version.minor, 0);
            assert_eq!(ad.version.patch, 0);
            assert!(ad.version.prerelease.is_none());
            assert!(ad.version.buildmetadata.is_none());
            assert_eq!(ad.content_hash, "a5dc94e2414b5445ddb4658b047166751f364f4a");
        } else {
            panic!("Unexpected parse result");
        }
    }
}
