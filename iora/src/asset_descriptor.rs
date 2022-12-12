use crate::{collection_utilities, AssetQuery, ListAssetsError, SemVer};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AssetDescriptor {
    pub name: String,
    pub version: SemVer,
    pub content_hash: String,
}

impl AssetDescriptor {
    pub fn new(name: &str, version: &SemVer, content_hash: &str) -> Self {
        AssetDescriptor {
            name: name.to_string(),
            version: version.clone(),
            content_hash: content_hash.to_string(),
        }
    }

    pub fn matches_query(&self, query: &AssetQuery) -> bool {
        let name_match = query.name_constraint.matches(&self.name);
        match &query.version_constraint {
            Some(vc) => name_match && vc.matches(&self.version),
            _ => name_match,
        }
    }

    pub fn filter_to_matching(
        descriptors: &[AssetDescriptor],
        query: &AssetQuery,
    ) -> Result<Vec<Self>, ListAssetsError> {
        let filtered_assets: Vec<&AssetDescriptor> = descriptors
            .iter()
            .filter(|&ad| ad.matches_query(query))
            .collect();
        let grouped = collection_utilities::group_by::<Vec<&AssetDescriptor>, String>(
            filtered_assets,
            |&ad| ad.name.to_string(),
        );
        let flattened: Vec<AssetDescriptor> =
            collection_utilities::reduce_to_max_by_key(&grouped, |ad| &ad.version);
        Ok(flattened)
    }
}

#[cfg(test)]
mod tests {
    use crate::{AssetDescriptor, NameConstraint, SemVer, VersionConstraint};
    use std::str::FromStr;

    #[test]
    fn asset_descriptor_match() {
        let ad = AssetDescriptor {
            name: "asset.name".to_string(),
            version: SemVer::from_str("23.45.678").unwrap(),
            content_hash: "content_hash".to_string(),
        };
        assert!(ad.matches_query(&NameConstraint::StartsWith("asset".to_string()).into()));
        assert!(!ad.matches_query(&NameConstraint::StartsWith("assert".to_string()).into()));
        assert!(ad.matches_query(
            &(
                NameConstraint::StartsWith("asset".to_string()),
                Some(VersionConstraint::MatchMajorVersionOnly(23))
            )
                .into()
        ));
        assert!(!ad.matches_query(
            &(
                NameConstraint::StartsWith("asset".to_string()),
                Some(VersionConstraint::MatchMajorVersionOnly(24))
            )
                .into()
        ));
        assert!(!ad.matches_query(
            &(
                NameConstraint::StartsWith("assert".to_string()),
                Some(VersionConstraint::MatchMajorVersionOnly(23))
            )
                .into()
        ));
    }
}
