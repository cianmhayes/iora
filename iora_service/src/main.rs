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
    AssetCatalog, AssetDescriptor, MockAssetCatalog, NameConstraint, SemVer, VersionConstraint, AssetQuery,
};
use std::sync::Arc;
use std::{net::SocketAddr, str::FromStr};

struct State {
    catalog_connection_pool: bb8::Pool<AssetCatalogConnectionManager>,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(State {
        catalog_connection_pool: bb8::Pool::builder()
            .build(AssetCatalogConnectionManager {})
            .await
            .unwrap(),
    });
    let app = Router::new()
        .route("/assets", get(list_assets))
        .layer(Extension(state));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
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
enum AssetCatalogConnectionError {
    NotImplemented,
}

struct AssetCatalogConnectionManager {}

#[async_trait]
impl ManageConnection for AssetCatalogConnectionManager {
    type Connection = iora::MockAssetCatalog;
    type Error = AssetCatalogConnectionError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let mut mock = MockAssetCatalog::new();
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

    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        false
    }
}

enum ListAssetsServiceError {
    BadNameConstraint,
    BadVersionConstraint,
    QueryFailed,
}

impl IntoResponse for ListAssetsServiceError {
    fn into_response(self) -> axum::response::Response {
        let response = {match self {
            ListAssetsServiceError::BadNameConstraint => (StatusCode::BAD_REQUEST, "Missing or bad name constraint"),
            ListAssetsServiceError::BadVersionConstraint => (StatusCode::BAD_REQUEST, "Missing or bad version constraint"),
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
    match (
        catalog,
        NameConstraint::from_str(&q.name),
        &q.version.map(|v| VersionConstraint::from_str(&v)),
    ) {
        (Err(catalog_err), _, _) => Err(ListAssetsServiceError::QueryFailed),
        (Ok(catalog), Ok(nc), Some(Ok(vc))) => match catalog.list_assets(&(&nc, vc).into()) {
            Ok(result) => Ok(Json::from(result)),
            _ => Err(ListAssetsServiceError::QueryFailed),
        },
        (Ok(catalog), Ok(nc), None) => match catalog.list_assets(&(nc, None).into()) {
            Ok(result) => Ok(Json::from(result)),
            _ => Err(ListAssetsServiceError::QueryFailed),
        },
        (_, Err(e), _) => Err(ListAssetsServiceError::BadNameConstraint),
        (_, _, Some(Err(e))) => Err(ListAssetsServiceError::BadVersionConstraint),
    }
}
