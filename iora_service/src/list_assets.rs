use crate::IoraServiceState;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{extract::Extension, extract::Query, response::Json};
use iora::{AssetIndex, AssetQuery, ConstraintParsingError, ListAssetsError};
use serde_json::json;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ListAssetsServiceError {
    #[error("Asset index is missing or unavailable. {0:?}")]
    AssetIndexNotFound(Option<String>),
    #[error("Asset index refused access. {0:?}")]
    AssetIndexAccessDenied(Option<String>),
    #[error("Failed to execute the query. Details: {0}")]
    AssetIndexInternalError(String),
    #[error("A name constraint is required but none was provided.")]
    MissingNameConstraint,
    #[error("A name constraint is required but the provided constraint was malformed: '{0}'.")]
    MalformedNameConstraint(String),
    #[error("The provided version constraint was malformed: '{0}'.")]
    MalformedVersionConstraint(String),
    #[error("Failed to execute the query. Details: {details:?}. Query: {query:?}")]
    BadQuery { query: String, details: String },
}

impl From<ConstraintParsingError> for ListAssetsServiceError {
    fn from(e: ConstraintParsingError) -> Self {
        match e {
            ConstraintParsingError::EmptyNameConstraint => Self::MissingNameConstraint,
            ConstraintParsingError::UnrecognizedNameConstraintStructure(s) => {
                Self::MalformedNameConstraint(s)
            }
            ConstraintParsingError::EmptyVersionConstraint => {
                Self::MalformedVersionConstraint("".to_owned())
            }
            ConstraintParsingError::UnrecognizedVersionConstraintStructure(s) => {
                Self::MalformedVersionConstraint(s)
            }
        }
    }
}

impl From<ListAssetsError> for ListAssetsServiceError {
    fn from(e: ListAssetsError) -> Self {
        match e {
            ListAssetsError::AssetIndexNotFound(s) => Self::AssetIndexNotFound(s),
            ListAssetsError::AssetIndexAccessDenied(s) => Self::AssetIndexAccessDenied(s),
            ListAssetsError::BadQuery { query, details } => Self::BadQuery { query, details },
            ListAssetsError::AssetIndexInternalError(s) => Self::AssetIndexInternalError(s),
            ListAssetsError::MisconfiguredIndex(s) => Self::AssetIndexInternalError(s)
        }
    }
}

impl IntoResponse for ListAssetsServiceError {
    fn into_response(self) -> axum::response::Response {
        let message = self.to_string();
        let mapping = match self {
            ListAssetsServiceError::BadQuery {
                query: _,
                details: _,
            } => (StatusCode::BAD_REQUEST, "BadQuery".to_owned()),
            ListAssetsServiceError::MissingNameConstraint => (StatusCode::BAD_REQUEST, "MissingNameConstraint".to_owned()),
            ListAssetsServiceError::MalformedNameConstraint(_) => (StatusCode::BAD_REQUEST, "MalformedNameConstraint".to_owned()),
            ListAssetsServiceError::MalformedVersionConstraint(_) => (StatusCode::BAD_REQUEST, "MalformedVersionConstraint".to_owned()),
            ListAssetsServiceError::AssetIndexAccessDenied(_) => (StatusCode::INTERNAL_SERVER_ERROR, "AssetIndexAccessDenied".to_owned()),
            ListAssetsServiceError::AssetIndexNotFound(_) => (StatusCode::INTERNAL_SERVER_ERROR, "AssetIndexNotFound".to_owned()),
            ListAssetsServiceError::AssetIndexInternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "AssetIndexInternalError".to_owned()),
        };
        (mapping.0, json!({ "code": mapping.1, "message": message}).to_string()).into_response()
    }
}

#[derive(serde::Deserialize)]
pub struct ListAssetParameters {
    name: String,
    version: Option<String>,
}

pub async fn list_assets(
    Query(q): Query<ListAssetParameters>,
    Extension(state): Extension<Arc<IoraServiceState>>,
) -> Result<Json<Vec<iora::AssetDescriptor>>, ListAssetsServiceError> {
    let catalog = state.asset_index_connection_pool.get().await;
    let query = AssetQuery::new_from_strings(&q.name, &q.version);
    match (catalog, query) {
        (Ok(catalog), Ok(query)) => match catalog.list_assets(&query) {
            Ok(result) => Ok(Json::from(result)),
            Err(list_error) => Err(list_error.into()),
        },
        (Err(_), _) => Err(ListAssetsServiceError::AssetIndexNotFound(None)),
        (_, Err(e)) => Err(e.into()),
    }
}
