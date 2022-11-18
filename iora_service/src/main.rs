use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    async_trait,
    extract::Extension,
    extract::Query,
    response::Json,
    routing::get,
    Router,
};
use bb8::ManageConnection;
use iora::{
    AssetCatalog, AssetDescriptor, MockAssetCatalog, SemVer, AssetQuery, ConstraintParsingError,
};
use std::sync::Arc;
use std::{net::SocketAddr, str::FromStr};


use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "iora")]
#[command(bin_name = "iora_service")]
struct IoraServiceParameters {
    #[arg(short, long, value_name = "PORT", required = true)]
    port: u16,
}

struct State {
    catalog_connection_pool: bb8::Pool<AssetCatalogConnectionManager>,
}

#[tokio::main]
async fn main() {
    let args = IoraServiceParameters::parse();
    let state = Arc::new(State {
        catalog_connection_pool: bb8::Pool::builder()
            .build(AssetCatalogConnectionManager {})
            .await
            .unwrap(),
    });
    let app = Router::new()
        .route("/assets", get(list_assets))
        .layer(Extension(state));
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(serde::Deserialize)]
struct ListAssetParameters {
    name: String,
    version: Option<String>,
}

#[derive(Debug)]
enum AssetCatalogConnectionError {}

struct AssetCatalogConnectionManager {}

#[async_trait]
impl ManageConnection for AssetCatalogConnectionManager {
    type Connection = iora::MockAssetCatalog;
    type Error = AssetCatalogConnectionError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let mock = MockAssetCatalog::new();
        mock.descriptors.borrow_mut().push(AssetDescriptor::new(
            "asset_en",
            &SemVer::from_str("1.0.0-beta+buildinfo").unwrap(),
            "hash1",
        ));
        mock.descriptors.borrow_mut().push(AssetDescriptor::new(
            "asset_de",
            &SemVer::from_str("2.0.0").unwrap(),
            "hash1",
        ));
        mock.descriptors.borrow_mut().push(AssetDescriptor::new(
            "asset_de",
            &SemVer::from_str("2.0.1").unwrap(),
            "hash1",
        ));
        mock.descriptors.borrow_mut().push(AssetDescriptor::new(
            "other_asset_en",
            &SemVer::from_str("1.0.0").unwrap(),
            "hash1",
        ));
        Ok(mock)
    }

    async fn is_valid(&self, _conn: &mut Self::Connection) -> Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

enum ListAssetsServiceError {
    MissingNameConstraint,
    MalformedNameConstraint,
    MalformedVersionConstraint,
    QueryFailed,
}

impl From<ConstraintParsingError> for ListAssetsServiceError {
    fn from(e: ConstraintParsingError) -> Self {
        match e {
            ConstraintParsingError::EmptyNameConstraint => Self::MissingNameConstraint,
            ConstraintParsingError::UnrecognizedNameConstraintStructure => Self::MalformedNameConstraint,
            ConstraintParsingError::EmptyVersionConstraint => Self::MalformedVersionConstraint,
            ConstraintParsingError::UnrecognizedVersionConstraintStructure => Self::MalformedVersionConstraint
        }
    }
}

impl IntoResponse for ListAssetsServiceError {
    fn into_response(self) -> axum::response::Response {
        let response = {match self {
            ListAssetsServiceError::MissingNameConstraint => (StatusCode::BAD_REQUEST, "Missing name constraint"),
            ListAssetsServiceError::MalformedNameConstraint => (StatusCode::BAD_REQUEST, "Bad name constraint"),
            ListAssetsServiceError::MalformedVersionConstraint => (StatusCode::BAD_REQUEST, "Bad version constraint"),
            ListAssetsServiceError::QueryFailed => (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong")
        }};
        response.into_response()
    }
}

async fn list_assets(
    Query(q): Query<ListAssetParameters>,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<Vec<iora::AssetDescriptor>>, ListAssetsServiceError> {
    let catalog = state.catalog_connection_pool.get().await;
    let query = AssetQuery::new_from_strings(&q.name, &q.version);
    match (
        catalog,
        query,
    ) {
        (Err(_), _) => Err(ListAssetsServiceError::QueryFailed),
        (Ok(catalog), Ok(query)) => match catalog.list_assets(&query) {
            Ok(result) => Ok(Json::from(result)),
            _ => Err(ListAssetsServiceError::QueryFailed),
        },
        (_, Err(e)) => Err(e.into()),
    }
}
